# Configuration Guide

This guide explains all configuration options for LeetCode Daily.

## Environment Variables

Configuration is managed through environment variables loaded from a `.env` file. Copy `.env.example` to `.env` and customize.

### Core Configuration (Required)

| Variable | Description | Example |
|----------|-------------|---------|
| `GITHUB_TOKEN` | GitHub Personal Access Token | `ghp_xxxxxxxxxxxx` |
| `REPO_OWNER` | GitHub repository owner | `yourusername` |
| `REPO_NAME` | GitHub repository name | `leetcode-daily` |
| `BIRTH_YEAR` | Your birth year (for age calculations) | `1990` |
| `TIMEZONE` | IANA timezone identifier | `America/New_York` |

### LeetCode Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `LEETCODE_ENDPOINT` | LeetCode GraphQL API endpoint | `https://leetcode.com/graphql/` |
| `LEETCODE_VARIANT` | Platform variant (`com` or `cn`) | `com` |

#### LeetCode Endpoints

```
┌─────────────────┬─────────────────────────────────────┐
│ LEETCODE_VARIANT│ Endpoint                            │
├─────────────────┼─────────────────────────────────────┤
│ com             │ https://leetcode.com/graphql/       │
│ cn              │ https://leetcode.cn/graphql/        │
└─────────────────┴─────────────────────────────────────┘
```

### Telegram Configuration (Optional)

| Variable | Description | Required For |
|----------|-------------|--------------|
| `TELEGRAM_TOKEN` | Bot token from @BotFather | `--telegram` |
| `TELEGRAM_CHAT_ID` | Target chat/channel ID | `--telegram` |

### Discord Configuration (Optional)

| Variable | Description | Required For |
|----------|-------------|--------------|
| `DISCORD_TOKEN` | Bot token from Discord Developer Portal | `--discord` |
| `DISCORD_CHANNEL_ID` | Target channel ID | `--discord` |

### Wake-Up Detection (Optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `WAKE_UP_START` | Start hour for wake-up detection | `3` |
| `WAKE_UP_END` | End hour for wake-up detection | `9` |

### Running Statistics (Optional)

| Variable | Description |
|----------|-------------|
| `RUNNING_PARQUET_URL` | Remote URL to parquet file |
| `RUNNING_PARQUET_FILE` | Local path to parquet file |

## GitHub Token Setup

### Step 1: Create Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Configure token:

```
┌─────────────────────────────────────────────────────────────┐
│ Token Settings                                               │
├─────────────────────────────────────────────────────────────┤
│ Note: leetcode-daily-bot                                    │
│ Expiration: No expiration (or as needed)                    │
│                                                              │
│ Required Scopes:                                            │
│ ☑ repo          - Full control of private repositories      │
│   ☑ repo:status - Commit statuses                           │
│   ☑ repo_deployment - Deployment statuses                   │
│   ☑ public_repo - Public repositories                       │
│   ☑ repo:invite - Repository invitations                    │
│   ☑ security_events - Read/write security events           │
└─────────────────────────────────────────────────────────────┘
```

### Step 2: Add Token to .env

```bash
# .env
GITHUB_TOKEN=ghp_your_generated_token_here
```

### Step 3: Secure the Token

```bash
# Set restrictive permissions (Linux/macOS)
chmod 600 .env

# Add to .gitignore
echo ".env" >> .gitignore
```

## Telegram Bot Setup

### Step 1: Create Bot

1. Open Telegram and search for @BotFather
2. Send `/newbot`
3. Follow prompts to name your bot
4. Save the bot token returned

```
┌─────────────────────────────────────────────────────────────┐
│ Example BotFather Response                                  │
├─────────────────────────────────────────────────────────────┤
│ Done! Congratulations on your new bot...                    │
│ Use this token to access the HTTP API:                      │
│ 1234567890:ABCdefGHIjklMNOpqrsTUVwxyz                       │
│ Keep your token secure...                                   │
└─────────────────────────────────────────────────────────────┘
```

### Step 2: Get Chat ID

#### For Personal Chat:

```bash
# Send a message to your bot, then:
curl "https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates" | jq
```

Look for `chat.id` in the response.

#### For Group Chat:

1. Add bot to group
2. Send a message mentioning the bot
3. Use getUpdates to find the chat ID (negative number for groups)

### Step 3: Configure

```bash
# .env
TELEGRAM_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_CHAT_ID=123456789
```

### Step 4: Build with Feature

