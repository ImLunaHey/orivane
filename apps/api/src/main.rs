use std::{
    collections::HashMap,
    env,
    hash::{Hash, Hasher},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

mod auth;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::{
        HeaderMap, HeaderName, HeaderValue, Method, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use bot_core::{
    AfkStatus, AuditEvent, CommandVolumeBucket, ConnectedGuild, CounterRun, EconomyAccount,
    EconomyTransaction, EmbedDelivery, EmbedPayload, EmbedTemplate, FormSubmission, Giveaway,
    GiveawayEntry, GlobalUser, GuildConfig, GuildDashboardStats, GuildMemberSummary,
    Infrastructure, InfrastructureHealth, InviteAttribution, InviteLeaderboardEntry,
    MessageDeletionJob, ModerationCase, NewAuditEvent, NewEmbedDelivery, NewEmbedTemplate,
    NewGiveaway, NewModerationCase, NewTicketPanel, Poll, Reminder, Suggestion,
    TemporaryVoiceChannel, Ticket, TicketAnalytics, TicketEvent, TicketNote, TicketPanel,
    UserGuildContext, VerificationAttempt, VoiceLinkEvent,
};
use bot_discord::{
    CommandDefinition, DiscordApi, GuildChannel, GuildRole, MessageButton, MessageButtonStyle,
    RolePanelComponents, RolePanelOption, TwilightDiscord,
};
use bot_observability::ServiceMetrics;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::auth::{Actor, AuthState};

#[derive(Clone)]
struct AppState {
    infrastructure: Infrastructure,
    auth: AuthState,
    discord_token: Option<String>,
    metrics: ServiceMetrics,
}

#[derive(Clone)]
struct MiddlewareState {
    metrics: ServiceMetrics,
    limiter: RateLimiter,
    frontend_origin: Arc<str>,
}

#[derive(Clone, Default)]
struct RateLimiter {
    windows: Arc<Mutex<HashMap<u64, RateWindow>>>,
}

struct RateWindow {
    started: Instant,
    reads: u32,
    mutations: u32,
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

#[derive(Serialize)]
struct ResourceIssue {
    path: String,
    kind: &'static str,
    id: Option<String>,
    message: String,
}

#[derive(Serialize)]
struct ResourceDiagnostics {
    status: &'static str,
    channels_available: bool,
    roles_available: bool,
    issues: Vec<ResourceIssue>,
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

#[derive(Deserialize)]
struct UpdateSuggestionStatus {
    status: String,
    staff_note: Option<String>,
}

#[derive(Deserialize)]
struct CreateGiveawayRequest {
    channel_id: String,
    title: String,
    #[serde(default)]
    description: String,
    winner_count: u8,
    minimum_account_age_days: u16,
    #[serde(default)]
    required_role_ids: Vec<String>,
    bonus_role_id: Option<String>,
    bonus_entries: u8,
    ends_at_unix: u64,
}

#[derive(Deserialize)]
struct SaveEmbedTemplateRequest {
    name: String,
    payload: EmbedPayload,
}

#[derive(Deserialize)]
struct DeliverEmbedRequest {
    channel_id: String,
}

#[derive(Deserialize)]
struct SaveTicketPanelRequest {
    name: String,
    title: String,
    #[serde(default)]
    description: String,
    channel_id: String,
    category_id: String,
    #[serde(default)]
    support_role_ids: Vec<String>,
    button_label: String,
    welcome_message: String,
    channel_name: String,
    intake_enabled: bool,
    intake_label: String,
    max_open_per_user: i16,
    #[serde(default)]
    auto_close_hours: i32,
    #[serde(default = "default_ticket_retention")]
    transcript_retention_days: i32,
    #[serde(default = "default_ticket_priority")]
    default_priority: String,
    enabled: bool,
}
fn default_ticket_retention() -> i32 {
    90
}
fn default_ticket_priority() -> String {
    "normal".into()
}
#[derive(Deserialize)]
struct TicketReasonRequest {
    #[serde(default)]
    reason: String,
}
#[derive(Deserialize)]
struct TicketTransferRequest {
    user_id: String,
}
#[derive(Deserialize)]
struct UpdateTicketMetadataRequest {
    priority: String,
    #[serde(default)]
    tags: Vec<String>,
}
#[derive(Deserialize)]
struct CreateTicketNoteRequest {
    body: String,
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
    let metrics = ServiceMetrics::new("api");
    let middleware = MiddlewareState {
        metrics: metrics.clone(),
        limiter: RateLimiter::default(),
        frontend_origin: Arc::from(frontend_origin.as_str()),
    };
    let state = AppState {
        infrastructure: Infrastructure::connect(&database_url, &redis_url).await?,
        auth: AuthState::new(&database_url, &frontend_origin).await?,
        discord_token: env::var("DISCORD_TOKEN").ok(),
        metrics: metrics.clone(),
    };
    let auth_router = lucid_auth::axum::router(state.auth.service.clone());
    let app = Router::new()
        .route("/health", get(health))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(health))
        .route("/metrics", get(prometheus_metrics))
        .route("/v1/audit", get(get_global_audit))
        .route("/v1/health", get(get_health_report))
        .route("/v1/servers", get(get_servers))
        .route("/v1/users", get(get_users))
        .route("/v1/users/{user_id}", get(get_user))
        .route("/v1/me/privacy-export", get(get_privacy_export))
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
        .route(
            "/v1/guilds/{guild_id}/resources/diagnostics",
            get(get_resource_diagnostics),
        )
        .route("/v1/guilds/{guild_id}/appeals", get(get_appeals))
        .route(
            "/v1/guilds/{guild_id}/appeals/{appeal_id}",
            get(get_appeal).patch(review_appeal),
        )
        .route("/v1/guilds/{guild_id}/polls", get(get_polls))
        .route("/v1/guilds/{guild_id}/polls/{poll_id}", get(get_poll))
        .route(
            "/v1/guilds/{guild_id}/polls/{poll_id}/close",
            axum::routing::post(close_poll_dashboard),
        )
        .route(
            "/v1/guilds/{guild_id}/invites/attributions",
            get(get_invite_attributions),
        )
        .route(
            "/v1/guilds/{guild_id}/invites/leaderboard",
            get(get_invite_leaderboard),
        )
        .route(
            "/v1/guilds/{guild_id}/verification/attempts",
            get(get_verification_attempts),
        )
        .route("/v1/guilds/{guild_id}/afk", get(get_afk_statuses))
        .route(
            "/v1/guilds/{guild_id}/afk/{user_id}",
            axum::routing::delete(clear_afk_status_dashboard),
        )
        .route("/v1/guilds/{guild_id}/counters/runs", get(get_counter_runs))
        .route(
            "/v1/guilds/{guild_id}/voice-links/events",
            get(get_voice_link_events),
        )
        .route(
            "/v1/guilds/{guild_id}/temporary-voice",
            get(get_temporary_voice_channels),
        )
        .route(
            "/v1/guilds/{guild_id}/temporary-voice/{channel_id}",
            axum::routing::delete(delete_temporary_voice_channel),
        )
        .route(
            "/v1/guilds/{guild_id}/economy/accounts",
            get(get_economy_accounts),
        )
        .route(
            "/v1/guilds/{guild_id}/economy/transactions",
            get(get_economy_transactions),
        )
        .route(
            "/v1/guilds/{guild_id}/ticket-panels",
            get(get_ticket_panels).post(create_ticket_panel),
        )
        .route(
            "/v1/guilds/{guild_id}/ticket-panels/{panel_id}",
            get(get_ticket_panel)
                .put(update_ticket_panel)
                .delete(delete_ticket_panel),
        )
        .route(
            "/v1/guilds/{guild_id}/ticket-panels/{panel_id}/deploy",
            axum::routing::post(deploy_ticket_panel),
        )
        .route("/v1/guilds/{guild_id}/tickets", get(get_tickets))
        .route(
            "/v1/guilds/{guild_id}/tickets/analytics",
            get(get_ticket_analytics),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}",
            get(get_ticket).patch(update_ticket_metadata),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/events",
            get(get_ticket_events),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/claim",
            axum::routing::post(claim_ticket),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/close",
            axum::routing::post(close_ticket),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/reopen",
            axum::routing::post(reopen_ticket),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/transfer",
            axum::routing::post(transfer_ticket),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/notes",
            get(get_ticket_notes).post(create_ticket_note),
        )
        .route(
            "/v1/guilds/{guild_id}/tickets/{ticket_id}/transcript",
            get(download_ticket_transcript),
        )
        .route(
            "/v1/guilds/{guild_id}/reaction-role-panels/{panel_id}/deploy",
            axum::routing::post(deploy_reaction_role_panel),
        )
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
        .route("/v1/guilds/{guild_id}/suggestions", get(get_suggestions))
        .route(
            "/v1/guilds/{guild_id}/suggestions/{suggestion_id}",
            get(get_suggestion).patch(update_suggestion_status),
        )
        .route(
            "/v1/guilds/{guild_id}/giveaways",
            get(get_giveaways).post(create_giveaway),
        )
        .route(
            "/v1/guilds/{guild_id}/giveaways/{giveaway_id}",
            get(get_giveaway),
        )
        .route(
            "/v1/guilds/{guild_id}/giveaways/{giveaway_id}/entries",
            get(get_giveaway_entries),
        )
        .route(
            "/v1/guilds/{guild_id}/giveaways/{giveaway_id}/end",
            axum::routing::post(end_giveaway),
        )
        .route(
            "/v1/guilds/{guild_id}/giveaways/{giveaway_id}/reroll",
            axum::routing::post(reroll_giveaway),
        )
        .route("/v1/guilds/{guild_id}/reminders", get(get_reminders))
        .route(
            "/v1/guilds/{guild_id}/reminders/{reminder_id}",
            get(get_reminder),
        )
        .route(
            "/v1/guilds/{guild_id}/reminders/{reminder_id}/cancel",
            axum::routing::post(cancel_reminder),
        )
        .route(
            "/v1/guilds/{guild_id}/message-deletions",
            get(get_message_deletions),
        )
        .route(
            "/v1/guilds/{guild_id}/message-deletions/{channel_id}/{message_id}",
            get(get_message_deletion),
        )
        .route(
            "/v1/guilds/{guild_id}/embeds",
            get(get_embed_templates).post(create_embed_template),
        )
        .route(
            "/v1/guilds/{guild_id}/embeds/{template_id}",
            get(get_embed_template)
                .put(update_embed_template)
                .delete(delete_embed_template),
        )
        .route(
            "/v1/guilds/{guild_id}/embed-deliveries",
            get(get_embed_deliveries),
        )
        .route(
            "/v1/guilds/{guild_id}/embeds/{template_id}/send",
            axum::routing::post(send_embed_template),
        )
        .route(
            "/v1/guilds/{guild_id}/embeds/{template_id}/deliveries/{delivery_id}/sync",
            axum::routing::post(sync_embed_delivery),
        )
        .route(
            "/v1/guilds/{guild_id}/forms/{form_id}/submissions",
            get(get_form_submissions),
        )
        .route(
            "/v1/guilds/{guild_id}/forms/{form_id}/submissions/{submission_id}",
            get(get_form_submission).patch(review_form_submission),
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
        .layer(from_fn_with_state(middleware, instrument_request))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    metrics.set_ready(true);
    tracing::info!(%bind, "API listening");
    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;
    metrics.set_ready(false);
    result?;
    Ok(())
}

async fn instrument_request(
    State(state): State<MiddlewareState>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(response) = reject_cross_site_mutation(&request, &state.frontend_origin) {
        return response;
    }
    if let Some(retry_after) = state.limiter.retry_after(&request) {
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Too many dashboard requests. Wait briefly and retry."
            })),
        )
            .into_response();
        response.headers_mut().insert(
            "retry-after",
            HeaderValue::from_str(&retry_after.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("60")),
        );
        return response;
    }
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .map(str::to_owned)
        .unwrap_or_else(|| state.metrics.request_id());
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        request.headers_mut().insert("x-request-id", value);
    }
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let started = Instant::now();
    let mut response = next.run(request).await;
    let status = response.status();
    state
        .metrics
        .record_request(status.is_server_error(), started.elapsed());
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert("x-frame-options", HeaderValue::from_static("DENY"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response.headers_mut().insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    tracing::info!(
        %request_id,
        %method,
        %path,
        status = status.as_u16(),
        latency_ms = started.elapsed().as_millis(),
        "request completed"
    );
    response
}

impl RateLimiter {
    fn retry_after(&self, request: &Request) -> Option<u64> {
        let path = request.uri().path();
        if !path.starts_with("/v1/") {
            return None;
        }
        let mutation = !matches!(*request.method(), Method::GET | Method::OPTIONS);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        request
            .headers()
            .get("cookie")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("anonymous")
            .hash(&mut hasher);
        if let Some(connect) = request
            .extensions()
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
        {
            connect.0.ip().hash(&mut hasher);
        }
        let key = hasher.finish();
        let now = Instant::now();
        let mut windows = self.windows.lock().expect("rate limiter lock poisoned");
        if windows.len() > 10_000 {
            windows
                .retain(|_, window| now.duration_since(window.started) < Duration::from_secs(60));
        }
        let window = windows.entry(key).or_insert(RateWindow {
            started: now,
            reads: 0,
            mutations: 0,
        });
        if now.duration_since(window.started) >= Duration::from_secs(60) {
            *window = RateWindow {
                started: now,
                reads: 0,
                mutations: 0,
            };
        }
        let counter = if mutation {
            &mut window.mutations
        } else {
            &mut window.reads
        };
        let limit = if mutation { 120 } else { 600 };
        if *counter >= limit {
            return Some(
                60_u64
                    .saturating_sub(now.duration_since(window.started).as_secs())
                    .max(1),
            );
        }
        *counter += 1;
        None
    }
}

fn reject_cross_site_mutation(request: &Request, frontend_origin: &str) -> Option<Response> {
    if matches!(*request.method(), Method::GET | Method::OPTIONS)
        || !request.uri().path().starts_with("/v1/")
    {
        return None;
    }
    let cross_site = request
        .headers()
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        == Some("cross-site");
    let wrong_origin = request
        .headers()
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| origin != frontend_origin);
    (cross_site || wrong_origin).then(|| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Cross-site dashboard mutation rejected" })),
        )
            .into_response()
    })
}

