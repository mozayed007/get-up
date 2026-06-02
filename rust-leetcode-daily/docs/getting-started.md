# Getting Started Guide

This guide walks you through installing, configuring, and running the daily routine for the first time.

## Prerequisites

### Required

| Requirement | Version | Purpose |
|-------------|---------|---------|
| **Rust** | 1.70+ | Compile and run the application |
| **GitHub PAT** | Any | Post messages to GitHub Issues |
| **Git** | Any | Clone the repository |

### Optional

| Requirement | Purpose |
|-------------|---------|
| **Telegram Bot Token** | Send notifications to Telegram |
| **Discord Bot Token** | Send notifications to Discord |
| **Running Data** | Parquet file with running statistics |

### Checking Prerequisites

```bash
# Check Rust installation
rustc --version
cargo --version

# If Rust is not installed, install via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Check Git
git --version
```

## Installation

### Step 1: Clone the Repository

```bash
git clone https://github.com/mozayed007/get-up.git
cd get-up/rust-leetcode-daily
```

### Step 2: Create Data Directory

```bash
mkdir -p data
```

### Step 3: Configure Environment

```bash
# Copy the example environment file
cp .env.example .env

# Edit with your settings
nano .env  # or use your preferred editor
```

### Step 4: Build the Application

```bash
# Build with default features (GitHub only)
cargo build --release

# Build with MCP server support
cargo build --release --features mcp

# Build with Telegram support
cargo build --release --features telegram

# Build with Discord support
cargo build --release --features discord

# Build with all features
cargo build --release --features telegram,discord,mcp
```

## First-Time Setup

### Step 1: Fetch LeetCode Problems

Before running the daily message, fetch the problem lists from LeetCode:

```bash
./target/release/routine-daily --fetch-leetcode
```

Expected output:
```
Fetching LeetCode problems...
EASY problems saved to data/leetcode_easy.txt
MEDIUM problems saved to data/leetcode_medium.txt
HARD problems saved to data/leetcode_hard.txt
```

This creates three files containing all free problems from LeetCode by difficulty.

### Step 2: Sync Deep-ML Problems

Sync the Deep-ML problem list from GitHub:

```bash
./target/release/routine-daily --sync-deepml
```

Expected output:
```
Syncing Deep-ML problems from GitHub...
Deep-ML problems saved to data/deepml_problems.txt
```

### Step 3: Verify Configuration

Run a dry-run to verify your configuration without posting:

```bash
./target/release/routine-daily --dry-run
```

Expected output (example):
```
☀️ Good morning — 2024-01-15 07:30:00

Day 15 · 15/365 (4.1%) ██░░░░░░░░░░░░░░░░░░

📚 Today's Problems

🟢 LeetCode Easy: 1. Two Sum
https://leetcode.com/problems/two-sum/

🟡 Deep-ML Medium: Matrix-Vector Dot Product
https://deep-ml.com/problem/1

🏃 Yesterday: 5.23 km · This month: 45.67 km · This year: 45.67 km

📜 On this day:
• 2020: [COVID-19 pandemic](https://en.wikipedia.org/wiki/COVID-19_pandemic) (I was 30 years old)
• 2001: [Wikipedia launched](https://en.wikipedia.org/wiki/Wikipedia) (I was 11 years old)

💬 Today's Quote
The only way to do great work is to love what you do.

—— Steve Jobs
```

## Quick Start Example

Here's a complete example showing a typical workflow:

```bash
# 1. Navigate to project directory
cd get-up/rust-leetcode-daily

# 2. Ensure .env is configured with at minimum:
#    GITHUB_TOKEN, REPO_OWNER, REPO_NAME, BIRTH_YEAR, TIMEZONE

# 3. Fetch LeetCode problems (first time only)
cargo run --release -- --fetch-leetcode

# 4. Sync Deep-ML problems (first time only)
cargo run --release -- --sync-deepml

# 5. Test with dry-run
cargo run --release -- --dry-run

# 6. Post to GitHub (during wake-up hours: 3-9 AM)
cargo run --release -- --post

# 7. Send to Telegram (if configured)
cargo run --release --features telegram -- --telegram

# 8. Send to Discord (if configured)
cargo run --release --features discord -- --discord

# 9. Full deployment (all channels)
cargo run --release --features telegram,discord -- --post --telegram --discord
```

## Verifying Installation

### Check 1: Binary Exists

```bash
ls -la target/release/routine-daily
```

### Check 2: Help Output

```bash
./target/release/routine-daily --help
```

Expected output:
```
A CLI tool and MCP server for daily motivational messages with LeetCode and Deep-ML problems

Usage: routine-daily [OPTIONS]

Options:
      --fetch-leetcode  Fetch all LeetCode problems (Easy, Medium, Hard)
      --fetch-easy      Fetch only EASY problems (legacy)
      --sync-deepml     Sync Deep-ML problems from GitHub
      --post            Post to GitHub Issue #1
      --telegram        Send via Telegram
      --discord         Send via Discord
      --dry-run         Print without posting
      --json            Output JSON
      --xml             Output XML
      --night           Run night routine
  -h, --help            Print help
  -V, --version         Print version
```

### Check 3: Data Files

```bash
# Should exist after --fetch-leetcode
ls -la data/leetcode_easy.txt
ls -la data/leetcode_medium.txt
ls -la data/leetcode_hard.txt

# Should exist after --sync-deepml
ls -la data/deepml_problems.txt

# Will be created on first run
ls -la data/used_problems.txt
```

### Check 4: Environment Variables

```bash
# Quick validation (Linux/macOS)
source .env && echo "GITHUB_TOKEN is set: ${GITHUB_TOKEN:0:10}..."

# Windows PowerShell
Get-Content .env | Select-String "GITHUB_TOKEN"
```

## Next Steps

- **Configure notifications**: See [Configuration Guide](configuration.md)
- **Understand the codebase**: See [Architecture Overview](architecture.md)
- **Deploy to production**: See [Deployment Guide](deployment.md)
- **Troubleshoot issues**: See [FAQ & Troubleshooting](faq.md)
