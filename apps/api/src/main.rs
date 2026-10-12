use std::{env, net::SocketAddr};

mod auth;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use bot_core::{
    AuditEvent, CommandVolumeBucket, ConnectedGuild, FormSubmission, GlobalUser, GuildConfig,
    GuildDashboardStats, GuildMemberSummary, Infrastructure, InfrastructureHealth, ModerationCase,
    NewAuditEvent, NewModerationCase, UserGuildContext,
};
use bot_discord::{CommandDefinition, DiscordApi, GuildChannel, GuildRole, TwilightDiscord};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::auth::{Actor, AuthState};

#[derive(Clone)]
struct AppState {
    infrastructure: Infrastructure,
    auth: AuthState,
    discord_token: Option<String>,
}

#[derive(Debug)]
struct ApiError(StatusCode, String);

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct CommandSyncResponse {
    synchronized: usize,
}

#[derive(Serialize)]
struct GlobalUserDetail {
    user: GlobalUser,
    contexts: Vec<UserGuildContext>,
}

#[derive(Deserialize)]
struct AuditParams {
    limit: Option<i64>,
    before_id: Option<i64>,
}

#[derive(Deserialize)]
struct DashboardActivity {
    action: String,
    guild_id: Option<String>,
    path: Option<String>,
    export: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    init_tracing();
    let database_url = required_env("DATABASE_URL")?;
    let redis_url = required_env("REDIS_URL")?;
    let bind: SocketAddr = env::var("API_BIND")
        .unwrap_or_else(|_| "127.0.0.1:3001".to_owned())
        .parse()
        .context("API_BIND must be a socket address")?;
    let frontend_origin =
        env::var("API_CORS_ORIGIN").unwrap_or_else(|_| "http://127.0.0.1:5173".to_owned());
    let cors_origin: HeaderValue = frontend_origin
        .parse()
        .context("API_CORS_ORIGIN must be a valid HTTP header value")?;
    let state = AppState {
        infrastructure: Infrastructure::connect(&database_url, &redis_url).await?,
        auth: AuthState::new(&database_url, &frontend_origin).await?,
        discord_token: env::var("DISCORD_TOKEN").ok(),
    };
    let auth_router = lucid_auth::axum::router(state.auth.service.clone());
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/audit", get(get_global_audit))
        .route("/v1/health", get(get_health_report))
        .route("/v1/servers", get(get_servers))
        .route("/v1/users", get(get_users))
        .route("/v1/users/{user_id}", get(get_user))
        .route("/v1/analytics/commands", get(get_global_command_volume))
        .route("/v1/audit/{event_id}", get(get_global_audit_event))
        .route("/v1/commands/sync", axum::routing::post(sync_commands))
        .route(
            "/v1/activity",
            axum::routing::post(record_dashboard_activity),
        )
        .route("/v1/guilds/{guild_id}/audit", get(get_guild_audit))
        .route(
            "/v1/guilds/{guild_id}/audit/{event_id}",
            get(get_guild_audit_event),
        )
        .route(
            "/v1/guilds/{guild_id}/config",
            get(get_config).put(put_config),
        )
        .route("/v1/guilds/{guild_id}/channels", get(get_guild_channels))
        .route("/v1/guilds/{guild_id}/roles", get(get_guild_roles))
        .route("/v1/guilds/{guild_id}/stats", get(get_guild_stats))
        .route("/v1/guilds/{guild_id}/members", get(get_guild_members))
        .route(
            "/v1/guilds/{guild_id}/analytics/commands",
            get(get_guild_command_volume),
        )
        .route(
            "/v1/guilds/{guild_id}/cases",
            get(get_cases).post(create_case),
        )
        .route("/v1/guilds/{guild_id}/cases/{case_id}", get(get_case))
        .route(
            "/v1/guilds/{guild_id}/forms/{form_id}/submissions",
            get(get_form_submissions),
        )
        .route(
            "/v1/guilds/{guild_id}/forms/{form_id}/submissions/{submission_id}",
            get(get_form_submission),
        )
        .merge(auth_router)
        .layer(
            CorsLayer::new()
                .allow_origin(cors_origin)
                .allow_credentials(true)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([CONTENT_TYPE, HeaderName::from_static("x-request-id")]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "API listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Response {
    match state.infrastructure.health().await {
        Ok(()) => (StatusCode::OK, Json(HealthResponse { status: "ok" })).into_response(),
        Err(error) => {
            tracing::error!(%error, "health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                }),
            )
                .into_response()
        }
    }
}

async fn get_servers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConnectedGuild>>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let servers = state
        .infrastructure
        .connected_guilds()
        .await
        .map_err(internal_error)?;
    record_read(&state, None, &actor, &headers, "fleet.servers.viewed").await;
    Ok(Json(servers))
}

async fn get_health_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<InfrastructureHealth>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let report = state.infrastructure.health_report().await;
    record_read(&state, None, &actor, &headers, "system.health.viewed").await;
    Ok(Json(report))
}