async fn liveness() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

async fn prometheus_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let mut output = state.metrics.render_prometheus();
    if let Ok(runtime) = state.infrastructure.runtime_metrics().await {
        output.push_str(&format!(
            concat!(
                "# HELP orivane_database_pool_connections Database connection pool size.\n",
                "# TYPE orivane_database_pool_connections gauge\n",
                "orivane_database_pool_connections{{state=\"total\"}} {}\n",
                "orivane_database_pool_connections{{state=\"idle\"}} {}\n",
                "# HELP orivane_due_job_backlog Jobs whose delivery time has passed.\n",
                "# TYPE orivane_due_job_backlog gauge\n",
                "orivane_due_job_backlog {}\n"
            ),
            runtime.database_pool_size, runtime.database_pool_idle, runtime.due_job_backlog,
        ));
    }
    (
        [(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        output,
    )
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

async fn get_privacy_export(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let actor = state.auth.actor(&headers).await?;
    let user = state
        .infrastructure
        .global_users()
        .await
        .map_err(internal_error)?
        .into_iter()
        .find(|user| user.id == actor.discord_user_id);
    let guild_contexts = if actor.discord_user_id.parse::<u64>().is_ok() {
        state
            .infrastructure
            .user_guild_contexts(&actor.discord_user_id)
            .await
            .map_err(internal_error)?
    } else {
        Vec::new()
    };
    record_read(
        &state,
        None,
        &actor,
        &headers,
        "privacy.personal_data.exported",
    )
    .await;
    Ok(Json(serde_json::json!({
        "schema_version": 1,
        "discord_user_id": actor.discord_user_id,
        "profile": user,
        "guild_contexts": guild_contexts,
        "explanation": {
            "profile": "Identity and fleet-wide aggregate data retained by Orivane.",
            "guild_contexts": "Per-server command, moderation, XP, and installation context visible to the signed-in Discord account.",
            "message_content": "Orivane audit logs do not retain deleted or edited message content."
        }
    })))
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
    let commands = all_commands();
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

fn all_commands() -> Vec<CommandDefinition> {
    let groups: &[&[CommandDefinition]] = &[
        bot_module_utility::COMMANDS,
        bot_module_moderation::COMMANDS,
        bot_module_leveling::COMMANDS,
        bot_module_tickets::COMMANDS,
        bot_module_custom_commands::COMMANDS,
        bot_module_autorole::COMMANDS,
        bot_module_anti_raid::COMMANDS,
        bot_module_forms::COMMANDS,
        bot_module_suggestions::COMMANDS,
        bot_module_giveaways::COMMANDS,
        bot_module_reminders::COMMANDS,
        bot_module_auto_delete::COMMANDS,
        bot_module_managed_embeds::COMMANDS,
        bot_module_temporary_voice::COMMANDS,
        bot_module_economy::COMMANDS,
        bot_module_verification::COMMANDS,
        bot_module_moderation_appeals::COMMANDS,
        bot_module_voice_text_linking::COMMANDS,
        bot_module_afk::COMMANDS,
        bot_module_polls::COMMANDS,
        bot_module_invite_tracking::COMMANDS,
        bot_module_counters::COMMANDS,
    ];
    groups
        .iter()
        .flat_map(|commands| commands.iter().cloned())
        .collect()
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
        .map_err(|error| discord_resource_error(&guild_id, "channels", error))?;
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
        .map_err(|error| discord_resource_error(&guild_id, "roles", error))?;
    roles.retain(|role| !role.managed && role.id != guild_id);
    roles.sort_by_key(|role| std::cmp::Reverse(role.position));
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

async fn get_resource_diagnostics(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ResourceDiagnostics>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let config = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    let token = state.discord_token.as_ref().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Discord resource checks are unavailable because the bot token is not configured"
                .to_owned(),
        )
    })?;
    let discord = TwilightDiscord::connect(token.clone())
        .await
        .map_err(internal_error)?;
    let channels_result = discord
        .guild_channels(guild_id.parse().expect("validated Discord snowflake"))
        .await;
    let roles_result = discord
        .guild_roles(guild_id.parse().expect("validated Discord snowflake"))
        .await;
    let channels_available = channels_result.is_ok();
    let roles_available = roles_result.is_ok();
    let channel_ids = channels_result
        .as_ref()
        .map(|items| {
            items
                .iter()
                .map(|item| item.id.clone())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();
    let role_ids = roles_result
        .as_ref()
        .map(|items| {
            items
                .iter()
                .map(|item| item.id.clone())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();
    let mut references = Vec::new();
    collect_resource_references(
        &serde_json::to_value(&config).map_err(internal_error)?,
        String::new(),
        &mut references,
    );
    let mut issues = Vec::new();
    if let Err(error) = channels_result {
        issues.push(ResourceIssue {
            path: "Discord channels".to_owned(),
            kind: "access",
            id: None,
            message: discord_resource_message("channels", &error.to_string()),
        });
    }
    if let Err(error) = roles_result {
        issues.push(ResourceIssue {
            path: "Discord roles".to_owned(),
            kind: "access",
            id: None,
            message: discord_resource_message("roles", &error.to_string()),
        });
    }
    for (path, kind, id) in references {
        let missing = match kind {
            "channel" => channels_available && !channel_ids.contains(id.as_str()),
            "role" => roles_available && !role_ids.contains(id.as_str()),
            _ => false,
        };
        if missing {
            issues.push(ResourceIssue {
                path,
                kind,
                id: Some(id),
                message: format!(
                    "This configured {kind} no longer exists or is no longer visible to Orivane. Select a replacement and save the module."
                ),
            });
        }
    }
    let status = if issues.is_empty() {
        "healthy"
    } else {
        "attention"
    };
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "guild.resources.diagnosed",
    )
    .await;
    Ok(Json(ResourceDiagnostics {
        status,
        channels_available,
        roles_available,
        issues,
    }))
}

fn collect_resource_references(
    value: &Value,
    path: String,
    output: &mut Vec<(String, &'static str, String)>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                let kind = if key.ends_with("channel_id") || key == "category_id" {
                    Some("channel")
                } else if key.ends_with("role_id") {
                    Some("role")
                } else {
                    None
                };
                let plural_kind = if key.ends_with("channel_ids") {
                    Some("channel")
                } else if key.ends_with("role_ids") {
                    Some("role")
                } else {
                    None
                };
                if let (Some(kind), Some(id)) = (kind, child.as_str())
                    && !id.is_empty()
                {
                    output.push((child_path.clone(), kind, id.to_owned()));
                } else if let (Some(kind), Some(ids)) = (plural_kind, child.as_array()) {
                    for (index, id) in ids.iter().filter_map(Value::as_str).enumerate() {
                        if !id.is_empty() {
                            output.push((format!("{child_path}[{index}]"), kind, id.to_owned()));
                        }
                    }
                }
                collect_resource_references(child, child_path, output);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_resource_references(child, format!("{path}[{index}]"), output);
            }
        }
        _ => {}
    }
}

fn discord_resource_error(
    guild_id: &str,
    resource: &'static str,
    error: impl std::fmt::Display,
) -> ApiError {
    let error = error.to_string();
    tracing::error!(%guild_id, %error, resource, "failed to fetch Discord guild resources");
    let status = if error.contains("429") {
        StatusCode::TOO_MANY_REQUESTS
    } else if error.contains("403") || error.contains("Missing Access") {
        StatusCode::FORBIDDEN
    } else if error.contains("404") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::BAD_GATEWAY
    };
    ApiError(status, discord_resource_message(resource, &error))
}

