use std::{
    collections::BTreeMap,
    env,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, Request, StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use reqwest::{Client, Method, Url};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpServerConfig, stdio,
        streamable_http_server::{
            session::local::LocalSessionManager, tower::StreamableHttpService,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Serialize)]
struct Endpoint {
    method: &'static str,
    path: &'static str,
    description: &'static str,
}

impl Endpoint {
    const fn read(path: &'static str, description: &'static str) -> Self {
        Self {
            method: "GET",
            path,
            description,
        }
    }

    const fn write(method: &'static str, path: &'static str, description: &'static str) -> Self {
        Self {
            method,
            path,
            description,
        }
    }

    const fn mutating(self) -> bool {
        !matches!(self.method.as_bytes(), b"GET")
    }
}

const ENDPOINTS: &[Endpoint] = &[
    Endpoint::read(
        "/v1/me/mcp-context",
        "Identify the signed-in Discord user and server attached to this access token",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/audit",
        "List a server's audit timeline",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/audit/{event_id}",
        "Get a server audit event",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/config", "Get server configuration"),
    Endpoint::write(
        "PUT",
        "/v1/guilds/{guild_id}/config",
        "Replace server configuration",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/channels", "List server channels"),
    Endpoint::read("/v1/guilds/{guild_id}/roles", "List server roles"),
    Endpoint::read(
        "/v1/guilds/{guild_id}/resources/diagnostics",
        "Diagnose Discord resource access",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/stats", "Get server statistics"),
    Endpoint::read("/v1/guilds/{guild_id}/members", "List server members"),
    Endpoint::read(
        "/v1/guilds/{guild_id}/members/{member_id}",
        "Get a server member",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/analytics/commands",
        "Get server command analytics",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/appeals", "List moderation appeals"),
    Endpoint::read(
        "/v1/guilds/{guild_id}/appeals/{appeal_id}",
        "Get a moderation appeal",
    ),
    Endpoint::write(
        "PATCH",
        "/v1/guilds/{guild_id}/appeals/{appeal_id}",
        "Review a moderation appeal",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/cases", "List moderation cases"),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/cases",
        "Create a moderation case",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/cases/{case_id}",
        "Get a moderation case",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/polls", "List polls"),
    Endpoint::read("/v1/guilds/{guild_id}/polls/{poll_id}", "Get a poll"),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/polls/{poll_id}/close",
        "Close a poll",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/invites/attributions",
        "List invite attributions",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/invites/leaderboard",
        "Get the invite leaderboard",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/verification/attempts",
        "List verification attempts",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/afk", "List AFK statuses"),
    Endpoint::write(
        "DELETE",
        "/v1/guilds/{guild_id}/afk/{user_id}",
        "Clear a member's AFK status",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/counters/runs",
        "List counter update runs",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/voice-links/events",
        "List voice-text link events",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/temporary-voice",
        "List temporary voice channels",
    ),
    Endpoint::write(
        "DELETE",
        "/v1/guilds/{guild_id}/temporary-voice/{channel_id}",
        "Delete a temporary voice channel",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/economy/accounts",
        "List economy accounts",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/economy/transactions",
        "List economy transactions",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/ticket-panels", "List ticket panels"),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/ticket-panels",
        "Create a ticket panel",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/ticket-panels/{panel_id}",
        "Get a ticket panel",
    ),
    Endpoint::write(
        "PUT",
        "/v1/guilds/{guild_id}/ticket-panels/{panel_id}",
        "Update a ticket panel",
    ),
    Endpoint::write(
        "DELETE",
        "/v1/guilds/{guild_id}/ticket-panels/{panel_id}",
        "Delete a ticket panel",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/ticket-panels/{panel_id}/deploy",
        "Deploy a ticket panel",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/tickets", "List tickets"),
    Endpoint::read(
        "/v1/guilds/{guild_id}/tickets/analytics",
        "Get ticket analytics",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/tickets/{ticket_id}", "Get a ticket"),
    Endpoint::write(
        "PATCH",
        "/v1/guilds/{guild_id}/tickets/{ticket_id}",
        "Update ticket metadata",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/events",
        "List ticket events",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/claim",
        "Claim a ticket",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/close",
        "Close a ticket",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/reopen",
        "Reopen a ticket",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/transfer",
        "Transfer a ticket",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/notes",
        "List internal ticket notes",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/notes",
        "Create an internal ticket note",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/tickets/{ticket_id}/transcript",
        "Download a ticket transcript",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/reaction-role-panels/{panel_id}/deploy",
        "Deploy a reaction-role panel",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/suggestions", "List suggestions"),
    Endpoint::read(
        "/v1/guilds/{guild_id}/suggestions/{suggestion_id}",
        "Get a suggestion",
    ),
    Endpoint::write(
        "PATCH",
        "/v1/guilds/{guild_id}/suggestions/{suggestion_id}",
        "Update suggestion status",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/giveaways", "List giveaways"),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/giveaways",
        "Create a giveaway",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/giveaways/{giveaway_id}",
        "Get a giveaway",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/giveaways/{giveaway_id}/entries",
        "List giveaway entries",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/giveaways/{giveaway_id}/end",
        "End a giveaway",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/giveaways/{giveaway_id}/reroll",
        "Reroll giveaway winners",
    ),
    Endpoint::read("/v1/guilds/{guild_id}/reminders", "List reminders"),
    Endpoint::read(
        "/v1/guilds/{guild_id}/reminders/{reminder_id}",
        "Get a reminder",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/reminders/{reminder_id}/cancel",
        "Cancel a reminder",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/message-deletions",
        "List deleted-message records",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/message-deletions/{channel_id}/{message_id}",
        "Get a deleted-message record",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/embeds",
        "List managed embed templates",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/embeds",
        "Create a managed embed template",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/embeds/{template_id}",
        "Get a managed embed template",
    ),
    Endpoint::write(
        "PUT",
        "/v1/guilds/{guild_id}/embeds/{template_id}",
        "Update a managed embed template",
    ),
    Endpoint::write(
        "DELETE",
        "/v1/guilds/{guild_id}/embeds/{template_id}",
        "Delete a managed embed template",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/embed-deliveries",
        "List managed embed deliveries",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/embeds/{template_id}/send",
        "Send a managed embed",
    ),
    Endpoint::write(
        "POST",
        "/v1/guilds/{guild_id}/embeds/{template_id}/deliveries/{delivery_id}/sync",
        "Synchronize a managed embed delivery",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/forms/{form_id}/submissions",
        "List form submissions",
    ),
    Endpoint::read(
        "/v1/guilds/{guild_id}/forms/{form_id}/submissions/{submission_id}",
        "Get a form submission",
    ),
    Endpoint::write(
        "PATCH",
        "/v1/guilds/{guild_id}/forms/{form_id}/submissions/{submission_id}",
        "Review a form submission",
    ),
];

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReadRequest {
    /// An allowlisted Orivane API path beginning with /v1/.
    path: String,
    /// Optional query parameters. Values are URL encoded by the MCP server.
    #[serde(default)]
    query: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MutationRequest {
    /// HTTP mutation method: POST, PUT, PATCH, or DELETE.
    method: String,
    /// An allowlisted Orivane API path beginning with /v1/.
    path: String,
    /// JSON request body. Omit for bodyless operations.
    #[serde(default)]
    body: Option<Value>,
}

#[derive(Clone)]
struct OrivaneMcp {
    api_base: Url,
    stdio_token: Option<Arc<str>>,
    client: Client,
}

#[tool_router]
impl OrivaneMcp {
    fn new(api_base: Url, stdio_token: Option<Arc<str>>) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("orivane-mcp/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("build MCP API client")?;
        Ok(Self {
            api_base,
            stdio_token,
            client,
        })
    }

    #[tool(
        description = "List the operations available for the connected Discord server, including whether each one changes state",
        annotations(
            title = "List server capabilities",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn server_capabilities(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::structured(endpoint_catalog()))
    }

    #[tool(
        description = "Read data for the Discord server attached to this connection. Use server_capabilities first to discover supported paths",
        annotations(
            title = "Read server data",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn server_read(
        &self,
        Parameters(request): Parameters<ReadRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let token = self.credential(&context)?;
        self.execute(Method::GET, &request.path, request.query, None, &token)
            .await
    }

    #[tool(
        description = "Change configuration or perform an operation for the Discord server attached to this connection. Requires explicit user approval",
        annotations(
            title = "Change server state",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn server_change(
        &self,
        Parameters(request): Parameters<MutationRequest>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let method = request
            .method
            .to_ascii_uppercase()
            .parse::<Method>()
            .map_err(|_| McpError::invalid_params("method is invalid", None))?;
        if !matches!(
            method,
            Method::POST | Method::PUT | Method::PATCH | Method::DELETE
        ) {
            return Err(McpError::invalid_params(
                "method must be POST, PUT, PATCH, or DELETE",
                None,
            ));
        }
        let token = self.credential(&context)?;
        self.execute(method, &request.path, BTreeMap::new(), request.body, &token)
            .await
    }

    async fn execute(
        &self,
        method: Method,
        path: &str,
        query: BTreeMap<String, String>,
        body: Option<Value>,
        token: &str,
    ) -> Result<CallToolResult, McpError> {
        validate_path(path)?;
        let endpoint = find_endpoint(method.as_str(), path).ok_or_else(|| {
            McpError::invalid_params(
                format!(
                    "{} {path} is not an allowlisted Orivane API endpoint",
                    method.as_str()
                ),
                Some(json!({ "method": method.as_str(), "path": path })),
            )
        })?;
        if method == Method::GET && endpoint.mutating() {
            return Err(McpError::invalid_params(
                "mutation requires server_change",
                None,
            ));
        }

        let mut url = self
            .api_base
            .join(path.trim_start_matches('/'))
            .map_err(|error| McpError::invalid_params(error.to_string(), None))?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        let request_id = format!("mcp-{}", unique_request_id());
        let started = Instant::now();
        let mut request = self
            .client
            .request(method.clone(), url)
            .bearer_auth(token)
            .header("x-request-id", &request_id);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(|error| {
            tracing::warn!(
                %request_id,
                method = %method,
                path,
                latency_ms = started.elapsed().as_millis(),
                error = %error,
                "MCP API request failed"
            );
            McpError::internal_error(format!("Orivane API request failed: {error}"), None)
        })?;
        let status = response.status();
        let response_request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or(&request_id)
            .to_owned();
        tracing::info!(
            request_id = %response_request_id,
            method = %method,
            path,
            status = status.as_u16(),
            latency_ms = started.elapsed().as_millis(),
            "MCP API request completed"
        );
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Ok(CallToolResult::structured_error(json!({
                "status": 502,
                "error": "Orivane API response exceeded the MCP size limit"
            })));
        }
        let bytes = response.bytes().await.map_err(|error| {
            McpError::internal_error(
                format!("could not read Orivane API response: {error}"),
                None,
            )
        })?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Ok(CallToolResult::structured_error(json!({
                "status": 502,
                "error": "Orivane API response exceeded the MCP size limit"
            })));
        }
        let body = serde_json::from_slice::<Value>(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
        let value = json!({
            "method": method.as_str(),
            "path": path,
            "status": status.as_u16(),
            "ok": status.is_success(),
            "content_type": content_type,
            "request_id": response_request_id,
            "body": body,
        });
        if status.is_success() {
            Ok(CallToolResult::structured(value))
        } else {
            Ok(CallToolResult::structured_error(value))
        }
    }
}

#[tool_handler]
impl ServerHandler for OrivaneMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::from_build_env())
        .with_instructions(
            "Orivane MCP acts for the signed-in Discord server owner or administrator. Access is limited to the single server attached to the revocable access token. Read orivane://server/context first, inspect current state before changing it, and use server_change only with explicit user approval."
                .to_owned(),
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                Resource::new(
                    "orivane://server/capabilities",
                    "Supported server capabilities",
                ),
                Resource::new(
                    "orivane://server/context",
                    "Authenticated Discord server context",
                ),
                Resource::new("orivane://server/config", "Current server configuration"),
            ],
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let value = match request.uri.as_str() {
            "orivane://server/capabilities" => endpoint_catalog(),
            "orivane://server/context" => self.resource_get("/v1/me/mcp-context", &context).await?,
            "orivane://server/config" => {
                let server = self.resource_get("/v1/me/mcp-context", &context).await?;
                let guild_id = server
                    .pointer("/body/guild_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        McpError::internal_error("server context omitted guild_id", None)
                    })?;
                self.resource_get(&format!("/v1/guilds/{guild_id}/config"), &context)
                    .await?
            }
            _ => {
                return Err(McpError::resource_not_found(
                    "unknown Orivane resource",
                    Some(json!({ "uri": request.uri })),
                ));
            }
        };
        let text = serde_json::to_string_pretty(&value)
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(text, request.uri)]).into())
    }
}

impl OrivaneMcp {
    fn credential(&self, context: &RequestContext<RoleServer>) -> Result<Arc<str>, McpError> {
        if let Some(parts) = context.extensions.get::<axum::http::request::Parts>()
            && let Some(token) = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
        {
            return Ok(Arc::from(token));
        }
        self.stdio_token.clone().ok_or_else(|| {
            McpError::invalid_request("a server-scoped Orivane access token is required", None)
        })
    }

    async fn resource_get(
        &self,
        path: &str,
        context: &RequestContext<RoleServer>,
    ) -> Result<Value, McpError> {
        let token = self.credential(context)?;
        let result = self
            .execute(Method::GET, path, BTreeMap::new(), None, &token)
            .await?;
        result
            .structured_content
            .ok_or_else(|| McpError::internal_error("API resource had no structured data", None))
    }
}

fn endpoint_catalog() -> Value {
    json!({
        "count": ENDPOINTS.len(),
        "endpoints": ENDPOINTS.iter().map(|endpoint| json!({
            "method": endpoint.method,
            "path": endpoint.path,
            "description": endpoint.description,
            "mutating": endpoint.mutating(),
        })).collect::<Vec<_>>()
    })
}

fn validate_path(path: &str) -> Result<(), McpError> {
    let lower = path.to_ascii_lowercase();
    if !path.starts_with("/v1/")
        || path.contains('?')
        || path.contains('#')
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|segment| segment == "." || segment == "..")
        || lower.contains("%2f")
        || lower.contains("%5c")
        || lower.contains("%2e")
    {
        return Err(McpError::invalid_params(
            "path must be a canonical allowlisted /v1/ path without query, fragment, traversal, or encoded separators",
            None,
        ));
    }
    Ok(())
}

fn find_endpoint(method: &str, path: &str) -> Option<&'static Endpoint> {
    ENDPOINTS
        .iter()
        .find(|endpoint| endpoint.method == method && template_matches(endpoint.path, path))
}

fn template_matches(template: &str, path: &str) -> bool {
    let template_parts = template.split('/').collect::<Vec<_>>();
    let path_parts = path.split('/').collect::<Vec<_>>();
    template_parts.len() == path_parts.len()
        && template_parts
            .iter()
            .zip(path_parts)
            .all(|(expected, actual)| {
                (!actual.is_empty() && expected.starts_with('{') && expected.ends_with('}'))
                    || expected == &actual
            })
}

fn unique_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}