async fn get_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<GlobalUser>>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let users = state
        .infrastructure
        .global_users()
        .await
        .map_err(internal_error)?;
    record_read(&state, None, &actor, &headers, "fleet.users.viewed").await;
    Ok(Json(users))
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<GlobalUserDetail>, ApiError> {
    validate_snowflake(&user_id)?;
    let actor = state.auth.require_owner(&headers).await?;
    let user = state
        .infrastructure
        .global_users()
        .await
        .map_err(internal_error)?
        .into_iter()
        .find(|user| user.id == user_id)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "global user not found".to_owned()))?;
    let contexts = state
        .infrastructure
        .user_guild_contexts(&user_id)
        .await
        .map_err(internal_error)?;
    record_read(&state, None, &actor, &headers, "fleet.user.viewed").await;
    Ok(Json(GlobalUserDetail { user, contexts }))
}

async fn get_global_command_volume(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CommandVolumeBucket>>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let volume = state
        .infrastructure
        .command_volume(None)
        .await
        .map_err(internal_error)?;
    record_read(&state, None, &actor, &headers, "analytics.commands.viewed").await;
    Ok(Json(volume))
}

async fn get_guild_stats(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<GuildDashboardStats>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let stats = state
        .infrastructure
        .guild_dashboard_stats(&guild_id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "analytics.summary.viewed",
    )
    .await;
    Ok(Json(stats))
}

async fn get_guild_members(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<GuildMemberSummary>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let members = state
        .infrastructure
        .guild_members(&guild_id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "community.members.viewed",
    )
    .await;
    Ok(Json(members))
}

async fn get_guild_command_volume(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<CommandVolumeBucket>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let volume = state
        .infrastructure
        .command_volume(Some(&guild_id))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "analytics.commands.viewed",
    )
    .await;
    Ok(Json(volume))
}

async fn sync_commands(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CommandSyncResponse>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let token = state.discord_token.as_ref().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "DISCORD_TOKEN is not configured".to_owned(),
        )
    })?;
    let commands: Vec<CommandDefinition> = bot_module_utility::COMMANDS.to_vec();
    let result = async {
        let discord = TwilightDiscord::connect(token.clone()).await?;
        discord.register_commands(&commands).await
    }
    .await;
    let mut event = NewAuditEvent::global(&actor.discord_user_id, "commands.synchronized");
    event.source = "dashboard_api".to_owned();
    event.target_type = "slash_command_registry".to_owned();
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    event.metadata = serde_json::json!({ "command_count": commands.len() });
    if let Err(error) = result {
        event.outcome = "failure".to_owned();
        event.reason = Some(error.to_string());
        state
            .infrastructure
            .record_audit_event(&event)
            .await
            .map_err(internal_error)?;
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "Discord command synchronization failed".to_owned(),
        ));
    }
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(Json(CommandSyncResponse {
        synchronized: commands.len(),
    }))
}