fn discord_resource_message(resource: &str, error: &str) -> String {
    if error.contains("429") {
        format!("Discord is rate-limiting {resource} checks. Wait a moment and retry.")
    } else if error.contains("403") || error.contains("Missing Access") {
        format!(
            "Orivane cannot read this server's {resource}. Confirm the bot is still installed and has View Channels and Manage Roles where required."
        )
    } else if error.contains("404") {
        "Discord reports that this server is unavailable to Orivane. The bot may have been removed."
            .to_owned()
    } else {
        format!("Discord's {resource} service could not be reached. Retry shortly.")
    }
}

async fn require_appeal_reviewer(
    state: &AppState,
    headers: &HeaderMap,
    guild_id: &str,
) -> Result<Actor, ApiError> {
    let config = state
        .infrastructure
        .guild_config(guild_id)
        .await
        .map_err(internal_error)?;
    let allowed_roles = &config.modules.moderation_appeals.reviewer_role_ids;
    if allowed_roles.is_empty() {
        return state.auth.require_guild(headers, guild_id).await;
    }

    let actor = state.auth.require_guild_member(headers, guild_id).await?;
    if actor.is_owner {
        return Ok(actor);
    }
    if let (Some(token), Ok(guild_id), Ok(user_id)) = (
        state.discord_token.clone(),
        guild_id.parse::<u64>(),
        actor.discord_user_id.parse::<u64>(),
    ) {
        match TwilightDiscord::connect(token).await {
            Ok(discord) => match discord.guild_member_role_ids(guild_id, user_id).await {
                Ok(member_roles)
                    if member_roles.iter().any(|role_id| {
                        allowed_roles
                            .iter()
                            .any(|allowed| allowed == &role_id.to_string())
                    }) =>
                {
                    return Ok(actor);
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(%guild_id, %user_id, %error, "could not resolve appeal reviewer roles")
                }
            },
            Err(error) => {
                tracing::warn!(%guild_id, %error, "could not connect Discord client for appeal authorization")
            }
        }
    }

    state.auth.require_guild(headers, guild_id).await
}

async fn get_appeals(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<FormSubmission>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = require_appeal_reviewer(&state, &headers, &guild_id).await?;
    let items = state
        .infrastructure
        .form_submissions(&guild_id, "moderation-appeals", params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "moderation_appeals.list.viewed",
    )
    .await;
    Ok(Json(items))
}

async fn get_appeal(
    State(state): State<AppState>,
    Path((guild_id, appeal_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<FormSubmission>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = require_appeal_reviewer(&state, &headers, &guild_id).await?;
    let item = state
        .infrastructure
        .form_submission(&guild_id, "moderation-appeals", appeal_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "appeal not found".into()))?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "moderation_appeal.viewed",
    )
    .await;
    Ok(Json(item))
}

async fn review_appeal(
    State(state): State<AppState>,
    Path((guild_id, appeal_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<ReviewFormSubmission>,
) -> Result<Json<FormSubmission>, ApiError> {
    validate_snowflake(&guild_id)?;
    if appeal_id <= 0
        || !matches!(input.status.as_str(), "approved" | "rejected" | "closed")
        || input
            .note
            .as_ref()
            .is_some_and(|note| note.chars().count() > 1000)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid appeal decision".into(),
        ));
    }
    let actor = require_appeal_reviewer(&state, &headers, &guild_id).await?;
    let before = state
        .infrastructure
        .form_submission(&guild_id, "moderation-appeals", appeal_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "appeal not found".into()))?;
    let mut event = NewAuditEvent::guild(
        &guild_id,
        &actor.discord_user_id,
        "moderation_appeal.reviewed",
    );
    event.source = "dashboard_api".into();
    event.target_type = "moderation_appeal".into();
    event.target_id = Some(appeal_id.to_string());
    event.before_state = Some(serde_json::json!({"status":before.status}));
    event.after_state = Some(serde_json::json!({"status":input.status,"note":input.note}));
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    let item = state
        .infrastructure
        .review_form_submission_audited(
            &guild_id,
            "moderation-appeals",
            appeal_id,
            &input.status,
            &actor.discord_user_id,
            input.note.as_deref(),
            &event,
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "appeal not found".into()))?;
    Ok(Json(item))
}