#[derive(Clone)]
struct HttpAuth {
    allowed_origins: Arc<[String]>,
    api_base: Url,
    client: Client,
}

async fn authorize_mcp(
    State(auth): State<HttpAuth>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok())
        && !auth.allowed_origins.iter().any(|allowed| allowed == origin)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if !provided.starts_with("orv_") || provided.len() != 68 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let context_url = auth
        .api_base
        .join("v1/me/mcp-context")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let response = auth
        .client
        .get(context_url)
        .bearer_auth(provided)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}

fn config() -> Result<(Url, Option<Arc<str>>)> {
    let api_base = env::var("ORIVANE_API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3010/".to_owned())
        .parse::<Url>()
        .context("ORIVANE_API_BASE_URL must be a valid URL")?;
    if !matches!(api_base.scheme(), "http" | "https")
        || api_base.cannot_be_a_base()
        || api_base.username() != ""
        || api_base.password().is_some()
    {
        bail!("ORIVANE_API_BASE_URL must be an HTTP(S) base URL without credentials");
    }
    let token = env::var("ORIVANE_USER_TOKEN")
        .ok()
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            if !token.starts_with("orv_") || token.len() != 68 {
                bail!("ORIVANE_USER_TOKEN must be a server-scoped token created in the Orivane dashboard");
            }
            Ok::<Arc<str>, anyhow::Error>(Arc::from(token))
        })
        .transpose()?;
    Ok((api_base, token))
}

