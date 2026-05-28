# Get Up ☀️

A Rust CLI that sends you a daily motivational message every morning — LeetCode EASY problem, on-this-day history with your age, running stats, and an inspirational quote — straight to **GitHub Issues**, **Telegram DM**, and **Discord DM**.

```
☀️ Good morning — 2026-05-28 08:13:22

Day 148 · 148/365 (40.5%) ████████░░░░░░░░░░░░

🟢 LeetCode EASY: 1637. Widest Vertical Area ...
https://leetcode.com/problems/widest-vertical-area-between-two-points-containing-no-points/

🏃 Yesterday: 5.23 km · This month: 45.6 km · This year: 312.4 km

📜 On this day:
• 2017: [Takuma Sato wins the Indy 500] (you were 17)
• 2016: [Harambe incident] (you were 16)

💬 Today's Quote
The only way to do great work is to love what you do.

—— Steve Jobs
```

## Features

- **Time-aware greeting** — `☀️ Good morning` / `⛅ Good afternoon` / `🌙 Good evening`
- **Smart LeetCode picker** — seeded random selection so you see each problem once before repeating
- **Your age in history** — events from Wikipedia on this day, tagged with your age at the time
- **Running stats** — reads your Strava or tracking data (parquet or CSV)
- **Triple notification** — GitHub Issue comment + Telegram DM + Discord DM
- **GitHub Actions cron** — deployed via CI, fires daily at 3-9 AM Cairo time

## Quick Start

```bash
# 1. clone
git clone https://github.com/mozayed007/get-up.git
cd get-up/rust-leetcode-daily

# 2. set up secrets
cp .env.example .env
# fill in: GITHUB_TOKEN, TELEGRAM_TOKEN, DISCORD_TOKEN, BIRTH_YEAR, TIMEZONE

# 3. first-time fetch
cargo run -- --fetch-easy

# 4. preview
cargo run --features telegram,discord -- --dry-run
```

## CLI

| Flag | Does |
|------|------|
| `--fetch-easy` | Download all LeetCode EASY problems to `data/leetcode_easy.txt` |
| `--dry-run` | Print message to terminal, don't post anywhere |
| `--post` | Post the message as a comment on GitHub Issue #1 |
| `--telegram` | Send via Telegram DM |
| `--discord` | Send via Discord DM |

## Running Stats

Place a file at `data/running.parquet` (or `.csv`) with two columns:

| Column | Example |
|--------|---------|
| `date` | `2026-05-27` |
| `distance_km` | `5.23` |

**On Android** — install [OpenTracks](https://f-droid.org/packages/de.dennisguse.opentracks/) (free, open-source) or any app that exports CSV, and drop the file in `data/`.

## GitHub Actions

The repo includes two workflows:

- **CI** — runs `cargo test --all-features` + `clippy` + `fmt` on every push
- **Daily Run** — fires at midnight UTC, posts to GitHub Issue + Telegram + Discord

Set these repo secrets:

```
GITHUB_TOKEN        ✓ (auto-injected)
TELEGRAM_TOKEN      ✓
TELEGRAM_CHAT_ID    ✓
DISCORD_TOKEN       ✓
DISCORD_USER_ID     ✓
BIRTH_YEAR          ✓
TIMEZONE            ✓ (Africa/Cairo)
```

## Project Layout

```
get-up/
├── .github/workflows/   ← CI + daily cron
├── rust-leetcode-daily/
│   ├── src/
│   │   ├── main.rs         ← entry point, orchestration
│   │   ├── config.rs       ← env var loading
│   │   ├── leetcode.rs     ← LeetCode REST + GraphQL client
│   │   ├── api.rs          ← quotes, history, running stats
│   │   ├── message.rs      ← message template + formatters
│   │   ├── utils.rs        ← time helpers
│   │   └── notification/   ← Telegram + Discord senders
│   ├── data/               ← problem lists, used problems, running stats
│   └── Cargo.toml
└── README.md
```

## Built With

- **Rust** — `reqwest`, `polars`, `serenity`, `teloxide`, `clap`, `chrono`
- **GitHub Actions** — auto-deploy via cron
- **LeetCode API** — problem fetching
- **Wikipedia API** — on-this-day history