async fn get_polls(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<Poll>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .polls(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "polls.list.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_poll(
    State(state): State<AppState>,
    Path((guild_id, poll_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Poll>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .poll(&guild_id, poll_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "poll not found".into()))?;
    record_read(&state, Some(&guild_id), &actor, &headers, "poll.viewed").await;
    Ok(Json(item))
}
async fn close_poll_dashboard(
    State(state): State<AppState>,
    Path((guild_id, poll_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let before = state
        .infrastructure
        .poll(&guild_id, poll_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "poll not found".into()))?;
    let Some((_, channel_id, message_id, question, options, counts, anonymous)) = state
        .infrastructure
        .close_poll(&guild_id, poll_id)
        .await
        .map_err(internal_error)?
    else {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "poll is already closed".into(),
        ));
    };
    if let Some(token) = state.discord_token.clone()
        && let (Ok(channel_id), Ok(message_id)) = (channel_id.parse(), message_id.parse())
    {
        let total: u64 = counts.iter().sum();
        let results = options
            .iter()
            .zip(&counts)
            .map(|(option, count)| format!("• {} — {count}", option.replace('@', "@\u{200b}")))
            .collect::<Vec<_>>()
            .join("\n");
        let content = format!(
            "**{}**\nPoll closed · {total} total vote(s){}\n{results}",
            question.replace('@', "@\u{200b}"),
            if anonymous { " · anonymous" } else { "" }
        );
        match TwilightDiscord::connect(token).await {
            Ok(discord) => {
                if let Err(error) = discord
                    .edit_message_with_buttons(channel_id, message_id, &content, &[])
                    .await
                {
                    tracing::warn!(%guild_id, %poll_id, %error, "could not synchronize closed poll message");
                }
            }
            Err(error) => {
                tracing::warn!(%guild_id, %poll_id, %error, "could not connect Discord client to synchronize closed poll")
            }
        }
    }
    let mut event = NewAuditEvent::guild(&guild_id, &actor.discord_user_id, "poll.closed");
    event.source = "dashboard_api".into();
    event.target_type = "poll".into();
    event.target_id = Some(poll_id.to_string());
    event.before_state = serde_json::to_value(before).ok();
    event.after_state = Some(serde_json::json!({"status":"closed"}));
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_invite_attributions(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<InviteAttribution>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .invite_attributions(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "invites.attributions.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_invite_leaderboard(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<InviteLeaderboardEntry>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .invite_leaderboard(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "invites.leaderboard.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_verification_attempts(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<VerificationAttempt>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .verification_attempts(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "verification.attempts.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_afk_statuses(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<AfkStatus>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .afk_statuses(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "afk.statuses.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn clear_afk_status_dashboard(
    State(state): State<AppState>,
    Path((guild_id, user_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_snowflake(&user_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if !state
        .infrastructure
        .clear_afk_status(&guild_id, &user_id)
        .await
        .map_err(internal_error)?
    {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "AFK status not found".into(),
        ));
    }
    let mut event = NewAuditEvent::guild(&guild_id, &actor.discord_user_id, "afk.status.cleared");
    event.source = "dashboard_api".into();
    event.target_type = "member".into();
    event.target_id = Some(user_id);
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn get_counter_runs(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<CounterRun>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .counter_runs(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "counters.runs.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_voice_link_events(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Query(params): Query<AuditParams>,
) -> Result<Json<Vec<VoiceLinkEvent>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .voice_link_events(&guild_id, params.limit.unwrap_or(200))
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "voice_links.events.viewed",
    )
    .await;
    Ok(Json(items))
}

async fn get_temporary_voice_channels(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<TemporaryVoiceChannel>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let channels = state
        .infrastructure
        .temporary_voice_channels(&guild_id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "temporary_voice.channels.viewed",
    )
    .await;
    Ok(Json(channels))
}

async fn get_economy_accounts(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<EconomyAccount>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let accounts = state
        .infrastructure
        .economy_accounts(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "economy.accounts.viewed",
    )
    .await;
    Ok(Json(accounts))
}

async fn get_economy_transactions(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<EconomyTransaction>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let transactions = state
        .infrastructure
        .economy_transactions(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "economy.transactions.viewed",
    )
    .await;
    Ok(Json(transactions))
}

async fn delete_temporary_voice_channel(
    State(state): State<AppState>,
    Path((guild_id, channel_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_snowflake(&channel_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let channel = state
        .infrastructure
        .temporary_voice_channel(&guild_id, &channel_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            ApiError(
                StatusCode::NOT_FOUND,
                "temporary voice channel not found".into(),
            )
        })?;
    let discord = TwilightDiscord::connect(state.discord_token.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "DISCORD_TOKEN is not configured".into(),
        )
    })?)
    .await
    .map_err(internal_error)?;
    discord
        .delete_channel(
            channel_id.parse().expect("validated snowflake"),
            "Removed from the Orivane dashboard",
        )
        .await
        .map_err(internal_error)?;
    state
        .infrastructure
        .unregister_temporary_voice_channel(&guild_id, &channel_id)
        .await
        .map_err(internal_error)?;
    let mut event = NewAuditEvent::guild(
        &guild_id,
        &actor.discord_user_id,
        "temporary_voice.channel.deleted",
    );
    event.source = "dashboard_api".into();
    event.target_type = "channel".into();
    event.target_id = Some(channel_id);
    event.before_state = serde_json::to_value(channel).ok();
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(StatusCode::NO_CONTENT)
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

async fn get_suggestions(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<Suggestion>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let suggestions = state
        .infrastructure
        .suggestions(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "suggestions.viewed",
    )
    .await;
    Ok(Json(suggestions))
}

async fn get_suggestion(
    State(state): State<AppState>,
    Path((guild_id, suggestion_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Suggestion>, ApiError> {
    validate_snowflake(&guild_id)?;
    if suggestion_id <= 0 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "invalid suggestion ID".into(),
        ));
    }
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let suggestion = state
        .infrastructure
        .suggestion(&guild_id, suggestion_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "suggestion not found".into()))?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "suggestion.viewed",
    )
    .await;
    Ok(Json(suggestion))
}

async fn update_suggestion_status(
    State(state): State<AppState>,
    Path((guild_id, suggestion_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<UpdateSuggestionStatus>,
) -> Result<Json<Suggestion>, ApiError> {
    validate_snowflake(&guild_id)?;
    if suggestion_id <= 0
        || !matches!(
            input.status.as_str(),
            "pending" | "approved" | "rejected" | "implemented" | "duplicate"
        )
        || input
            .staff_note
            .as_ref()
            .is_some_and(|note| note.chars().count() > 500)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid suggestion status or staff note".into(),
        ));
    }
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let before = state
        .infrastructure
        .suggestion(&guild_id, suggestion_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "suggestion not found".into()))?;
    let mut event = NewAuditEvent::guild(
        &guild_id,
        &actor.discord_user_id,
        "suggestion.status.updated",
    );
    event.source = "dashboard_api".into();
    event.target_type = "suggestion".into();
    event.target_id = Some(suggestion_id.to_string());
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    event.before_state = Some(serde_json::json!({ "status": before.status }));
    event.after_state = Some(serde_json::json!({ "status": input.status }));
    event.metadata = serde_json::json!({ "status": input.status });
    let suggestion = state
        .infrastructure
        .update_suggestion_status_audited(
            &guild_id,
            suggestion_id,
            &input.status,
            input.staff_note.as_deref(),
            &event,
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "suggestion not found".into()))?;
    Ok(Json(suggestion))
}

async fn get_giveaways(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<Giveaway>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .giveaways(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "giveaways.viewed",
    )
    .await;
    Ok(Json(items))
}

async fn get_giveaway(
    State(state): State<AppState>,
    Path((guild_id, giveaway_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Giveaway>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(giveaway_id, "giveaway")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .giveaway(&guild_id, giveaway_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "giveaway not found".into()))?;
    record_read(&state, Some(&guild_id), &actor, &headers, "giveaway.viewed").await;
    Ok(Json(item))
}

async fn get_giveaway_entries(
    State(state): State<AppState>,
    Path((guild_id, giveaway_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Vec<GiveawayEntry>>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(giveaway_id, "giveaway")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if state
        .infrastructure
        .giveaway(&guild_id, giveaway_id)
        .await
        .map_err(internal_error)?
        .is_none()
    {
        return Err(ApiError(StatusCode::NOT_FOUND, "giveaway not found".into()));
    }
    let entries = state
        .infrastructure
        .giveaway_entries(&guild_id, giveaway_id, 5_000)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "giveaway.entries.viewed",
    )
    .await;
    Ok(Json(entries))
}

async fn create_giveaway(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<CreateGiveawayRequest>,
) -> Result<(StatusCode, Json<Giveaway>), ApiError> {
    validate_snowflake(&guild_id)?;
    validate_giveaway_input(&input)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let config = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    if !config.modules.giveaways.enabled {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "giveaways module is disabled".into(),
        ));
    }
    let ends_at =
        chrono::DateTime::from_timestamp(i64::try_from(input.ends_at_unix).unwrap_or(i64::MAX), 0)
            .ok_or_else(|| {
                ApiError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid giveaway end time".into(),
                )
            })?;
    let giveaway = state
        .infrastructure
        .create_giveaway(&NewGiveaway {
            guild_id: guild_id.clone(),
            channel_id: input.channel_id.clone(),
            creator_id: actor.discord_user_id.clone(),
            title: input.title.trim().into(),
            description: input.description.trim().into(),
            winner_count: i16::from(input.winner_count),
            minimum_account_age_days: i32::from(input.minimum_account_age_days),
            required_role_ids: serde_json::json!(input.required_role_ids),
            bonus_role_id: input.bonus_role_id.clone(),
            bonus_entries: i16::from(input.bonus_entries),
            ends_at,
        })
        .await
        .map_err(internal_error)?;
    let discord = api_discord(&state).await?;
    let delivery = discord
        .send_message_with_buttons(
            input.channel_id.parse().expect("validated channel"),
            &giveaway_post(&giveaway),
            &[MessageButton {
                custom_id: format!("orivane:giveaway:enter:{}", giveaway.id),
                label: "Enter giveaway".into(),
                style: MessageButtonStyle::Primary,
            }],
        )
        .await;
    let message_id = match delivery {
        Ok(id) => id,
        Err(error) => {
            state
                .infrastructure
                .fail_giveaway_delivery(&guild_id, giveaway.id)
                .await
                .map_err(internal_error)?;
            record_giveaway_action(
                &state,
                &actor,
                &headers,
                &guild_id,
                giveaway.id,
                "giveaway.created",
                "failure",
                serde_json::json!({ "delivery": "failed" }),
                Some(error.to_string()),
            )
            .await?;
            return Err(ApiError(
                StatusCode::BAD_GATEWAY,
                "Discord giveaway delivery failed".into(),
            ));
        }
    };
    state
        .infrastructure
        .activate_giveaway(&guild_id, giveaway.id, &message_id.to_string())
        .await
        .map_err(internal_error)?;
    record_giveaway_action(&state, &actor, &headers, &guild_id, giveaway.id, "giveaway.created", "success", serde_json::json!({ "winner_count": input.winner_count, "ends_at_unix": input.ends_at_unix }), None).await?;
    let saved = state
        .infrastructure
        .giveaway(&guild_id, giveaway.id)
        .await
        .map_err(internal_error)?
        .expect("activated giveaway exists");
    Ok((StatusCode::CREATED, Json(saved)))
}

async fn end_giveaway(
    State(state): State<AppState>,
    Path((guild_id, giveaway_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Giveaway>, ApiError> {
    change_giveaway_winners(state, guild_id, giveaway_id, headers, false).await
}

async fn reroll_giveaway(
    State(state): State<AppState>,
    Path((guild_id, giveaway_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Giveaway>, ApiError> {
    change_giveaway_winners(state, guild_id, giveaway_id, headers, true).await
}

async fn change_giveaway_winners(
    state: AppState,
    guild_id: String,
    giveaway_id: i64,
    headers: HeaderMap,
    reroll: bool,
) -> Result<Json<Giveaway>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(giveaway_id, "giveaway")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = if reroll {
        state
            .infrastructure
            .reroll_giveaway(&guild_id, giveaway_id)
            .await
    } else {
        state
            .infrastructure
            .end_giveaway(&guild_id, giveaway_id, true)
            .await
    }
    .map_err(internal_error)?
    .ok_or_else(|| {
        ApiError(
            StatusCode::CONFLICT,
            if reroll {
                "no replacement entrants are available".into()
            } else {
                "giveaway is not active".into()
            },
        )
    })?;
    let delivered = match api_discord(&state).await {
        Ok(discord) => discord
            .send_message(
                item.channel_id.parse().expect("stored channel is valid"),
                &giveaway_result_post(&item, reroll),
            )
            .await
            .is_ok(),
        Err(_) => false,
    };
    if delivered {
        state
            .infrastructure
            .mark_giveaway_announced(&guild_id, giveaway_id)
            .await
            .map_err(internal_error)?;
    }
    record_giveaway_action(&state, &actor, &headers, &guild_id, giveaway_id, if reroll { "giveaway.rerolled" } else { "giveaway.ended" }, "success", serde_json::json!({ "winner_count": winner_ids(&item).len(), "announcement_delivered": delivered }), None).await?;
    Ok(Json(item))
}

fn validate_giveaway_input(input: &CreateGiveawayRequest) -> Result<(), ApiError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if validate_snowflake(&input.channel_id).is_err()
        || input.title.trim().is_empty()
        || input.title.chars().count() > 100
        || input.description.chars().count() > 1_800
        || !(1..=25).contains(&input.winner_count)
        || input.minimum_account_age_days > 3650
        || input.required_role_ids.len() > 25
        || input
            .required_role_ids
            .iter()
            .any(|id| validate_snowflake(id).is_err())
        || input
            .bonus_role_id
            .as_ref()
            .is_some_and(|id| validate_snowflake(id).is_err())
        || input.bonus_entries > 25
        || input.ends_at_unix <= now
        || input.ends_at_unix > now.saturating_add(31_536_000)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid giveaway configuration".into(),
        ));
    }
    Ok(())
}

fn validate_record_id(id: i64, name: &str) -> Result<(), ApiError> {
    if id <= 0 {
        Err(ApiError(
            StatusCode::BAD_REQUEST,
            format!("invalid {name} ID"),
        ))
    } else {
        Ok(())
    }
}

async fn get_reminders(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<Reminder>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let reminders = state
        .infrastructure
        .reminders(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "reminders.viewed",
    )
    .await;
    Ok(Json(reminders))
}

async fn get_reminder(
    State(state): State<AppState>,
    Path((guild_id, reminder_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Reminder>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(reminder_id, "reminder")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let reminder = state
        .infrastructure
        .reminder(&guild_id, reminder_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "reminder not found".into()))?;
    record_read(&state, Some(&guild_id), &actor, &headers, "reminder.viewed").await;
    Ok(Json(reminder))
}

async fn cancel_reminder(
    State(state): State<AppState>,
    Path((guild_id, reminder_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Reminder>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(reminder_id, "reminder")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if !state
        .infrastructure
        .cancel_reminder_as_admin(&guild_id, reminder_id)
        .await
        .map_err(internal_error)?
    {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "reminder is not active".into(),
        ));
    }
    let reminder = state
        .infrastructure
        .reminder(&guild_id, reminder_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "reminder not found".into()))?;
    let mut event = NewAuditEvent::guild(&guild_id, &actor.discord_user_id, "reminder.cancelled");
    event.source = "dashboard_api".into();
    event.target_type = "reminder".into();
    event.target_id = Some(reminder_id.to_string());
    event.metadata = serde_json::json!({ "delivery_mode": reminder.delivery_mode });
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(Json(reminder))
}
async fn get_message_deletions(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<MessageDeletionJob>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let jobs = state
        .infrastructure
        .message_deletion_jobs(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "auto_delete.jobs.viewed",
    )
    .await;
    Ok(Json(jobs))
}

async fn get_message_deletion(
    State(state): State<AppState>,
    Path((guild_id, channel_id, message_id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Json<MessageDeletionJob>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_snowflake(&channel_id)?;
    validate_snowflake(&message_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let job = state
        .infrastructure
        .message_deletion_job(&guild_id, &channel_id, &message_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "deletion job not found".into()))?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "auto_delete.job.viewed",
    )
    .await;
    Ok(Json(job))
}

fn ticket_panel_input(
    guild_id: &str,
    actor: &Actor,
    input: SaveTicketPanelRequest,
) -> Result<(NewTicketPanel, String), ApiError> {
    if input.name.trim().is_empty()
        || input.name.chars().count() > 80
        || input.title.trim().is_empty()
        || input.title.chars().count() > 100
        || input.description.chars().count() > 1000
        || input.button_label.trim().is_empty()
        || input.button_label.chars().count() > 80
        || input.welcome_message.trim().is_empty()
        || input.welcome_message.chars().count() > 1800
        || input.channel_name.trim().is_empty()
        || input.channel_name.chars().count() > 100
        || input.intake_label.trim().is_empty()
        || input.max_open_per_user < 1
        || input.max_open_per_user > 5
        || !(0..=8760).contains(&input.auto_close_hours)
        || !(1..=3650).contains(&input.transcript_retention_days)
        || !matches!(
            input.default_priority.as_str(),
            "low" | "normal" | "high" | "urgent"
        )
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid ticket panel settings".into(),
        ));
    }
    validate_snowflake(&input.channel_id)?;
    validate_snowflake(&input.category_id)?;
    for role in &input.support_role_ids {
        validate_snowflake(role)?
    }
    if input.support_role_ids.is_empty() {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "at least one support role is required".into(),
        ));
    }
    let channel = input.channel_id.clone();
    Ok((
        NewTicketPanel {
            guild_id: guild_id.into(),
            name: input.name,
            title: input.title,
            description: input.description,
            panel_channel_id: Some(channel.clone()),
            category_id: input.category_id,
            support_role_ids: serde_json::json!(input.support_role_ids),
            button_label: input.button_label,
            welcome_message: input.welcome_message,
            channel_name: input.channel_name,
            intake_enabled: input.intake_enabled,
            intake_label: input.intake_label,
            max_open_per_user: input.max_open_per_user,
            auto_close_hours: input.auto_close_hours,
            transcript_retention_days: input.transcript_retention_days,
            default_priority: input.default_priority,
            enabled: input.enabled,
            created_by: actor.discord_user_id.clone(),
        },
        channel,
    ))
}

