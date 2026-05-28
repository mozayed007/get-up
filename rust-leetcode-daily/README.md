# Rust LeetCode Daily

A Rust CLI tool that fetches daily LeetCode EASY problems and posts motivational messages to GitHub Issues with optional Telegram and Discord notifications.

## Features

- Fetches daily LeetCode EASY problems
- Generates motivational messages with quotes, history events, running stats
- Posts to GitHub Issues
- Telegram and Discord notification support (feature flags)

## Example Output

```
Wake up time: 2026-02-16 08:30:00

Good morning!

Day 47 of the year.

12.8% ███████░░░░░░░░░░░

🟢 Today's LeetCode EASY:
[1. Two Sum](https://leetcode.com/problems/two-sum/)

🏃 Running Stats:
Yesterday: 5.23 km (1 sessions)
This month: 45.67 km (8 sessions)
This year: 123.45 km (22 sessions)

• 2020: [Some historical event](https://en.wikipedia.org/wiki/...) (I was 30 years old)

Today's Quote:
The only way to do great work is to love what you do.

—— Steve Jobs
```

## Prerequisites

- Rust 1.75+
- GitHub Personal Access Token
- Optional: Telegram/Discord bot tokens

## Installation

```bash
# Clone repo
git clone https://github.com/yourusername/rust-leetcode-daily.git
cd rust-leetcode-daily

# Copy .env.example to .env and fill values
cp .env.example .env

# Build
cargo build --release

# With features
cargo build --release --features telegram,discord
```

## Usage

```bash
# First time setup - fetch easy problems
leetcode-daily --fetch-easy

# Preview message without posting
leetcode-daily --dry-run

# Post to GitHub
leetcode-daily --post

# Full run with all notifications
leetcode-daily --post --telegram --discord
```

## Cron Setup

```cron
# Run daily at 8:00 AM
0 8 * * * /path/to/leetcode-daily --post --telegram --discord
```

## Project Structure

```
rust-leetcode-daily/
├── Cargo.toml
├── README.md
├── .env.example
├── .gitignore
├── .github/
│   └── workflows/
│       ├── daily.yml
│       └── ci.yml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── utils.rs
│   ├── leetcode.rs
│   ├── api.rs
│   ├── message.rs
│   └── notification/
│       ├── mod.rs
│       ├── telegram.rs
│       └── discord.rs
└── data/
    ├── leetcode_easy.txt
    └── used_problems.txt
```

## License

MIT