async fn get_config(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<GuildConfig>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let config = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "guild.config.viewed",
    )
    .await;
    Ok(Json(config))
}

async fn get_guild_channels(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<GuildChannel>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let token = state.discord_token.as_ref().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "DISCORD_TOKEN is not configured".to_owned(),
        )
    })?;
    let discord = TwilightDiscord::connect(token.clone())
        .await
        .map_err(|error| {
            tracing::error!(%guild_id, %error, "failed to initialize Discord channel lookup");
            ApiError(
                StatusCode::BAD_GATEWAY,
                "Discord channel lookup failed".to_owned(),
            )
        })?;
    let channels = discord
        .guild_channels(guild_id.parse().expect("validated Discord snowflake"))
        .await
        .map_err(|error| {
            tracing::error!(%guild_id, %error, "failed to fetch Discord guild channels");
            ApiError(
                StatusCode::BAD_GATEWAY,
                "Discord channel lookup failed".to_owned(),
            )
        })?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "guild.channels.viewed",
    )
    .await;
    Ok(Json(channels))
}

async fn get_guild_roles(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<GuildRole>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let token = state.discord_token.as_ref().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "DISCORD_TOKEN is not configured".to_owned(),
        )
    })?;
    let discord = TwilightDiscord::connect(token.clone())
        .await
        .map_err(|error| {
            tracing::error!(%guild_id, %error, "failed to initialize Discord role lookup");
            ApiError(
                StatusCode::BAD_GATEWAY,
                "Discord role lookup failed".to_owned(),
            )
        })?;
    let mut roles = discord
        .guild_roles(guild_id.parse().expect("validated Discord snowflake"))
        .await
        .map_err(|error| {
            tracing::error!(%guild_id, %error, "failed to fetch Discord guild roles");
            ApiError(
                StatusCode::BAD_GATEWAY,
                "Discord role lookup failed".to_owned(),
            )
        })?;
    roles.retain(|role| !role.managed && role.id != guild_id);
    roles.sort_by(|left, right| right.position.cmp(&left.position));
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "guild.roles.viewed",
    )
    .await;
    Ok(Json(roles))
}

async fn put_config(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Json(mut config): Json<GuildConfig>,
) -> Result<Json<GuildConfig>, ApiError> {
    let request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    let actor = match state.auth.require_guild(&headers, &guild_id).await {
        Ok(actor) => actor,
        Err(error) => {
            record_failed_config_change(
                &state,
                &guild_id,
                "anonymous",
                request_id,
                "denied",
                &error.1,
                None,
            )
            .await;
            return Err(error);
        }
    };
    if let Err(error) = validate_snowflake(&guild_id) {
        record_failed_config_change(
            &state,
            &guild_id,
            &actor.discord_user_id,
            request_id,
            "failure",
            &error.1,
            None,
        )
        .await;
        return Err(error);
    }
    config.guild_id = guild_id.clone();
    if let Err(error) = config.validate() {
        let message = error.to_string();
        record_failed_config_change(
            &state,
            &guild_id,
            &actor.discord_user_id,
            request_id,
            "failure",
            &message,
            serde_json::to_value(&config).ok(),
        )
        .await;
        return Err(ApiError(StatusCode::UNPROCESSABLE_ENTITY, message));
    }
    let before = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    state
        .infrastructure
        .attribute_guild_installer(&guild_id, &actor.discord_user_id)
        .await
        .map_err(internal_error)?;
    let mut event = NewAuditEvent::guild(&guild_id, actor.discord_user_id, "guild.config.updated");
    event.actor_kind = "user".to_owned();
    event.target_type = "guild_config".to_owned();
    event.target_id = Some(guild_id);
    event.source = "dashboard_api".to_owned();
    event.request_id = request_id;
    event.before_state = serde_json::to_value(before).ok();
    event.after_state = serde_json::to_value(&config).ok();
    state
        .infrastructure
        .put_guild_config_audited(&config, &event)
        .await
        .map_err(internal_error)?;
    Ok(Json(config))
}