async fn ticket_audit(
    state: &AppState,
    guild_id: &str,
    actor: &Actor,
    headers: &HeaderMap,
    action: &str,
    target: i64,
    metadata: serde_json::Value,
) {
    let mut event = NewAuditEvent::guild(guild_id, &actor.discord_user_id, action);
    event.source = "dashboard_api".into();
    event.target_type = if action.starts_with("ticket.panel.") {
        "ticket_panel"
    } else {
        "ticket"
    }
    .into();
    event.target_id = Some(target.to_string());
    event.request_id = header_value(headers, "x-request-id").map(str::to_owned);
    event.metadata = metadata;
    let _ = state.infrastructure.record_audit_event(&event).await;
}

async fn get_ticket_panels(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<TicketPanel>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .ticket_panels(&guild_id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "tickets.panels.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_ticket_panel(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<TicketPanel>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .ticket_panel(&guild_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket panel not found".into()))?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "tickets.panel.viewed",
    )
    .await;
    Ok(Json(item))
}
async fn create_ticket_panel(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<SaveTicketPanelRequest>,
) -> Result<(StatusCode, Json<TicketPanel>), ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let (input, _) = ticket_panel_input(&guild_id, &actor, input)?;
    let item = state
        .infrastructure
        .create_ticket_panel(&input)
        .await
        .map_err(internal_error)?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.panel.created",
        item.id,
        serde_json::json!({"name":item.name}),
    )
    .await;
    Ok((StatusCode::CREATED, Json(item)))
}
async fn update_ticket_panel(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<SaveTicketPanelRequest>,
) -> Result<Json<TicketPanel>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let (input, _) = ticket_panel_input(&guild_id, &actor, input)?;
    let item = state
        .infrastructure
        .update_ticket_panel(&guild_id, id, &input)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket panel not found".into()))?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.panel.updated",
        id,
        serde_json::json!({"name":item.name}),
    )
    .await;
    Ok(Json(item))
}
async fn delete_ticket_panel(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if !state
        .infrastructure
        .delete_ticket_panel(&guild_id, id)
        .await
        .map_err(internal_error)?
    {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "ticket panel not found".into(),
        ));
    }
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.panel.deleted",
        id,
        serde_json::json!({}),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