```bash
cargo build --release --features telegram
```

## Discord Bot Setup

### Step 1: Create Application

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application"
3. Name your bot and save

### Step 2: Create Bot User

1. Navigate to "Bot" section
2. Click "Add Bot"
3. Save the token (click "Reset Token" if needed)

```
┌─────────────────────────────────────────────────────────────┐
│ Bot Permissions Required                                    │
├─────────────────────────────────────────────────────────────┤
│ Text Permissions:                                           │
│ ☑ Send Messages                                            │
│ ☑ Embed Links                                              │
│ ☑ Read Message History                                     │
└─────────────────────────────────────────────────────────────┘
```

### Step 3: Get Channel ID

1. Enable Developer Mode in Discord (Settings → Advanced → Developer Mode)
2. Right-click the target channel
3. Select "Copy ID"

### Step 4: Invite Bot to Server

Generate OAuth2 URL:
1. Go to OAuth2 → URL Generator
2. Select "bot" scope
3. Select "Send Messages" permission
4. Copy and open the generated URL

### Step 5: Configure

```bash
# .env
DISCORD_TOKEN=MTk4NjIyNDgzNDc...
DISCORD_CHANNEL_ID=123456789012345678
```

### Step 6: Build with Feature

```bash
cargo build --release --features discord
```

## LeetCode Endpoint Configuration

### Using LeetCode.com (Default)

```bash
LEETCODE_ENDPOINT=https://leetcode.com/graphql/
LEETCODE_VARIANT=com
```

### Using LeetCode CN

```bash
LEETCODE_ENDPOINT=https://leetcode.cn/graphql/
LEETCODE_VARIANT=cn
```

This affects:
- Problem URLs in generated messages
- Daily challenge source
- API requests

## Timezone Configuration

Use IANA timezone identifiers:

```bash
# Examples
TIMEZONE=UTC
TIMEZONE=America/New_York
TIMEZONE=America/Los_Angeles
TIMEZONE=Europe/London
TIMEZONE=Asia/Shanghai
TIMEZONE=Asia/Tokyo
TIMEZONE=Australia/Sydney
```

### Finding Your Timezone

```bash
# Linux
timedatectl list-timezones | grep -i your_city

# Common timezones
echo "UTC, America/New_York, America/Los_Angeles, Europe/London, Asia/Shanghai"
```

## Running Stats Parquet Format

The `data/running.parquet` file should have the following schema:

```
┌─────────────────┬─────────────┬─────────────────────────────────┐
│ Column          │ Type        │ Description                     │
├─────────────────┼─────────────┼─────────────────────────────────┤
│ date            │ Date        │ Date of the run                 │
│ distance_km     │ Float64     │ Distance in kilometers          │
│ duration_min    │ Float64     │ Duration in minutes (optional)  │
│ notes           │ String      │ Notes (optional)                │
└─────────────────┴─────────────┴─────────────────────────────────┘
```

### Example Data

| date | distance_km |
|------|-------------|
| 2024-01-14 | 5.23 |
| 2024-01-13 | 3.15 |
| 2024-01-12 | 8.00 |

### Creating a Sample Parquet File

```python
import polars as pl
from datetime import date

df = pl.DataFrame({
    "date": [date(2024, 1, 14), date(2024, 1, 13)],
    "distance_km": [5.23, 3.15]
})

df.write_parquet("data/running.parquet")
```

## Configuration File Example

Complete `.env` file:

```bash
# GitHub Configuration
GITHUB_TOKEN=ghp_your_token_here
REPO_OWNER=yourusername
REPO_NAME=leetcode-daily

# Personal Configuration
BIRTH_YEAR=1990
TIMEZONE=America/New_York

# LeetCode Configuration
LEETCODE_ENDPOINT=https://leetcode.com/graphql/
LEETCODE_VARIANT=com

# Telegram (Optional)
TELEGRAM_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_CHAT_ID=123456789

# Discord (Optional)
DISCORD_TOKEN=MTk4NjIyNDgzNDc...
DISCORD_CHANNEL_ID=123456789012345678

# Wake-Up Detection
WAKE_UP_START=3
WAKE_UP_END=9

# Running Stats (Optional)
RUNNING_PARQUET_FILE=data/running.parquet
```

## Next Steps

- **CLI usage**: See [API Reference](api-reference.md)
- **Architecture**: See [Architecture Overview](architecture.md)
- **Deployment**: See [Deployment Guide](deployment.md)
