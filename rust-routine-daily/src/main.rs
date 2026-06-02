use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use routine_daily::config;
use routine_daily::providers::deepml::DeepMLProvider;
use routine_daily::providers::leetcode::LeetCodeProvider;
use routine_daily::notification::{DiscordNotifier, Notifier, TelegramNotifier};
use routine_daily::routine::{self, OutputFormat, RoutineOptions, RoutineType};
use routine_daily::utils::append_line;

const LEETCODE_EASY_FILE: &str = "data/leetcode_easy.txt";
const LEETCODE_MEDIUM_FILE: &str = "data/leetcode_medium.txt";
const LEETCODE_HARD_FILE: &str = "data/leetcode_hard.txt";
const DEEPML_FILE: &str = "data/deepml_problems.txt";
const USED_FILE: &str = "data/used_problems.txt";

#[derive(Parser, Debug)]
#[command(name = "routine-daily")]
#[command(
    about = "A CLI tool and MCP server for daily motivational messages with LeetCode and Deep-ML problems"
)]
#[command(version)]
struct Args {
    #[arg(
        long,
        help = "Fetch and save all LeetCode problems (Easy, Medium, Hard) to data/leetcode_*.txt"
    )]
    fetch_leetcode: bool,

    #[arg(
        long,
        help = "Fetch and save all EASY problems to data/leetcode_easy.txt (legacy, use --fetch-leetcode)"
    )]
    fetch_easy: bool,

    #[arg(
        long,
        help = "Sync Deep-ML problems from GitHub to data/deepml_problems.txt"
    )]
    sync_deepml: bool,

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

    // Handle fetch-leetcode
    if args.fetch_leetcode {
        let provider = LeetCodeProvider::new(&config);
        println!("Fetching LeetCode problems...");
        provider
            .fetch_easy_list(LEETCODE_EASY_FILE)
            .await
            .context("Failed to fetch EASY problems")?;
        println!("EASY problems saved to {}", LEETCODE_EASY_FILE);
        provider
            .fetch_medium_list(LEETCODE_MEDIUM_FILE)
            .await
            .context("Failed to fetch MEDIUM problems")?;
        println!("MEDIUM problems saved to {}", LEETCODE_MEDIUM_FILE);
        provider
            .fetch_hard_list(LEETCODE_HARD_FILE)
            .await
            .context("Failed to fetch HARD problems")?;
        println!("HARD problems saved to {}", LEETCODE_HARD_FILE);
        return Ok(());
    }

    // Handle fetch-easy (legacy)
    if args.fetch_easy {
        let provider = LeetCodeProvider::new(&config);
        println!("Fetching EASY problems...");
        provider
            .fetch_easy_list(LEETCODE_EASY_FILE)
            .await
            .context("Failed to fetch EASY problems")?;
        println!("EASY problems saved to {}", LEETCODE_EASY_FILE);
        return Ok(());
    }

    // Handle sync-deepml
    if args.sync_deepml {
        let provider = DeepMLProvider::with_token(config.github_token.clone());
        println!("Syncing Deep-ML problems from GitHub...");
        provider
            .sync_problems(DEEPML_FILE)
            .await
            .context("Failed to sync Deep-ML problems")?;
        println!("Deep-ML problems saved to {}", DEEPML_FILE);
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

    // Mark problems as used
    for problem in &result.problems {
        append_line(USED_FILE, &problem.problem.slug).await?;
    }

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
        let now = routine_daily::utils::get_local_time(&config);
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
                routine_daily::mcp::run_stdio(_config).await?;
            }
            "http" | "sse" => {
                eprintln!("Starting MCP server in HTTP/SSE mode on port {}...", port);
                routine_daily::mcp::run_http(_config, port).await?;
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