async fn deploy_ticket_panel(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<TicketPanel>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let panel = state
        .infrastructure
        .ticket_panel(&guild_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket panel not found".into()))?;
    if !panel.enabled {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "enable the panel before deploying it".into(),
        ));
    }
    let channel_id = panel.panel_channel_id.clone().ok_or_else(|| {
        ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "select a panel channel before deploying".into(),
        )
    })?;
    let discord = TwilightDiscord::connect(state.discord_token.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Discord actions are unavailable".into(),
        )
    })?)
    .await
    .map_err(internal_error)?;
    let discord_channel_id = channel_id.parse().map_err(|_| {
        ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid panel channel".into(),
        )
    })?;
    let content = format!("**{}**\n{}", panel.title, panel.description);
    let buttons = [MessageButton {
        custom_id: format!("orivane:ticket:open:{id}"),
        label: panel.button_label.clone(),
        style: MessageButtonStyle::Primary,
    }];
    let (message_id, synchronized) = if let Some(existing) = panel.panel_message_id.as_deref() {
        let message_id = existing.parse::<u64>().map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "stored panel message is invalid".into(),
            )
        })?;
        discord
            .edit_message_with_buttons(discord_channel_id, message_id, &content, &buttons)
            .await
            .map_err(internal_error)?;
        (message_id, true)
    } else {
        (
            discord
                .send_message_with_buttons(discord_channel_id, &content, &buttons)
                .await
                .map_err(internal_error)?,
            false,
        )
    };
    let item = state
        .infrastructure
        .mark_ticket_panel_deployed(&guild_id, id, &channel_id, &message_id.to_string())
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket panel not found".into()))?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        if synchronized {
            "ticket.panel.synchronized"
        } else {
            "ticket.panel.deployed"
        },
        id,
        serde_json::json!({"channel_id":channel_id,"message_id":message_id}),
    )
    .await;
    Ok(Json(item))
}

async fn get_tickets(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<Ticket>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .tickets(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(&state, Some(&guild_id), &actor, &headers, "tickets.viewed").await;
    Ok(Json(items))
}
async fn get_ticket_analytics(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<TicketAnalytics>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let analytics = state
        .infrastructure
        .ticket_analytics(&guild_id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "tickets.analytics.viewed",
    )
    .await;
    Ok(Json(analytics))
}
async fn get_ticket(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Ticket>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .ticket(&guild_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket not found".into()))?;
    record_read(&state, Some(&guild_id), &actor, &headers, "ticket.viewed").await;
    Ok(Json(item))
}
async fn update_ticket_metadata(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<UpdateTicketMetadataRequest>,
) -> Result<Json<Ticket>, ApiError> {
    validate_snowflake(&guild_id)?;
    if !matches!(
        input.priority.as_str(),
        "low" | "normal" | "high" | "urgent"
    ) || input.tags.len() > 10
        || input
            .tags
            .iter()
            .any(|tag| tag.trim().is_empty() || tag.chars().count() > 32)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid ticket priority or tags".into(),
        ));
    }
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let mut tags: Vec<String> = input
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .collect();
    tags.sort();
    tags.dedup();
    let item = state
        .infrastructure
        .update_ticket_metadata(
            &guild_id,
            id,
            &actor.discord_user_id,
            &input.priority,
            &serde_json::json!(tags),
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket not found".into()))?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.metadata.updated",
        id,
        serde_json::json!({"priority": item.priority, "tags": item.tags}),
    )
    .await;
    Ok(Json(item))
}
async fn get_ticket_events(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Vec<TicketEvent>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .ticket_events(&guild_id, id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "ticket.events.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn get_ticket_notes(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Vec<TicketNote>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .ticket_notes(&guild_id, id)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "ticket.notes.viewed",
    )
    .await;
    Ok(Json(items))
}
async fn create_ticket_note(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<CreateTicketNoteRequest>,
) -> Result<(StatusCode, Json<TicketNote>), ApiError> {
    validate_snowflake(&guild_id)?;
    if input.body.trim().is_empty() || input.body.chars().count() > 2000 {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ticket note must contain between 1 and 2000 characters".into(),
        ));
    }
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if state
        .infrastructure
        .ticket(&guild_id, id)
        .await
        .map_err(internal_error)?
        .is_none()
    {
        return Err(ApiError(StatusCode::NOT_FOUND, "ticket not found".into()));
    }
    let note = state
        .infrastructure
        .add_ticket_note(&guild_id, id, &actor.discord_user_id, &input.body)
        .await
        .map_err(internal_error)?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.note.created",
        id,
        serde_json::json!({"note_id": note.id}),
    )
    .await;
    Ok((StatusCode::CREATED, Json(note)))
}
async fn download_ticket_transcript(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .ticket(&guild_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket not found".into()))?;
    let transcript = item.transcript.ok_or_else(|| {
        ApiError(
            StatusCode::NOT_FOUND,
            "this ticket does not have a captured transcript".into(),
        )
    })?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "ticket.transcript.downloaded",
    )
    .await;
    Ok((
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            ),
            (
                CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=ticket-{id}.txt"))
                    .map_err(internal_error)?,
            ),
        ],
        transcript,
    )
        .into_response())
}
async fn claim_ticket(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Ticket>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .claim_ticket(&guild_id, id, &actor.discord_user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::CONFLICT, "ticket is not open".into()))?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.claimed",
        id,
        serde_json::json!({}),
    )
    .await;
    Ok(Json(item))
}
async fn close_ticket(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<TicketReasonRequest>,
) -> Result<Json<Ticket>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if input.reason.chars().count() > 500 {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "close reason is too long".into(),
        ));
    }
    let before = state
        .infrastructure
        .ticket(&guild_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket not found".into()))?;
    let discord = TwilightDiscord::connect(state.discord_token.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Discord actions are unavailable".into(),
        )
    })?)
    .await
    .map_err(internal_error)?;
    let channel_id = before.channel_id.parse::<u64>().map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "stored ticket channel is invalid".into(),
        )
    })?;
    let transcript = discord
        .channel_transcript(channel_id)
        .await
        .map_err(internal_error)?;
    let item = state
        .infrastructure
        .close_ticket(
            &guild_id,
            id,
            &actor.discord_user_id,
            if input.reason.trim().is_empty() {
                "Closed from dashboard"
            } else {
                input.reason.trim()
            },
            &transcript,
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::CONFLICT, "ticket is not open".into()))?;
    discord
        .set_ticket_member_writable(
            channel_id,
            before.opener_id.parse().map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "stored ticket member is invalid".into(),
                )
            })?,
            false,
            "Ticket closed from Orivane dashboard",
        )
        .await
        .map_err(internal_error)?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.closed",
        id,
        serde_json::json!({"reason":item.closed_reason}),
    )
    .await;
    Ok(Json(item))
}
async fn reopen_ticket(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<Ticket>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let before = state
        .infrastructure
        .ticket(&guild_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "ticket not found".into()))?;
    let discord = TwilightDiscord::connect(state.discord_token.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Discord actions are unavailable".into(),
        )
    })?)
    .await
    .map_err(internal_error)?;
    let item = state
        .infrastructure
        .reopen_ticket(&guild_id, id, &actor.discord_user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::CONFLICT, "ticket is not closed".into()))?;
    discord
        .set_ticket_member_writable(
            before.channel_id.parse().map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "stored ticket channel is invalid".into(),
                )
            })?,
            before.opener_id.parse().map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "stored ticket member is invalid".into(),
                )
            })?,
            true,
            "Ticket reopened from Orivane dashboard",
        )
        .await
        .map_err(internal_error)?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.reopened",
        id,
        serde_json::json!({}),
    )
    .await;
    Ok(Json(item))
}
async fn transfer_ticket(
    State(state): State<AppState>,
    Path((guild_id, id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<TicketTransferRequest>,
) -> Result<Json<Ticket>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_snowflake(&input.user_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .assign_ticket(&guild_id, id, &actor.discord_user_id, &input.user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::CONFLICT, "ticket is not open".into()))?;
    ticket_audit(
        &state,
        &guild_id,
        &actor,
        &headers,
        "ticket.transferred",
        id,
        serde_json::json!({"assigned_to":input.user_id}),
    )
    .await;
    Ok(Json(item))
}

async fn deploy_reaction_role_panel(
    State(state): State<AppState>,
    Path((guild_id, panel_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<GuildConfig>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let mut config = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    let panel = config
        .modules
        .reaction_roles
        .panels
        .iter()
        .find(|panel| panel.id == panel_id)
        .cloned()
        .ok_or_else(|| {
            ApiError(
                StatusCode::NOT_FOUND,
                "reaction-role panel not found".into(),
            )
        })?;
    if !panel.enabled || panel.options.is_empty() {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "enable the panel and configure at least one role before deploying".into(),
        ));
    }
    let channel_id = panel.channel_id.parse::<u64>().map_err(|_| {
        ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "reaction-role channel is invalid".into(),
        )
    })?;
    let definition = RolePanelComponents {
        custom_id: if panel.component_type == "select" {
            format!("orivane:rr-select:{}", panel.id)
        } else {
            format!("orivane:rr:{}", panel.id)
        },
        kind: panel.component_type.clone(),
        exclusive: panel.exclusive,
        options: panel
            .options
            .iter()
            .map(|option| RolePanelOption {
                id: option.id.clone(),
                label: option.label.clone(),
                description: option.description.clone(),
                emoji: option.emoji.clone(),
            })
            .collect(),
    };
    let content = if panel.description.trim().is_empty() {
        format!("**{}**", panel.title)
    } else {
        format!("**{}**\n{}", panel.title, panel.description)
    };
    let discord = TwilightDiscord::connect(state.discord_token.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Discord actions are unavailable".into(),
        )
    })?)
    .await
    .map_err(internal_error)?;
    let synchronized = panel.message_id.is_some();
    let message_id = if let Some(message_id) = panel.message_id.as_deref() {
        discord
            .edit_role_panel(
                channel_id,
                message_id.parse().map_err(|_| {
                    ApiError(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "stored reaction-role message is invalid".into(),
                    )
                })?,
                &content,
                &definition,
            )
            .await
            .map_err(internal_error)?;
        message_id.to_owned()
    } else {
        discord
            .send_role_panel(channel_id, &content, &definition)
            .await
            .map_err(internal_error)?
            .to_string()
    };
    let before = config.clone();
    config
        .modules
        .reaction_roles
        .panels
        .iter_mut()
        .find(|item| item.id == panel_id)
        .expect("reaction-role panel was loaded")
        .message_id = Some(message_id.clone());
    config
        .validate()
        .map_err(|error| ApiError(StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))?;
    let mut event = NewAuditEvent::guild(
        &guild_id,
        &actor.discord_user_id,
        if synchronized {
            "reaction_roles.panel.synchronized"
        } else {
            "reaction_roles.panel.deployed"
        },
    );
    event.source = "dashboard_api".into();
    event.target_type = "reaction_role_panel".into();
    event.target_id = Some(panel_id);
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    event.before_state = serde_json::to_value(before).ok();
    event.after_state = serde_json::to_value(&config).ok();
    event.metadata =
        serde_json::json!({"channel_id":channel_id.to_string(),"message_id":message_id});
    state
        .infrastructure
        .put_guild_config_audited(&config, &event)
        .await
        .map_err(internal_error)?;
    Ok(Json(config))
}