fn init_tracing(stdio_transport: bool) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,rmcp=warn".into());
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if stdio_transport {
        builder.with_writer(std::io::stderr).with_ansi(false).init();
    } else {
        builder.init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let transport = env::var("MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_owned());
    init_tracing(transport == "stdio");
    let (api_base, token) = config()?;
    if transport == "stdio" && token.is_none() {
        bail!("ORIVANE_USER_TOKEN is required for the stdio transport");
    }
    let service = OrivaneMcp::new(api_base.clone(), token)?;
    match transport.as_str() {
        "stdio" => {
            tracing::info!("Orivane MCP server starting over stdio");
            service.serve(stdio()).await?.waiting().await?;
        }
        "http" => {
            let bind = env::var("MCP_BIND")
                .unwrap_or_else(|_| "127.0.0.1:3012".to_owned())
                .parse::<SocketAddr>()
                .context("MCP_BIND must be a socket address")?;
            let allowed_origins = env::var("MCP_ALLOWED_ORIGINS")
                .or_else(|_| env::var("APP_ORIGIN"))
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let mcp_service: StreamableHttpService<OrivaneMcp, LocalSessionManager> =
                StreamableHttpService::new(
                    move || Ok(service.clone()),
                    LocalSessionManager::default().into(),
                    StreamableHttpServerConfig::default()
                        .with_legacy_session_mode(false)
                        .with_json_response(true),
                );
            let auth = HttpAuth {
                allowed_origins: allowed_origins.into(),
                api_base,
                client: Client::builder()
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(Duration::from_secs(10))
                    .redirect(reqwest::redirect::Policy::none())
                    .build()
                    .context("build MCP authentication client")?,
            };
            let protected = Router::new()
                .nest_service("/mcp", mcp_service)
                .layer(middleware::from_fn_with_state(auth, authorize_mcp));
            let app = Router::new()
                .route("/health", get(|| async { "ok" }))
                .merge(protected);
            let listener = tokio::net::TcpListener::bind(bind).await?;
            tracing::info!(%bind, "Orivane MCP server listening");
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
        }
        _ => bail!("MCP_TRANSPORT must be stdio or http"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_catalog_has_unique_method_path_pairs() {
        let mut pairs = std::collections::HashSet::new();
        for endpoint in ENDPOINTS {
            assert!(pairs.insert((endpoint.method, endpoint.path)));
        }
        assert!(ENDPOINTS.len() > 70);
    }

    #[test]
    fn endpoint_catalog_contains_only_real_customer_api_paths() {
        let api_source = include_str!("../../api/src/main.rs");
        let mut remaining = api_source;
        let mut api_paths = std::collections::HashSet::new();
        while let Some(route_start) = remaining.find(".route(") {
            remaining = &remaining[route_start + ".route(".len()..];
            let Some(quote_start) = remaining.find('"') else {
                break;
            };
            remaining = &remaining[quote_start + 1..];
            let Some(quote_end) = remaining.find('"') else {
                break;
            };
            let path = &remaining[..quote_end];
            if path.starts_with("/v1/") {
                api_paths.insert(path);
            }
            remaining = &remaining[quote_end + 1..];
        }

        for endpoint in ENDPOINTS {
            assert!(
                api_paths.contains(endpoint.path),
                "MCP path {} does not exist in the API",
                endpoint.path,
            );
        }
        for operator_path in [
            "/v1/audit",
            "/v1/health",
            "/v1/servers",
            "/v1/users",
            "/v1/analytics/commands",
            "/v1/commands/sync",
        ] {
            assert!(
                ENDPOINTS
                    .iter()
                    .all(|endpoint| endpoint.path != operator_path),
                "operator path {operator_path} leaked into the customer MCP allowlist"
            );
        }
    }

    #[test]
    fn endpoint_templates_match_ids_but_not_extra_segments() {
        assert!(find_endpoint("GET", "/v1/guilds/123/members/456").is_some());
        assert!(find_endpoint("GET", "/v1/guilds/123/members").is_some());
        assert!(find_endpoint("GET", "/v1/guilds/123/members/456/extra").is_none());
        assert!(find_endpoint("DELETE", "/v1/guilds/123/afk/456").is_some());
        assert!(find_endpoint("POST", "/v1/guilds/123/afk/456").is_none());
    }

    #[test]
    fn rejects_non_api_and_ambiguous_paths() {
        for path in [
            "/api/auth/session",
            "/metrics",
            "/v1/../metrics",
            "/v1/servers?limit=1",
            "/v1/%2e%2e/metrics",
            "/v1/guilds%2f123/config",
        ] {
            assert!(validate_path(path).is_err(), "accepted {path}");
        }
        assert!(validate_path("/v1/guilds/123/config").is_ok());
    }

    #[test]
    fn tool_router_exposes_read_write_and_discovery_tools() {
        let router = OrivaneMcp::tool_router();
        assert!(router.has_route("server_capabilities"));
        assert!(router.has_route("server_read"));
        assert!(router.has_route("server_change"));
        assert_eq!(router.list_all().len(), 3);

        let read = router
            .get("server_read")
            .and_then(|tool| tool.annotations.as_ref())
            .expect("read tool annotations");
        assert_eq!(read.read_only_hint, Some(true));
        assert_eq!(read.destructive_hint, Some(false));

        let mutation = router
            .get("server_change")
            .and_then(|tool| tool.annotations.as_ref())
            .expect("mutation tool annotations");
        assert_eq!(mutation.read_only_hint, Some(false));
        assert_eq!(mutation.destructive_hint, Some(true));
        assert_eq!(mutation.open_world_hint, Some(true));
    }
}