async fn get_global_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<AuditEvent>>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let events = state
        .infrastructure
        .audit_events(None, params.limit.unwrap_or(100), params.before_id)
        .await
        .map_err(internal_error)?;
    record_read(&state, None, &actor, &headers, "audit.global.viewed").await;
    Ok(Json(events))
}

async fn get_guild_audit(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<AuditEvent>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let events = state
        .infrastructure
        .audit_events(
            Some(&guild_id),
            params.limit.unwrap_or(100),
            params.before_id,
        )
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "audit.guild.viewed",
    )
    .await;
    Ok(Json(events))
}

async fn get_global_audit_event(
    State(state): State<AppState>,
    Path(event_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<AuditEvent>, ApiError> {
    let actor = state.auth.require_owner(&headers).await?;
    let event = state
        .infrastructure
        .audit_event(event_id, None)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "audit event not found".to_owned()))?;
    record_read(
        &state,
        event.guild_id.as_deref(),
        &actor,
        &headers,
        "audit.event.viewed",
    )
    .await;
    Ok(Json(event))
}

async fn get_guild_audit_event(
    State(state): State<AppState>,
    Path((guild_id, event_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<AuditEvent>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let event = state
        .infrastructure
        .audit_event(event_id, Some(&guild_id))
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "audit event not found".to_owned()))?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "audit.event.viewed",
    )
    .await;
    Ok(Json(event))
}

async fn record_dashboard_activity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<DashboardActivity>,
) -> Result<StatusCode, ApiError> {
    if !matches!(
        input.action.as_str(),
        "dashboard.page.viewed" | "dashboard.data.exported"
    ) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "unsupported dashboard activity".to_owned(),
        ));
    }
    if input.path.as_ref().is_some_and(|path| path.len() > 500)
        || input.export.as_ref().is_some_and(|name| name.len() > 100)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "activity metadata is too long".to_owned(),
        ));
    }
    let actor = match input.guild_id.as_deref() {
        Some(guild_id) => {
            validate_snowflake(guild_id)?;
            state.auth.require_guild(&headers, guild_id).await?
        }
        None => state.auth.require_owner(&headers).await?,
    };
    let mut event = match input.guild_id.as_deref() {
        Some(guild_id) => NewAuditEvent::guild(guild_id, &actor.discord_user_id, &input.action),
        None => NewAuditEvent::global(&actor.discord_user_id, &input.action),
    };
    event.source = "dashboard_web".to_owned();
    event.target_type = "dashboard_page".to_owned();
    event.target_id = input.path.clone();
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    event.metadata = serde_json::json!({ "path": input.path, "export": input.export });
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_cases(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<ModerationCase>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let cases = state
        .infrastructure
        .moderation_cases(&guild_id, 200)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "moderation.cases.viewed",
    )
    .await;
    Ok(Json(cases))
}

async fn get_case(
    State(state): State<AppState>,
    Path((guild_id, case_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<ModerationCase>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let case = state
        .infrastructure
        .moderation_case(&guild_id, case_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            ApiError(
                StatusCode::NOT_FOUND,
                "moderation case not found".to_owned(),
            )
        })?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "moderation.case.viewed",
    )
    .await;
    Ok(Json(case))
}

