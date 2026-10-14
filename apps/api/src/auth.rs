use std::{collections::HashSet, env, sync::Arc};

use anyhow::{Context, Result, bail};
use axum::http::HeaderMap;
use lucid_auth::{
    AuthConfig, AuthService, BuiltinProvider, BuiltinProviderKind, SessionWithUser,
    postgres::PostgresStore,
};
use reqwest::Client;
use serde::Deserialize;
use sqlx08::{PgPool, postgres::PgPoolOptions};

use crate::ApiError;

const MANAGE_GUILD: u64 = 1 << 5;
const ADMINISTRATOR: u64 = 1 << 3;

#[derive(Clone)]
pub(crate) struct AuthState {
    pub service: Arc<AuthService>,
    owners: Arc<HashSet<String>>,
    http: Client,
    pub(crate) pool: PgPool,
    development_bypass: bool,
    operator_token: Option<Arc<str>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Actor {
    pub discord_user_id: String,
    pub is_owner: bool,
    pub guild_scope: Option<String>,
}

#[derive(Deserialize)]
struct DiscordGuild {
    id: String,
    permissions: String,
}

#[derive(Deserialize)]
struct DiscordProfile {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

impl AuthState {
    pub async fn new(database_url: &str, frontend_origin: &str) -> Result<Self> {
        let secret = required_env("BETTER_AUTH_SECRET")?;
        let auth_url = env::var("BETTER_AUTH_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3001/api/auth".to_owned());
        let development_bypass = env_flag("AUTH_DEVELOPMENT_BYPASS");
        let operator_token = env::var("ORIVANE_OPERATOR_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty())
            .map(|token| {
                if token.len() < 32 {
                    bail!("ORIVANE_OPERATOR_TOKEN must contain at least 32 characters");
                }
                Ok::<Arc<str>, anyhow::Error>(Arc::from(token))
            })
            .transpose()?;

        let mut config = AuthConfig::new(secret.into_bytes())?;
        config.set_base_url(&auth_url)?;
        config.trust_origin(frontend_origin)?;
        config.enable_cors();
        config.development_bypass = development_bypass;

        match (
            env::var("DISCORD_CLIENT_ID"),
            env::var("DISCORD_CLIENT_SECRET"),
        ) {
            (Ok(client_id), Ok(client_secret)) => {
                let mut discord =
                    BuiltinProvider::new(BuiltinProviderKind::Discord, client_id, client_secret);
                if !discord
                    .config_mut()
                    .scopes
                    .iter()
                    .any(|scope| scope == "guilds")
                {
                    discord.config_mut().scopes.push("guilds".to_owned());
                }
                config.add_social_provider(discord)?;
            }
            _ if !development_bypass => {
                bail!(
                    "DISCORD_CLIENT_ID and DISCORD_CLIENT_SECRET are required unless AUTH_DEVELOPMENT_BYPASS=true"
                )
            }
            _ => tracing::warn!("Discord OAuth is disabled by the explicit development bypass"),
        }

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("connect Lucid Auth PostgreSQL store")?;
        let store = Arc::new(PostgresStore::new(pool.clone(), Default::default()));
        let service = Arc::new(AuthService::try_new(store.clone(), config)?);
        let schema = store.migrate_all(&service.plugin_migrations()).await?;
        if !schema.compatible {
            bail!(
                "Lucid Auth database schema is incompatible: {:?}",
                schema.issues
            );
        }

        let owners = env::var("BOT_OWNER_DISCORD_IDS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
            .collect();

        Ok(Self {
            service,
            owners: Arc::new(owners),
            http: Client::new(),
            pool,
            development_bypass,
            operator_token,
        })
    }

    pub async fn actor(&self, headers: &HeaderMap) -> Result<Actor, ApiError> {
        if let Some(actor) = self.operator_actor(headers) {
            return Ok(actor);
        }
        let session = self.session(headers).await?;
        let discord = self
            .service
            .list_linked_accounts(&session)
            .await
            .map_err(auth_internal)?
            .into_iter()
            .find(|account| account.provider_id == "discord");

        if let Some(discord) = discord {
            self.refresh_discord_identity(&session, &discord.id).await;
            return Ok(Actor {
                is_owner: self.owners.contains(&discord.account_id),
                discord_user_id: discord.account_id,
                guild_scope: None,
            });
        }
        if self.development_bypass {
            return Ok(Actor {
                discord_user_id: "local-development".to_owned(),
                is_owner: true,
                guild_scope: None,
            });
        }
        Err(ApiError::unauthorized(
            "a linked Discord account is required",
        ))
    }

    pub async fn require_owner(&self, headers: &HeaderMap) -> Result<Actor, ApiError> {
        let actor = self.actor(headers).await?;
        if actor.is_owner {
            Ok(actor)
        } else {
            Err(ApiError::forbidden("bot owner access is required"))
        }
    }

    pub async fn require_access_token(&self, headers: &HeaderMap) -> Result<Actor, ApiError> {
        self.access_token_actor(headers)
            .await?
            .ok_or_else(|| ApiError::unauthorized("a valid Orivane MCP access token is required"))
    }

    pub async fn require_guild(
        &self,
        headers: &HeaderMap,
        guild_id: &str,
    ) -> Result<Actor, ApiError> {
        if let Some(actor) = self.operator_actor(headers) {
            return Ok(actor);
        }
        if let Some(actor) = self.access_token_actor(headers).await? {
            return if actor.guild_scope.as_deref() == Some(guild_id) {
                Ok(actor)
            } else {
                Err(ApiError::forbidden(
                    "this access token is scoped to a different server",
                ))
            };
        }
        self.require_dashboard_guild(headers, guild_id).await
    }

    pub async fn require_dashboard_guild(
        &self,
        headers: &HeaderMap,
        guild_id: &str,
    ) -> Result<Actor, ApiError> {
        if self.operator_actor(headers).is_some()
            || self.access_token_actor(headers).await?.is_some()
        {
            return Err(ApiError::forbidden(
                "manage MCP access tokens from a signed-in dashboard session",
            ));
        }
        let session = self.session(headers).await?;
        let account = self
            .service
            .list_linked_accounts(&session)
            .await
            .map_err(auth_internal)?
            .into_iter()
            .find(|account| account.provider_id == "discord");

        let Some(account) = account else {
            if self.development_bypass {
                return Ok(Actor {
                    discord_user_id: "local-development".to_owned(),
                    is_owner: true,
                    guild_scope: None,
                });
            }
            return Err(ApiError::unauthorized(
                "a linked Discord account is required",
            ));
        };
        let actor = Actor {
            is_owner: self.owners.contains(&account.account_id),
            discord_user_id: account.account_id.clone(),
            guild_scope: None,
        };
        self.refresh_discord_identity(&session, &account.id).await;
        if actor.is_owner {
            return Ok(actor);
        }

        let token = self
            .service
            .get_provider_access_token(&session, &account.id)
            .await
            .map_err(auth_internal)?;
        let response = self
            .http
            .get("https://discord.com/api/v10/users/@me/guilds")
            .bearer_auth(token.access_token)
            .send()
            .await
            .map_err(auth_internal)?;
        if !response.status().is_success() {
            tracing::warn!(status = %response.status(), "Discord guild authorization lookup failed");
            return Err(ApiError::unauthorized(
                "Discord authorization could not be verified",
            ));
        }
        let guilds: Vec<DiscordGuild> = response.json().await.map_err(auth_internal)?;
        let permitted = guilds.into_iter().any(|guild| {
            guild.id == guild_id
                && guild
                    .permissions
                    .parse::<u64>()
                    .is_ok_and(|permissions| permissions & (MANAGE_GUILD | ADMINISTRATOR) != 0)
        });
        if permitted {
            Ok(actor)
        } else {
            Err(ApiError::forbidden("Manage Server permission is required"))
        }
    }

    pub async fn require_guild_member(
        &self,
        headers: &HeaderMap,
        guild_id: &str,
    ) -> Result<Actor, ApiError> {
        if let Some(actor) = self.operator_actor(headers) {
            return Ok(actor);
        }
        if let Some(actor) = self.access_token_actor(headers).await? {
            return if actor.guild_scope.as_deref() == Some(guild_id) {
                Ok(actor)
            } else {
                Err(ApiError::forbidden(
                    "this access token is scoped to a different server",
                ))
            };
        }
        let (actor, guilds) = self.guild_actor_and_guilds(headers).await?;
        if actor.is_owner || guilds.iter().any(|guild| guild.id == guild_id) {
            Ok(actor)
        } else {
            Err(ApiError::forbidden("server membership is required"))
        }
    }

    async fn guild_actor_and_guilds(
        &self,
        headers: &HeaderMap,
    ) -> Result<(Actor, Vec<DiscordGuild>), ApiError> {
        if let Some(actor) = self.operator_actor(headers) {
            return Ok((actor, Vec::new()));
        }
        let session = self.session(headers).await?;
        let account = self
            .service
            .list_linked_accounts(&session)
            .await
            .map_err(auth_internal)?
            .into_iter()
            .find(|account| account.provider_id == "discord");
        let Some(account) = account else {
            if self.development_bypass {
                return Ok((
                    Actor {
                        discord_user_id: "local-development".to_owned(),
                        is_owner: true,
                        guild_scope: None,
                    },
                    vec![],
                ));
            }
            return Err(ApiError::unauthorized(
                "a linked Discord account is required",
            ));
        };
        let actor = Actor {
            is_owner: self.owners.contains(&account.account_id),
            discord_user_id: account.account_id.clone(),
            guild_scope: None,
        };
        self.refresh_discord_identity(&session, &account.id).await;
        if actor.is_owner {
            return Ok((actor, vec![]));
        }
        let token = self
            .service
            .get_provider_access_token(&session, &account.id)
            .await
            .map_err(auth_internal)?;
        let response = self
            .http
            .get("https://discord.com/api/v10/users/@me/guilds")
            .bearer_auth(token.access_token)
            .send()
            .await
            .map_err(auth_internal)?;
        if !response.status().is_success() {
            tracing::warn!(status = %response.status(), "Discord guild authorization lookup failed");
            return Err(ApiError::unauthorized(
                "Discord authorization could not be verified",
            ));
        }
        let guilds = response.json().await.map_err(auth_internal)?;
        Ok((actor, guilds))
    }

    fn operator_actor(&self, headers: &HeaderMap) -> Option<Actor> {
        let expected = self.operator_token.as_deref()?;
        if !bearer_matches(headers, expected) {
            return None;
        }
        Some(Actor {
            discord_user_id: "orivane-operator".to_owned(),
            is_owner: true,
            guild_scope: None,
        })
    }

    async fn access_token_actor(&self, headers: &HeaderMap) -> Result<Option<Actor>, ApiError> {
        let Some(token) = bearer_token(headers).filter(|token| token.starts_with("orv_")) else {
            return Ok(None);
        };
        let token_hash = hex_sha256(token.as_bytes());
        let row = sqlx08::query_as::<_, (i64, String, String)>(
            "UPDATE mcp_access_tokens SET last_used_at = now() \
             WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now() \
             RETURNING id, discord_user_id, guild_id",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(auth_internal)?;
        Ok(row.map(|(_, discord_user_id, guild_id)| Actor {
            discord_user_id,
            is_owner: false,
            guild_scope: Some(guild_id),
        }))
    }

    async fn session(&self, headers: &HeaderMap) -> Result<SessionWithUser, ApiError> {
        if let Some(token) = lucid_auth::axum::session_token(&self.service, headers) {
            return self
                .service
                .session(&token)
                .await
                .map_err(auth_internal)?
                .ok_or_else(|| ApiError::unauthorized("session is invalid or expired"));
        }
        self.service
            .development_session()
            .ok_or_else(|| ApiError::unauthorized("sign in with Discord"))
    }

    async fn refresh_discord_identity(&self, session: &SessionWithUser, account_id: &str) {
        if let Err(error) = self.try_refresh_discord_identity(session, account_id).await {
            tracing::warn!(%error, "could not refresh Discord username and display name");
        }
    }

    async fn try_refresh_discord_identity(
        &self,
        session: &SessionWithUser,
        account_id: &str,
    ) -> Result<(), anyhow::Error> {
        let identity_is_fresh: bool = sqlx08::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM discord_identities d \
             JOIN account a ON a.\"accountId\" = d.user_id \
             WHERE a.id = $1 AND d.updated_at >= now() - interval '15 minutes')",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .context("check Discord identity freshness")?;
        if identity_is_fresh {
            return Ok(());
        }
        let token = self
            .service
            .get_provider_access_token(session, account_id)
            .await
            .context("get Discord access token")?;
        let response = self
            .http
            .get("https://discord.com/api/v10/users/@me")
            .bearer_auth(token.access_token)
            .send()
            .await
            .context("fetch Discord profile")?
            .error_for_status()
            .context("Discord profile response")?;
        let profile: DiscordProfile = response.json().await.context("decode Discord profile")?;
        let avatar_url = profile.avatar.as_ref().map(|hash| {
            let extension = if hash.starts_with("a_") { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}?size=128",
                profile.id, hash, extension
            )
        });
        sqlx08::query(
            "INSERT INTO discord_identities (user_id, username, display_name, avatar_url) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (user_id) DO UPDATE SET username = EXCLUDED.username, \
             display_name = EXCLUDED.display_name, avatar_url = EXCLUDED.avatar_url, updated_at = now()",
        )
        .bind(&profile.id)
        .bind(&profile.username)
        .bind(&profile.global_name)
        .bind(avatar_url)
        .execute(&self.pool)
        .await
        .context("store Discord identity")?;
        Ok(())
    }
}

fn bearer_matches(headers: &HeaderMap, expected: &str) -> bool {
    let Some(provided) = bearer_token(headers) else {
        return false;
    };
    provided.len() == expected.len()
        && openssl::memcmp::eq(provided.as_bytes(), expected.as_bytes())
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

pub(crate) fn hex_sha256(value: &[u8]) -> String {
    openssl::sha::sha256(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn auth_internal(error: impl std::fmt::Display) -> ApiError {
    tracing::error!(%error, "authentication request failed");
    ApiError::internal()
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn env_flag(name: &str) -> bool {
    env::var(name).is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes"))
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};

    use super::{bearer_matches, hex_sha256};

    #[test]
    fn service_bearer_requires_an_exact_case_sensitive_match() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer correct-token"),
        );
        assert!(bearer_matches(&headers, "correct-token"));
        assert!(!bearer_matches(&headers, "wrong-token"));
        assert!(!bearer_matches(&headers, "CORRECT-TOKEN"));
    }

    #[test]
    fn service_bearer_rejects_missing_or_wrong_schemes() {
        let mut headers = HeaderMap::new();
        assert!(!bearer_matches(&headers, "correct-token"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Basic correct-token"),
        );
        assert!(!bearer_matches(&headers, "correct-token"));
    }

    #[test]
    fn access_tokens_are_hashed_deterministically_without_storing_plaintext() {
        assert_eq!(
            hex_sha256(b"orv_example"),
            "5433a8aba87c4d78b50cdf045e3774b2bdb6b244eed56609260341ac464040a4"
        );
    }
}
