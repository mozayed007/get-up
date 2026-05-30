use std::sync::Arc;

use anyhow::Result;
use rmcp::model::*;
use rmcp::transport::io::stdio;
use rmcp::{tool, tool_router, ServerHandler};
use rmcp::handler::server::wrapper::Parameters;
use serde::Deserialize;

use crate::config::Config;
use crate::routine::{self, OutputFormat, RoutineOptions, RoutineType, Section};

/// The MCP server state. Holds config shared across tool calls.
#[derive(Clone)]
pub struct GetUpServer {
    config: Arc<Config>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunRoutineParams {
    /// Routine type: "morning" or "night".
    #[serde(default)]
    pub routine_type: Option<String>,

    /// Include daily problems (default: true).
    #[serde(default)]
    pub include_problems: Option<bool>,

    /// Include running statistics (default: true).
    #[serde(default)]
    pub include_running: Option<bool>,

    /// Include on-this-day history (default: true).
    #[serde(default)]
    pub include_history: Option<bool>,

    /// Include motivational quote (default: true).
    #[serde(default)]
    pub include_quote: Option<bool>,

    /// Include year progress bar (default: true).
    #[serde(default)]
    pub include_year_progress: Option<bool>,

    /// Output format: "text", "json", or "xml" (default: "json").
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EmptyParams {}

impl GetUpServer {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    fn parse_options(params: &RunRoutineParams) -> RoutineOptions {
        let routine_type = params
            .routine_type
            .as_deref()
            .map(|s| match s.to_lowercase().as_str() {
                "night" => RoutineType::Night,
                _ => RoutineType::Morning,
            })
            .unwrap_or_default();

        let format = params
            .format
            .as_deref()
            .map(|s| match s.to_lowercase().as_str() {
                "json" => OutputFormat::Json,
                "xml" => OutputFormat::Xml,
                _ => OutputFormat::Text,
            })
            .unwrap_or(OutputFormat::Json);

        let mut sections = Vec::new();
        if params.include_problems.unwrap_or(true) {
            sections.push(Section::Problems);
        }
        if params.include_running.unwrap_or(true) {
            sections.push(Section::Running);
        }
        if params.include_history.unwrap_or(true) {
            sections.push(Section::History);
        }
        if params.include_quote.unwrap_or(true) {
            sections.push(Section::Quote);
        }
        if params.include_year_progress.unwrap_or(true) {
            sections.push(Section::YearProgress);
        }

        RoutineOptions {
            routine_type,
            sections,
            format,
        }
    }
}

#[tool_router]
impl GetUpServer {
    #[tool(
        name = "run_routine",
        description = "Run the full morning or night routine. Returns a complete RoutineResult with all requested sections (problems from LeetCode and Deep-ML, running stats, history, quote, year progress). Agents can customize which sections to include and the output format."
    )]
    async fn run_routine(
        &self,
        Parameters(params): Parameters<RunRoutineParams>,
    ) -> Result<String, String> {
        let options = Self::parse_options(&params);
        routine::run_routine(&self.config, &options)
            .await
            .map(|result| {
                let format = options.format;
                match format {
                    OutputFormat::Json => {
                        routine::to_json(&result).unwrap_or_else(|_| result.formatted_message.clone())
                    }
                    OutputFormat::Xml => {
                        routine::to_xml(&result).unwrap_or_else(|_| result.formatted_message.clone())
                    }
                    OutputFormat::Text => result.formatted_message,
                }
            })
            .map_err(|e| format!("Failed to run routine: {}", e))
    }

