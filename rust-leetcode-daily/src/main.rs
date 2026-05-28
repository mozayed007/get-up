mod api;
mod config;
mod leetcode;
mod message;
mod notification;
mod utils;

use anyhow::{Context, Result};
use chrono::{Datelike, Timelike};
use clap::Parser;
use leetcode::LeetCode;
use notification::{DiscordNotifier, Notifier, TelegramNotifier};
use utils::append_line;

const EASY_FILE: &str = "data/leetcode_easy.txt";
const USED_FILE: &str = "data/used_problems.txt";
const PARQUET_FILE: &str = "data/running.parquet";

#[derive(Parser, Debug)]
#[command(name = "leetcode-daily")]
#[command(about = "A CLI tool for daily motivational messages with LeetCode problems")]
struct Args {
    #[arg(long, help = "Fetch and save all EASY problems to data/leetcode_easy.txt")]
    fetch_easy: bool,

    #[arg(long, help = "Post the generated message to GitHub Issue #1")]
    post: bool,

    #[arg(long, help = "Send notification via Telegram")]
    telegram: bool,

    #[arg(long, help = "Send notification via Discord")]
    discord: bool,

    #[arg(long, help = "Print message to stdout without posting")]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::Config::load()?;
    let client = reqwest::Client::new();
    let leetcode = LeetCode::new(&config);
    let now = utils::get_local_time(&config);

    if args.fetch_easy {
        println!("Fetching EASY problems...");
        leetcode
            .fetch_easy_list(EASY_FILE)
            .await
            .context("Failed to fetch EASY problems")?;
        println!("EASY problems saved to {}", EASY_FILE);
        return Ok(());
    }

    let current_hour = now.hour();
    let is_early_wake_up = (3..=9).contains(&current_hour);

    let octocrab = if args.post && !args.dry_run {
        let crab = octocrab::Octocrab::builder()
            .personal_token(config.github_token.clone())
            .build()?;

        let today = now.format("%Y-%m-%d").to_string();
        let issue = crab
            .issues(&config.repo_owner, &config.repo_name)
            .get(1)
            .await?;

        if issue.comments > 0 {
            let mut page = crab
                .issues(&config.repo_owner, &config.repo_name)
                .list_comments(1)
                .per_page(100)
                .send()
                .await?;

            loop {
                for comment in &page.items {
                    if comment.body.as_ref().is_some_and(|b| b.contains(&today)) {
                        println!("Already posted today, skipping...");
                        return Ok(());
                    }
                }
                if page.next.is_some() {
                    page = crab.get_page(&page.next).await?
                        .ok_or_else(|| anyhow::anyhow!("Expected next page"))?;
                } else {
                    break;
                }
            }
        }

        Some(crab)
    } else {
        None
    };

    let get_up_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let day_of_year = utils::get_day_of_year(&now);
    let year_progress = utils::get_year_progress(&now);

    let quote = api::fetch_quote(&client)
        .await
        .context("Failed to fetch quote")?;

    let history = api::fetch_history(&client, config.birth_year, now.year(), now.month(), now.day())
        .await
        .context("Failed to fetch history")?;

    let running = api::fetch_running_stats(PARQUET_FILE, now.date_naive())
        .await
        .context("Failed to fetch running stats")?;

    let problem = leetcode
        .get_today_problem(EASY_FILE, USED_FILE)
        .await
        .context("Failed to get today's problem")?;

    let leetcode_info = message::format_problem_message(&problem, &config.leetcode_variant);
    let running_info = message::format_running(&running);
    let history_today = message::format_history(&history);

    let message = message::build_message(
        &get_up_time,
        day_of_year,
        &year_progress,
        &leetcode_info,
        &running_info,
        &history_today,
        &quote,
    );

    if args.dry_run {
        println!("{}", message);
        return Ok(());
    }

    if !is_early_wake_up {
        println!("You wake up late");
        println!("\n{}", message);
        return Ok(());
    }

    if let Some(crab) = &octocrab {
        crab.issues(&config.repo_owner, &config.repo_name)
            .create_comment(1, &message)
            .await?;

        append_line(USED_FILE, &problem.slug).await?;
        println!("Posted to GitHub Issue #1");
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
        notifier.send_message(&message).await?;
        println!("Sent notification via {}", notifier.name());
    }

    Ok(())
}