async fn get_embed_templates(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<EmbedTemplate>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .embed_templates(&guild_id, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "managed_embeds.templates.viewed",
    )
    .await;
    Ok(Json(items))
}

async fn get_embed_template(
    State(state): State<AppState>,
    Path((guild_id, template_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<Json<EmbedTemplate>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(template_id, "embed template")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .embed_template(&guild_id, template_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "embed template not found".into()))?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "managed_embeds.template.viewed",
    )
    .await;
    Ok(Json(item))
}

async fn create_embed_template(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<SaveEmbedTemplateRequest>,
) -> Result<(StatusCode, Json<EmbedTemplate>), ApiError> {
    validate_snowflake(&guild_id)?;
    validate_embed(&input)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .create_embed_template(&NewEmbedTemplate {
            guild_id: guild_id.clone(),
            name: input.name.trim().into(),
            payload: input.payload,
            created_by: actor.discord_user_id.clone(),
        })
        .await
        .map_err(internal_error)?;
    record_embed_audit(
        &state,
        &actor,
        &headers,
        &guild_id,
        item.id,
        "managed_embeds.template.created",
        "success",
        serde_json::json!({"name":item.name}),
        None,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(item)))
}

async fn update_embed_template(
    State(state): State<AppState>,
    Path((guild_id, template_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<SaveEmbedTemplateRequest>,
) -> Result<Json<EmbedTemplate>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(template_id, "embed template")?;
    validate_embed(&input)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let item = state
        .infrastructure
        .update_embed_template(&guild_id, template_id, &input.name, &input.payload)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "embed template not found".into()))?;
    record_embed_audit(
        &state,
        &actor,
        &headers,
        &guild_id,
        template_id,
        "managed_embeds.template.updated",
        "success",
        serde_json::json!({"name":item.name}),
        None,
    )
    .await?;
    Ok(Json(item))
}

async fn delete_embed_template(
    State(state): State<AppState>,
    Path((guild_id, template_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(template_id, "embed template")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    if !state
        .infrastructure
        .delete_embed_template(&guild_id, template_id)
        .await
        .map_err(internal_error)?
    {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "embed template not found".into(),
        ));
    }
    record_embed_audit(
        &state,
        &actor,
        &headers,
        &guild_id,
        template_id,
        "managed_embeds.template.deleted",
        "success",
        serde_json::json!({}),
        None,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_embed_deliveries(
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<EmbedDelivery>>, ApiError> {
    validate_snowflake(&guild_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let items = state
        .infrastructure
        .embed_deliveries(&guild_id, None, 500)
        .await
        .map_err(internal_error)?;
    record_read(
        &state,
        Some(&guild_id),
        &actor,
        &headers,
        "managed_embeds.deliveries.viewed",
    )
    .await;
    Ok(Json(items))
}

async fn send_embed_template(
    State(state): State<AppState>,
    Path((guild_id, template_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(input): Json<DeliverEmbedRequest>,
) -> Result<(StatusCode, Json<EmbedDelivery>), ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(template_id, "embed template")?;
    validate_snowflake(&input.channel_id)?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let config = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    if !config.modules.managed_embeds.enabled {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "managed embeds module is disabled".into(),
        ));
    }
    let template = state
        .infrastructure
        .embed_template(&guild_id, template_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "embed template not found".into()))?;
    let payload: EmbedPayload = serde_json::from_value(template.payload).map_err(internal_error)?;
    let guild_name = state
        .infrastructure
        .connected_guild_name(&guild_id)
        .await
        .map_err(internal_error)?
        .unwrap_or_else(|| "this server".into());
    let rich = bot_module_managed_embeds::rich_embed(
        &payload,
        guild_id.parse().expect("validated guild"),
        &guild_name,
        actor.discord_user_id.parse().unwrap_or_default(),
    );
    let delivery = match api_discord(&state)
        .await?
        .send_rich_embed(input.channel_id.parse().expect("validated channel"), &rich)
        .await
    {
        Ok(message_id) => NewEmbedDelivery {
            guild_id: guild_id.clone(),
            template_id,
            channel_id: input.channel_id.clone(),
            message_id: Some(message_id.to_string()),
            actor_id: actor.discord_user_id.clone(),
            action: "send".into(),
            status: "delivered".into(),
            error: None,
        },
        Err(error) => NewEmbedDelivery {
            guild_id: guild_id.clone(),
            template_id,
            channel_id: input.channel_id.clone(),
            message_id: None,
            actor_id: actor.discord_user_id.clone(),
            action: "send".into(),
            status: "failed".into(),
            error: Some(error.to_string()),
        },
    };
    let failed = delivery.status == "failed";
    let reason = delivery.error.clone();
    let saved = state
        .infrastructure
        .record_embed_delivery(&delivery)
        .await
        .map_err(internal_error)?;
    record_embed_audit(&state,&actor,&headers,&guild_id,template_id,"managed_embeds.sent",if failed{"failure"}else{"success"},serde_json::json!({"channel_id":input.channel_id,"delivery_id":saved.id,"message_id":saved.message_id}),reason.clone()).await?;
    if failed {
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "Discord embed delivery failed".into(),
        ));
    }
    Ok((StatusCode::CREATED, Json(saved)))
}

async fn sync_embed_delivery(
    State(state): State<AppState>,
    Path((guild_id, template_id, delivery_id)): Path<(String, i64, i64)>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<EmbedDelivery>), ApiError> {
    validate_snowflake(&guild_id)?;
    validate_record_id(template_id, "embed template")?;
    validate_record_id(delivery_id, "embed delivery")?;
    let actor = state.auth.require_guild(&headers, &guild_id).await?;
    let config = state
        .infrastructure
        .guild_config(&guild_id)
        .await
        .map_err(internal_error)?;
    if !config.modules.managed_embeds.enabled {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "managed embeds module is disabled".into(),
        ));
    }
    let template = state
        .infrastructure
        .embed_template(&guild_id, template_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "embed template not found".into()))?;
    let previous = state
        .infrastructure
        .embed_delivery(&guild_id, template_id, delivery_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "embed delivery not found".into()))?;
    let message_id = previous.message_id.as_deref().ok_or_else(|| {
        ApiError(
            StatusCode::CONFLICT,
            "failed deliveries cannot be synchronized".into(),
        )
    })?;
    let payload: EmbedPayload = serde_json::from_value(template.payload).map_err(internal_error)?;
    let guild_name = state
        .infrastructure
        .connected_guild_name(&guild_id)
        .await
        .map_err(internal_error)?
        .unwrap_or_else(|| "this server".into());
    let rich = bot_module_managed_embeds::rich_embed(
        &payload,
        guild_id.parse().expect("validated guild"),
        &guild_name,
        actor.discord_user_id.parse().unwrap_or_default(),
    );
    let result = api_discord(&state)
        .await?
        .edit_rich_embed(
            previous.channel_id.parse().expect("stored channel"),
            message_id.parse().map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "stored message ID is invalid".into(),
                )
            })?,
            &rich,
        )
        .await;
    let error = result.err().map(|e| e.to_string());
    let saved = state
        .infrastructure
        .record_embed_delivery(&NewEmbedDelivery {
            guild_id: guild_id.clone(),
            template_id,
            channel_id: previous.channel_id.clone(),
            message_id: previous.message_id.clone(),
            actor_id: actor.discord_user_id.clone(),
            action: "edit".into(),
            status: if error.is_some() {
                "failed".into()
            } else {
                "delivered".into()
            },
            error: error.clone(),
        })
        .await
        .map_err(internal_error)?;
    record_embed_audit(
        &state,
        &actor,
        &headers,
        &guild_id,
        template_id,
        "managed_embeds.delivery.synchronized",
        if error.is_some() {
            "failure"
        } else {
            "success"
        },
        serde_json::json!({"delivery_id":saved.id,"source_delivery_id":delivery_id}),
        error.clone(),
    )
    .await?;
    if error.is_some() {
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "Discord embed update failed".into(),
        ));
    }
    Ok((StatusCode::CREATED, Json(saved)))
}

