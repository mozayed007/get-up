use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use leetcode_daily::config;
use leetcode_daily::leetcode::LeetCode;
use leetcode_daily::notification::{DiscordNotifier, Notifier, TelegramNotifier};
use leetcode_daily::routine::{self, OutputFormat, RoutineOptions, RoutineType};

const EASY_FILE: &str = "data/leetcode_easy.txt";

#[derive(Parser, Debug)]
#[command(name = "leetcode-daily")]
#[command(about = "A CLI tool and MCP server for daily motivational messages with LeetCode problems")]
#[command(version)]
struct Args {
    #[arg(
        long,
        help = "Fetch and save all EASY problems to data/leetcode_easy.txt"
    )]
    fetch_easy: bool,

    #[arg(long, help = "Post the generated message to GitHub Issue #1")]
    post: bool,

    #[arg(long, help = "Send notification via Telegram")]
    telegram: bool,

    #[arg(long, help = "Send notification via Discord")]
    discord: bool,

    #[arg(long, help = "Print message to stdout without posting")]
    dry_run: bool,

    #[arg(long, help = "Output structured JSON instead of formatted text")]
    json: bool,

    #[arg(long, help = "Output structured XML instead of formatted text")]
    xml: bool,

    #[arg(long, help = "Run night routine instead of morning")]
    night: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the MCP server for agent integration
    Mcp {
        #[arg(long, default_value = "stdio", help = "Transport: stdio or http")]
        transport: String,

        #[arg(long, default_value = "3000", help = "Port for HTTP transport")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::Config::load()?;

    // Handle MCP subcommand
    if let Some(cmd) = &args.command {
        match cmd {
            Commands::Mcp { transport, port } => {
                run_mcp(config, transport, *port).await?;
                return Ok(());
            }
        }
    }

    // Handle fetch-easy
    if args.fetch_easy {
        let leetcode = LeetCode::new(&config);
        println!("Fetching EASY problems...");
        leetcode
            .fetch_easy_list(EASY_FILE)
            .await
            .context("Failed to fetch EASY problems")?;
        println!("EASY problems saved to {}", EASY_FILE);
        return Ok(());
    }

    // Determine output format
    let format = if args.json {
        OutputFormat::Json
    } else if args.xml {
        OutputFormat::Xml
    } else {
        OutputFormat::Text
    };

    let routine_type = if args.night {
        RoutineType::Night
    } else {
        RoutineType::Morning
    };

    let options = RoutineOptions {
        routine_type,
        format,
        ..RoutineOptions::default()
    };

    let result = routine::run_routine(&config, &options)
        .await
        .context("Failed to run routine")?;

    // Output based on format
    match format {
        OutputFormat::Json => {
            println!("{}", routine::to_json(&result)?);
        }
        OutputFormat::Xml => {
            println!("{}", routine::to_xml(&result)?);
        }
        OutputFormat::Text => {
            println!("{}", result.formatted_message);
        }
    }

    // Handle posting and notifications (only for text format, backward compat)
    if format == OutputFormat::Text {
        let now = leetcode_daily::utils::get_local_time(&config);
        let current_hour = chrono::Timelike::hour(&now);
        let is_early_wake_up = (3..=9).contains(&current_hour);

        if args.dry_run {
            return Ok(());
        }

        let has_explicit_targets = args.post || args.telegram || args.discord;

        if !is_early_wake_up && !args.night && !has_explicit_targets {
            println!("You wake up late");
            return Ok(());
        }

        if args.post {
            match octocrab::Octocrab::builder()
                .personal_token(config.github_token.clone())
                .build()
            {
                Ok(crab) => {
                    match crab
                        .issues(&config.repo_owner, &config.repo_name)
                        .create_comment(1, &result.formatted_message)
                        .await
                    {
                        Ok(_) => println!("Posted to GitHub Issue #1"),
                        Err(e) => eprintln!("Failed to post to GitHub: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to create GitHub client: {}", e),
            }
        }

        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();
        if args.telegram {
            if let Some(n) = TelegramNotifier::from_config(&config) {
                notifiers.push(Box::new(n));
            }
        }
        if args.discord {
            if let Some(n) = DiscordNotifier::from_config(&config) {
                notifiers.push(Box::new(n));
            }
        }

        for notifier in &notifiers {
            match notifier.send_message(&result.formatted_message).await {
                Ok(_) => println!("Sent notification via {}", notifier.name()),
                Err(e) => eprintln!("Failed to send {}: {}", notifier.name(), e),
            }
        }
    }

    Ok(())
}

#[allow(clippy::needless_return)]
async fn run_mcp(_config: config::Config, transport: &str, port: u16) -> Result<()> {
    #[cfg(feature = "mcp")]
    {
        match transport {
            "stdio" => {
                eprintln!("Starting MCP server in stdio mode...");
                leetcode_daily::mcp::run_stdio(_config).await?;
            }
            "http" | "sse" => {
                eprintln!("Starting MCP server in HTTP/SSE mode on port {}...", port);
                leetcode_daily::mcp::run_http(_config, port).await?;
            }
            other => {
                eprintln!("Unknown transport: {}. Use 'stdio' or 'http'.", other);
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    #[cfg(not(feature = "mcp"))]
    {
        let _ = (transport, port);
        eprintln!("MCP server requires the 'mcp' feature. Rebuild with:");
        eprintln!("  cargo build --release --features mcp");
        std::process::exit(1);
    }
}
