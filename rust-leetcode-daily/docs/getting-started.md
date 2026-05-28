# Getting Started Guide

This guide walks you through installing, configuring, and running LeetCode Daily for the first time.

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
git clone https://github.com/yourusername/rust-leetcode-daily.git
cd rust-leetcode-daily
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

# Build with Telegram support
cargo build --release --features telegram

# Build with Discord support
cargo build --release --features discord

# Build with all notification features
cargo build --release --features telegram,discord
```

## First-Time Setup

### Step 1: Fetch EASY Problems

Before running the daily message, fetch the list of EASY problems from LeetCode:

```bash
./target/release/leetcode-daily --fetch-easy
```

Expected output:
```
Fetching EASY problems...
EASY problems saved to data/leetcode_easy.txt
```

This creates `data/leetcode_easy.txt` containing all free EASY problems from LeetCode.

### Step 2: Verify Configuration

Run a dry-run to verify your configuration without posting:

```bash
./target/release/leetcode-daily --dry-run
```

Expected output (example):
```
Wake up time: 2024-01-15 07:30:00

Good morning!

Day 15 of the year.

15/365 (4.1%) ██░░░░░░░░░░░░░░░░░░

🟢 Today's LeetCode EASY:
[1. Two Sum](https://leetcode.com/problems/two-sum/)
Keep going! 🚀

🏃 Running Stats:
Yesterday: 5.23 km (1 sessions)
This month: 45.67 km (12 sessions)
This year: 45.67 km (12 sessions)

• 2020: [COVID-19 pandemic](https://en.wikipedia.org/wiki/COVID-19_pandemic) (I was 30 years old)
• 2001: [Wikipedia launched](https://en.wikipedia.org/wiki/Wikipedia) (I was 11 years old)

Today's Quote:
The only way to do great work is to love what you do.

—— Steve Jobs
```

## Quick Start Example

Here's a complete example showing a typical workflow:

```bash
# 1. Navigate to project directory
cd rust-leetcode-daily

# 2. Ensure .env is configured with at minimum:
#    GITHUB_TOKEN, REPO_OWNER, REPO_NAME, BIRTH_YEAR, TIMEZONE

# 3. Fetch EASY problems (first time only)
cargo run --release -- --fetch-easy

# 4. Test with dry-run
cargo run --release -- --dry-run

# 5. Post to GitHub (during wake-up hours: 3-9 AM)
cargo run --release -- --post

# 6. Send to Telegram (if configured)
cargo run --release --features telegram -- --telegram

# 7. Send to Discord (if configured)
cargo run --release --features discord -- --discord

# 8. Full deployment (all channels)
cargo run --release --features telegram,discord -- --post --telegram --discord
```

## Verifying Installation

### Check 1: Binary Exists

```bash
ls -la target/release/leetcode-daily
```

### Check 2: Help Output

```bash
./target/release/leetcode-daily --help
```

Expected output:
```
A CLI tool for daily motivational messages with LeetCode problems

Usage: leetcode-daily [OPTIONS]

Options:
      --fetch-easy  Fetch and save all EASY problems to data/leetcode_easy.txt
      --post        Post the generated message to GitHub Issue #1
      --telegram    Send notification via Telegram
      --discord     Send notification via Discord
      --dry-run     Print message to stdout without posting
  -h, --help        Print help
  -V, --version     Print version
```

### Check 3: Data Files

```bash
# Should exist after --fetch-easy
ls -la data/leetcode_easy.txt

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