fn validate_embed(input: &SaveEmbedTemplateRequest) -> Result<(), ApiError> {
    let p = &input.payload;
    let total = p.title.chars().count()
        + p.description.chars().count()
        + p.author_name.chars().count()
        + p.footer_text.chars().count()
        + p.fields
            .iter()
            .map(|f| f.name.chars().count() + f.value.chars().count())
            .sum::<usize>();
    let invalid_url =
        |v: &str| !v.is_empty() && !v.starts_with("https://") && !v.starts_with("http://");
    if input.name.trim().is_empty()
        || input.name.chars().count() > 100
        || p.content.chars().count() > 2000
        || p.title.chars().count() > 256
        || p.description.chars().count() > 4096
        || p.author_name.chars().count() > 256
        || p.footer_text.chars().count() > 2048
        || p.fields.len() > 25
        || p.fields.iter().any(|f| {
            f.name.trim().is_empty()
                || f.name.chars().count() > 256
                || f.value.trim().is_empty()
                || f.value.chars().count() > 1024
        })
        || total > 6000
        || p.color > 0xFF_FF_FF
        || [
            &p.url,
            &p.author_url,
            &p.author_icon_url,
            &p.footer_icon_url,
            &p.thumbnail_url,
            &p.image_url,
        ]
        .into_iter()
        .any(|v| invalid_url(v))
        || (p.content.trim().is_empty()
            && p.title.trim().is_empty()
            && p.description.trim().is_empty()
            && p.fields.is_empty()
            && p.image_url.is_empty())
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid embed template".into(),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn record_embed_audit(
    state: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    guild_id: &str,
    template_id: i64,
    action: &str,
    outcome: &str,
    metadata: serde_json::Value,
    reason: Option<String>,
) -> Result<(), ApiError> {
    let mut event = NewAuditEvent::guild(guild_id, &actor.discord_user_id, action);
    event.source = "dashboard_api".into();
    event.target_type = "embed_template".into();
    event.target_id = Some(template_id.to_string());
    event.outcome = outcome.into();
    event.metadata = metadata;
    event.reason = reason;
    event.request_id = header_value(headers, "x-request-id").map(str::to_owned);
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(())
}

async fn api_discord(state: &AppState) -> Result<TwilightDiscord, ApiError> {
    let token = state.discord_token.clone().ok_or_else(|| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "DISCORD_TOKEN is not configured".into(),
        )
    })?;
    TwilightDiscord::connect(token).await.map_err(|error| {
        ApiError(
            StatusCode::BAD_GATEWAY,
            format!("Discord connection failed: {error}"),
        )
    })
}
fn winner_ids(item: &Giveaway) -> Vec<String> {
    serde_json::from_value(item.winner_ids.clone()).unwrap_or_default()
}
fn giveaway_post(item: &Giveaway) -> String {
    let description = safe_discord_text(item.description.trim());
    let details = if description.is_empty() {
        String::new()
    } else {
        format!("\n{description}\n")
    };
    format!(
        "**Giveaway #{} · {}**\n{}\nEnds <t:{}:R> · {} winner{}\nPress the button below to enter.",
        item.id,
        safe_discord_text(&item.title),
        details,
        item.ends_at.timestamp(),
        item.winner_count,
        if item.winner_count == 1 { "" } else { "s" }
    )
}
fn giveaway_result_post(item: &Giveaway, reroll: bool) -> String {
    let winners = winner_ids(item);
    let mentions = if winners.is_empty() {
        "No eligible entries were received.".into()
    } else {
        winners
            .iter()
            .map(|id| format!("<@{id}>"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "**{}: Giveaway #{} · {}**\nWinner{}: {mentions}",
        if reroll { "Rerolled" } else { "Ended" },
        item.id,
        safe_discord_text(&item.title),
        if winners.len() == 1 { "" } else { "s" }
    )
}
fn safe_discord_text(value: &str) -> String {
    value.replace('@', "@\u{200b}")
}
#[allow(clippy::too_many_arguments)]
async fn record_giveaway_action(
    state: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    guild_id: &str,
    id: i64,
    action: &str,
    outcome: &str,
    metadata: serde_json::Value,
    reason: Option<String>,
) -> Result<(), ApiError> {
    let mut event = NewAuditEvent::guild(guild_id, &actor.discord_user_id, action);
    event.source = "dashboard_api".into();
    event.target_type = "giveaway".into();
    event.target_id = Some(id.to_string());
    event.outcome = outcome.into();
    event.metadata = metadata;
    event.reason = reason;
    event.request_id = header_value(headers, "x-request-id").map(str::to_owned);
    state
        .infrastructure
        .record_audit_event(&event)
        .await
        .map_err(internal_error)?;
    Ok(())
}

async fn get_form_submission(
    State(state): State<AppState>,
    Path((guild_id, form_id, submission_id)): Path<(String, String, i64)>,
    headers: HeaderMap,
) -> Result<Json<FormSubmission>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_form_id(&form_id)?;
    let actor = if form_id == "moderation-appeals" {
        require_appeal_reviewer(&state, &headers, &guild_id).await?
    } else {
        state.auth.require_guild(&headers, &guild_id).await?
    };
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

#[derive(Deserialize)]
struct ReviewFormSubmission {
    status: String,
    note: Option<String>,
}
async fn review_form_submission(
    State(state): State<AppState>,
    Path((guild_id, form_id, submission_id)): Path<(String, String, i64)>,
    headers: HeaderMap,
    Json(input): Json<ReviewFormSubmission>,
) -> Result<Json<FormSubmission>, ApiError> {
    validate_snowflake(&guild_id)?;
    validate_form_id(&form_id)?;
    if form_id != "moderation-appeals"
        || submission_id <= 0
        || !matches!(input.status.as_str(), "approved" | "rejected" | "closed")
        || input
            .note
            .as_ref()
            .is_some_and(|note| note.chars().count() > 1000)
    {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid appeal decision".into(),
        ));
    }
    let actor = require_appeal_reviewer(&state, &headers, &guild_id).await?;
    let before = state
        .infrastructure
        .form_submission(&guild_id, &form_id, submission_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "appeal not found".into()))?;
    let mut event = NewAuditEvent::guild(
        &guild_id,
        &actor.discord_user_id,
        "moderation_appeal.reviewed",
    );
    event.source = "dashboard_api".into();
    event.target_type = "moderation_appeal".into();
    event.target_id = Some(submission_id.to_string());
    event.before_state = Some(serde_json::json!({"status":before.status}));
    event.after_state = Some(serde_json::json!({"status":input.status,"note":input.note}));
    event.request_id = header_value(&headers, "x-request-id").map(str::to_owned);
    let item = state
        .infrastructure
        .review_form_submission_audited(
            &guild_id,
            &form_id,
            submission_id,
            &input.status,
            &actor.discord_user_id,
            input.note.as_deref(),
            &event,
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "appeal not found".into()))?;
    Ok(Json(item))
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
    } else if action.starts_with("suggestion") {
        "suggestion"
    } else if action.starts_with("giveaway") {
        "giveaway"
    } else if action.starts_with("fleet") {
        "guild_fleet"
    } else if action.starts_with("analytics") {
        "analytics_report"
    } else if action.starts_with("temporary_voice") {
        "voice_channel"
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
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown requested; draining API requests");
}

#[cfg(test)]
mod resource_diagnostic_tests {
    use super::*;
    use axum::body::Body;

    #[test]
    fn collects_nested_channel_and_role_references() {
        let value = serde_json::json!({
            "guild_id": "1",
            "modules": {
                "welcome": { "channel_id": "10" },
                "automod": { "ignored_channel_ids": ["11"], "ignored_role_ids": ["20"] },
                "tickets": { "panels": [{ "category_id": "12", "support_role_ids": ["21"] }] }
            }
        });
        let mut output = Vec::new();
        collect_resource_references(&value, String::new(), &mut output);
        assert!(
            output
                .iter()
                .any(|(_, kind, id)| *kind == "channel" && id == "10")
        );
        assert!(
            output
                .iter()
                .any(|(_, kind, id)| *kind == "channel" && id == "12")
        );
        assert!(
            output
                .iter()
                .any(|(_, kind, id)| *kind == "role" && id == "21")
        );
        assert!(!output.iter().any(|(_, _, id)| id == "1"));
    }

    #[test]
    fn classifies_common_discord_failures() {
        assert!(discord_resource_message("roles", "403 Missing Access").contains("cannot read"));
        assert!(discord_resource_message("channels", "429").contains("rate-limiting"));
        assert!(discord_resource_message("channels", "404").contains("removed"));
    }

    #[test]
    fn rejects_cross_site_mutations_but_allows_reads() {
        let mutation = Request::builder()
            .method(Method::PUT)
            .uri("/v1/guilds/1/config")
            .header("origin", "https://evil.example")
            .body(Body::empty())
            .unwrap();
        assert!(reject_cross_site_mutation(&mutation, "https://orivane.example").is_some());
        let read = Request::builder()
            .method(Method::GET)
            .uri("/v1/guilds/1/config")
            .header("origin", "https://evil.example")
            .body(Body::empty())
            .unwrap();
        assert!(reject_cross_site_mutation(&read, "https://orivane.example").is_none());
    }

    #[test]
    fn rate_limits_mutations_independently_from_reads() {
        let limiter = RateLimiter::default();
        for _ in 0..120 {
            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/activity")
                .header("cookie", "session=test")
                .body(Body::empty())
                .unwrap();
            assert_eq!(limiter.retry_after(&request), None);
        }
        let limited = Request::builder()
            .method(Method::POST)
            .uri("/v1/activity")
            .header("cookie", "session=test")
            .body(Body::empty())
            .unwrap();
        assert!(limiter.retry_after(&limited).is_some());
        let read = Request::builder()
            .method(Method::GET)
            .uri("/v1/health")
            .header("cookie", "session=test")
            .body(Body::empty())
            .unwrap();
        assert_eq!(limiter.retry_after(&read), None);
    }
}