    #[tool(
        name = "get_problems",
        description = "Get today's problems from all platforms (LeetCode and Deep-ML). Returns a list of problems with platform, ID, title, slug, difficulty, URL, and whether it's the daily challenge. The difficulty is determined by the scheduler (weekdays: Easy/Medium, weekends: Medium/Hard)."
    )]
    async fn get_problems(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<String, String> {
        let options = RoutineOptions {
            sections: vec![Section::Problems],
            format: OutputFormat::Json,
            ..Default::default()
        };
        routine::run_routine(&self.config, &options)
            .await
            .map(|result| {
                if !result.problems.is_empty() {
                    serde_json::to_string_pretty(&result.problems)
                        .unwrap_or_else(|_| result.formatted_message.clone())
                } else {
                    r#"{"error": "No problems available"}"#.to_string()
                }
            })
            .map_err(|e| format!("Failed to get problems: {}", e))
    }

    #[tool(
        name = "get_quote",
        description = "Get a motivational quote. Returns the quote text, author, and source (api or fallback)."
    )]
    async fn get_quote(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<String, String> {
        let options = RoutineOptions {
            sections: vec![Section::Quote],
            format: OutputFormat::Json,
            ..Default::default()
        };
        routine::run_routine(&self.config, &options)
            .await
            .map(|result| {
                if let Some(ref q) = result.quote {
                    serde_json::to_string_pretty(q)
                        .unwrap_or_else(|_| result.formatted_message.clone())
                } else {
                    r#"{"error": "No quote available"}"#.to_string()
                }
            })
            .map_err(|e| format!("Failed to get quote: {}", e))
    }

    #[tool(
        name = "get_history",
        description = "Get on-this-day historical events from Wikipedia. Returns a list of events with year, text, URL, and age context."
    )]
    async fn get_history(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<String, String> {
        let options = RoutineOptions {
            sections: vec![Section::History],
            format: OutputFormat::Json,
            ..Default::default()
        };
        routine::run_routine(&self.config, &options)
            .await
            .map(|result| {
                if let Some(ref events) = result.history {
                    serde_json::to_string_pretty(events)
                        .unwrap_or_else(|_| result.formatted_message.clone())
                } else {
                    r#"{"error": "No history available"}"#.to_string()
                }
            })
            .map_err(|e| format!("Failed to get history: {}", e))
    }

    #[tool(
        name = "get_running_stats",
        description = "Get running statistics. Returns yesterday's distance, this month's total, and this year's total in kilometers."
    )]
    async fn get_running_stats(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<String, String> {
        let options = RoutineOptions {
            sections: vec![Section::Running],
            format: OutputFormat::Json,
            ..Default::default()
        };
        routine::run_routine(&self.config, &options)
            .await
            .map(|result| {
                if let Some(ref stats) = result.running {
                    serde_json::to_string_pretty(stats)
                        .unwrap_or_else(|_| result.formatted_message.clone())
                } else {
                    r#"{"error": "No running stats available"}"#.to_string()
                }
            })
            .map_err(|e| format!("Failed to get running stats: {}", e))
    }

    #[tool(
        name = "get_year_progress",
        description = "Get the current year progress. Returns the day of year, total days, percentage complete, and a visual progress bar."
    )]
    async fn get_year_progress(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<String, String> {
        let options = RoutineOptions {
            sections: vec![Section::YearProgress],
            format: OutputFormat::Json,
            ..Default::default()
        };
        routine::run_routine(&self.config, &options)
            .await
            .map(|result| {
                if let Some(ref yp) = result.year_progress {
                    serde_json::to_string_pretty(yp)
                        .unwrap_or_else(|_| result.formatted_message.clone())
                } else {
                    r#"{"error": "No year progress available"}"#.to_string()
                }
            })
            .map_err(|e| format!("Failed to get year progress: {}", e))
    }
}

#[rmcp::tool_handler]
impl ServerHandler for GetUpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::new("get-up", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "get-up is your personal AI morning/night routine engine. \
             It delivers daily problems from LeetCode and Deep-ML with scheduled difficulty. \
             Use run_routine to get the full daily briefing, or call individual tools \
             (get_problems, get_quote, get_history, get_running_stats, get_year_progress) \
             for specific data. All tools return structured JSON by default. \
             Customize run_routine with options for routine_type (morning/night), \
             which sections to include, and output format (text/json/xml)."
                .to_string(),
        )
    }
}

/// Start the MCP server in stdio mode.
pub async fn run_stdio(config: Config) -> Result<()> {
    let server = GetUpServer::new(config);
    let transport = stdio();
    let running = rmcp::serve_server(server, transport).await?;
    running.waiting().await?;
    Ok(())
}

/// Start the MCP server in HTTP/SSE mode on the given port.
pub async fn run_http(config: Config, port: u16) -> Result<()> {
    use rmcp::transport::streamable_http_server::tower::StreamableHttpService;
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

    let server = GetUpServer::new(config);
    let service: StreamableHttpService<GetUpServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(server.clone()),
            Default::default(),
            Default::default(),
        );

    let router = axum::Router::new().nest_service("/mcp", service);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("MCP HTTP server listening on http://{}", addr);
    println!("Endpoint: http://{}/mcp", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