async fn get_form_submissions(
    State(state): State<AppState>,
    Path((guild_id, form_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Vec<FormSubmission>>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_form_id(&form_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let submissions = state
        .infrastructure
        .form_submissions(&guild_id, &form_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "forms.submissions.viewed",
    )
    .await;
    Ok(Json(submissions))
}

async fn get_form_submission(
    State(state): State<AppState>,
    Path((guild_id, form_id, submission_id)): Path<(String, String, i64)>,
    headers: HeaderMap,
) -> Result<Json<FormSubmission>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_form_id(&form_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let submission = state
        .infrastructure
        .form_submission(&guild_id, &form_id, submission_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            ApiError(
                StatusCode::NOT_FOUND,
                "form submission not found".to_owned(),
            )
        })?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "forms.submission.viewed",
    )
    .await;
    Ok(Json(submission))
}

async fn create_case(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<NewModerationCase>,
) -> Result<(StatusCode, Json<ModerationCase>), ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    input
        .validate()
        .map_err(|message| ApiError(StatusCode::UNPROCESSABLE_ENTITY, message.to_owned()))?;
    let mut event =
        NewAuditEvent::guild(&guild_id, &actor.discord_user_id, "moderation.case.created");
    event.source = "dashboard_api".to_owned();
    event.target_type = "member".to_owned();
    event.target_id = Some(input.member_id.clone());
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    event.metadata = serde_json::json!({ "action": input.action });
    let case = state
        .infrastructure
        .create_moderation_case_audited(&guild_id, &actor.discord_user_id, &input, &event)
        .await
        .map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(case)))
}

async fn record_read(
    state: &AppState,
    guild_id: Option<&str>,
    actor: &Actor,
    headers: &HeaderMap,
    action: &str,
) {
    let mut event = match guild_id {
        Some(guild_id) => NewAuditEvent::guild(guild_id, &actor.discord_user_id, action),
        None => NewAuditEvent::global(&actor.discord_user_id, action),
    };
    event.source = "dashboard_api".to_owned();
    event.target_type = if action.starts_with("guild.config") {
        "guild_config"
    } else if action.starts_with("moderation") {
        "moderation_case"
    } else if action.starts_with("forms.") {
        "form_submission"
    } else if action.starts_with("fleet") {
        "guild_fleet"
    } else if action.starts_with("analytics") {
        "analytics_report"
    } else {
        "audit_log"
    }
    .to_owned();
    event.request_id = header_value(headers, "x-request-id").map(str::to_owned);
    if let Err(error) = state.infrastructure.record_audit_event(&event).await {
        tracing::error!(%error, action, "failed to persist audit read event");
    }
}

async fn record_failed_config_change(
    state: &AppState,
    guild_id: &str,
    actor_id: &str,
    request_id: Option<String>,
    outcome: &str,
    reason: &str,
    after_state: Option<serde_json::Value>,
) {
    let mut event = NewAuditEvent::guild(guild_id, actor_id, "guild.config.updated");
    event.actor_kind = "api".to_owned();
    event.target_type = "guild_config".to_owned();
    event.target_id = Some(guild_id.to_owned());
    event.outcome = outcome.to_owned();
    event.source = "dashboard_api".to_owned();
    event.request_id = request_id;
    event.reason = Some(reason.to_owned());
    event.after_state = after_state;
    if let Err(error) = state.infrastructure.record_audit_event(&event).await {
        tracing::error!(%error, "failed to persist rejected configuration audit event");
    }
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn validate_snowflake(value: &str) -> Result<(), ApiError> {
    if value.parse::<u64>().is_ok_and(|id| id > 0) {
        Ok(())
    } else {
        Err(ApiError(
            StatusCode::BAD_REQUEST,
            "invalid guild ID".to_owned(),
        ))
    }
}

fn validate_form_id(value: &str) -> Result<(), ApiError> {
    if !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        Ok(())
    } else {
        Err(ApiError(
            StatusCode::BAD_REQUEST,
            "invalid form ID".to_owned(),
        ))
    }
}

fn internal_error(error: impl std::fmt::Display) -> ApiError {
    tracing::error!(%error, "request failed");
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error".to_owned(),
    )
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.1 }));
        (self.0, body).into_response()
    }
}

impl ApiError {
    fn unauthorized(message: impl Into<String>) -> Self {
        Self(StatusCode::UNAUTHORIZED, message.into())
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self(StatusCode::FORBIDDEN, message.into())
    }

    fn internal() -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error".to_owned(),
        )
    }
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
