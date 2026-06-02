# Deployment Guide

This guide covers deploying rust-leetcode-daily to run automatically every day.

## Building for Production

### Release Build

```bash
# Build optimized binary
cargo build --release

# With specific features
cargo build --release --features telegram,discord

# Binary location
ls -la target/release/routine-daily
```

### Build Options

```bash
# Minimal build (GitHub only)
cargo build --release

# With Telegram
cargo build --release --features telegram

# With Discord
cargo build --release --features discord

# With all notification channels
cargo build --release --features telegram,discord
```

### Cross-Compilation

```bash
# Add target
rustup target add x86_64-unknown-linux-musl

# Build for Linux (static binary)
cargo build --release --target x86_64-unknown-linux-musl

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

### Binary Size Optimization

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
panic = "abort"      # Smaller panic handling
```

## Cron Job Setup (Linux/macOS)

### Basic Cron Configuration

```bash
# Edit crontab
crontab -e

# Add entry (runs at 6:00 AM daily)
0 6 * * * cd /path/to/routine-daily && ./target/release/routine-daily --post --telegram >> /var/log/routine-daily.log 2>&1
```

### Cron Schedule Examples

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6) (Sunday = 0)
│ │ │ │ │
* * * * * command

Examples:
0 6 * * *     # Every day at 6:00 AM
30 7 * * 1-5  # Weekdays at 7:30 AM
0 5,6,7 * * * # At 5, 6, and 7 AM
*/30 6-8 * * * # Every 30 min between 6-8 AM
```

### Environment Variables in Cron

```bash
# Option 1: Source .env in cron
0 6 * * * cd /path/to/routine-daily && export $(cat .env | xargs) && ./target/release/routine-daily --post

# Option 2: Use env file directly (modify code to use dotenvy in release)

# Option 3: Set all variables in cron
0 6 * * * GITHUB_TOKEN=xxx REPO_OWNER=user REPO_NAME=repo BIRTH_YEAR=1990 TIMEZONE=America/New_York /path/to/routine-daily --post
```

### systemd Timer (Alternative to Cron)

Create service file `/etc/systemd/system/routine-daily.service`:

```ini
[Unit]
Description=LeetCode Daily Wake-up Message
After=network.target

[Service]
Type=oneshot
WorkingDirectory=/opt/routine-daily
EnvironmentFile=/opt/routine-daily/.env
ExecStart=/opt/routine-daily/routine-daily --post --telegram
User=leetcode
Group=leetcode

[Install]
WantedBy=multi-user.target
```

Create timer file `/etc/systemd/system/routine-daily.timer`:

```ini
[Unit]
Description=Run LeetCode Daily at 6 AM

[Timer]
OnCalendar=*-*-* 06:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable routine-daily.timer
sudo systemctl start routine-daily.timer

# Check status
systemctl list-timers
```

## Windows Task Scheduler

### Using GUI

1. Open Task Scheduler (`taskschd.msc`)

2. Click "Create Task"

3. **General Tab:**
   - Name: "LeetCode Daily"
   - Security options: "Run whether user is logged on or not"

4. **Triggers Tab:**
   - New → Daily
   - Start: Today, 6:00:00 AM
   - Repeat task every: 1 day

5. **Actions Tab:**
   - New → Start a program
   - Program: `C:\path\to\routine-daily.exe`
   - Arguments: `--post --telegram`
   - Start in: `C:\path\to\routine-daily`

6. **Conditions Tab:**
   - Uncheck "Start the task only if the computer is on AC power"

7. **Settings Tab:**
   - Check "Run task as soon as possible after a scheduled start is missed"

### Using PowerShell

```powershell
# Create action
$action = New-ScheduledTaskAction -Execute "C:\path\to\routine-daily.exe" -Argument "--post --telegram" -WorkingDirectory "C:\path\to\routine-daily"

# Create trigger (daily at 6 AM)
$trigger = New-ScheduledTaskTrigger -Daily -At 6am

# Create settings
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -DontStopOnIdleEnd

# Register task
Register-ScheduledTask -TaskName "LeetCode Daily" -Action $action -Trigger $trigger -Settings $settings -User "YourUsername"
```

### Environment Variables on Windows

Set system environment variables:

```powershell
# Set variables
[Environment]::SetEnvironmentVariable("GITHUB_TOKEN", "ghp_xxx", "User")
[Environment]::SetEnvironmentVariable("REPO_OWNER", "username", "User")
[Environment]::SetEnvironmentVariable("REPO_NAME", "routine-daily", "User")
[Environment]::SetEnvironmentVariable("BIRTH_YEAR", "1990", "User")
[Environment]::SetEnvironmentVariable("TIMEZONE", "America/New_York", "User")

# Or use .env file in the working directory
```

## GitHub Actions Deployment

### Workflow File

Create `.github/workflows/daily.yml`:

```yaml
name: Daily Wake-up Message

on:
  schedule:
    # Runs at 6:00 AM UTC daily
    - cron: '0 6 * * *'
  workflow_dispatch: # Manual trigger

env:
  CARGO_TERM_COLOR: always

jobs:
  build-and-run:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build
        run: cargo build --release --features telegram
      
      - name: Fetch EASY problems (if needed)
        run: |
          if [ ! -f data/leetcode_easy.txt ]; then
            ./target/release/routine-daily --fetch-easy
          fi
      
      - name: Run daily message
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          REPO_OWNER: ${{ github.repository_owner }}
          REPO_NAME: ${{ github.event.repository.name }}
          BIRTH_YEAR: ${{ secrets.BIRTH_YEAR }}
          TIMEZONE: ${{ secrets.TIMEZONE }}
          TELEGRAM_TOKEN: ${{ secrets.TELEGRAM_TOKEN }}
          TELEGRAM_CHAT_ID: ${{ secrets.TELEGRAM_CHAT_ID }}
        run: |
          ./target/release/routine-daily --post --telegram
      
      - name: Commit used problems
        run: |
          git config --local user.email "github-actions[bot]@users.noreply.github.com"
          git config --local user.name "github-actions[bot]"
          git add data/used_problems.txt || true
          git diff --quiet && git diff --staged --quiet || git commit -m "Update used problems"
          git push
```

### Required Secrets

Configure in Settings → Secrets and variables → Actions:

| Secret | Description |
|--------|-------------|
| `BIRTH_YEAR` | Your birth year |
| `TIMEZONE` | Your timezone (e.g., `America/New_York`) |
| `TELEGRAM_TOKEN` | Telegram bot token |
| `TELEGRAM_CHAT_ID` | Telegram chat ID |
| `DISCORD_TOKEN` | Discord bot token (if using Discord) |
| `DISCORD_CHANNEL_ID` | Discord channel ID (if using Discord) |

### Self-hosted Runner (Optional)

For machines that are always on:

1. Go to Settings → Actions → Runners
2. Click "New self-hosted runner"
3. Follow setup instructions
4. Modify workflow:

```yaml
jobs:
  build-and-run:
    runs-on: self-hosted
```

## Docker Deployment

### Dockerfile

Create `Dockerfile`:

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --features telegram,discord && rm -rf src

# Copy source
COPY src ./src

# Build for real
RUN touch src/main.rs && cargo build --release --features telegram,discord

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/routine-daily /usr/local/bin/

# Create data directory
RUN mkdir -p /app/data

# Set working directory
WORKDIR /app

# Run
ENTRYPOINT ["routine-daily"]
CMD ["--help"]
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  routine-daily:
    build: .
    volumes:
      - ./data:/app/data
      - ./.env:/app/.env:ro
    command: ["--post", "--telegram"]
    
  # Optional: Cron scheduler
  ofelia:
    image: mcuadros/ofelia:latest
    depends_on:
      - routine-daily
    command: daemon --docker
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    labels:
      ofelia.job-local.routine-daily.schedule: "0 6 * * *"
      ofelia.job-local.routine-daily.command: "docker-compose run --rm routine-daily --post --telegram"
```

### Running with Docker

```bash
# Build image
docker build -t routine-daily .

# Run once
docker run --rm \
  -v $(pwd)/data:/app/data \
  -e GITHUB_TOKEN=xxx \
  -e REPO_OWNER=user \
  -e REPO_NAME=repo \
  -e BIRTH_YEAR=1990 \
  -e TIMEZONE=America/New_York \
  routine-daily --post --telegram

# Using docker-compose
docker-compose up
```

### Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: routine-daily
spec:
  schedule: "0 6 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: routine-daily
            image: routine-daily:latest
            args: ["--post", "--telegram"]
            env:
            - name: GITHUB_TOKEN
              valueFrom:
                secretKeyRef:
                  name: leetcode-secrets
                  key: github-token
            - name: REPO_OWNER
              value: "your-username"
            - name: REPO_NAME
              value: "routine-daily"
            - name: BIRTH_YEAR
              value: "1990"
            - name: TIMEZONE
              value: "America/New_York"
            - name: TELEGRAM_TOKEN
              valueFrom:
                secretKeyRef:
                  name: leetcode-secrets
                  key: telegram-token
            - name: TELEGRAM_CHAT_ID
              valueFrom:
                secretKeyRef:
                  name: leetcode-secrets
                  key: telegram-chat-id
            volumeMounts:
            - name: data
              mountPath: /app/data
          volumes:
          - name: data
            persistentVolumeClaim:
              claimName: leetcode-data-pvc
          restartPolicy: OnFailure
```

## Environment Secrets Management

### Best Practices

1. **Never commit secrets to repository**
   ```bash
   # Add to .gitignore
   .env
   *.pem
   *_token
   ```

2. **Use secret management tools:**
   - GitHub Secrets (for Actions)
   - HashiCorp Vault
   - AWS Secrets Manager
   - Azure Key Vault

3. **Rotate tokens regularly**
   - GitHub PAT: Every 90 days
   - Bot tokens: When compromised

4. **Use minimal permissions**
   - GitHub: Only `repo` scope
   - Telegram/Discord: Only send messages

### Local Development

```bash
# Use .env file (never commit)
cp .env.example .env
# Edit .env with real values

# Or use direnv
echo 'export GITHUB_TOKEN=xxx' > .envrc
direnv allow
```

### Production

```bash
# Use environment variables
export GITHUB_TOKEN=$(cat /run/secrets/github_token)

# Or use systemd's EnvironmentFile
# In service file:
EnvironmentFile=/etc/routine-daily/secrets.conf
```

## Monitoring and Logging

### Log to File

```bash
# Redirect output
0 6 * * * /path/to/routine-daily --post >> /var/log/leetcode.log 2>&1
```

### Log Rotation

Create `/etc/logrotate.d/routine-daily`:

```
/var/log/leetcode.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
```

### Health Checks

```bash
#!/bin/bash
# healthcheck.sh

LOG_FILE="/var/log/leetcode.log"
LAST_RUN=$(stat -c %Y "$LOG_FILE" 2>/dev/null || echo 0)
NOW=$(date +%s)
DIFF=$((NOW - LAST_RUN))

# Alert if last run was more than 25 hours ago
if [ $DIFF -gt 90000 ]; then
    echo "Warning: routine-daily hasn't run in over 25 hours"
    exit 1
fi

exit 0
```

## Related Documentation

- [Configuration Guide](./configuration.md) - Setting up tokens
- [FAQ](./faq.md) - Common deployment issues
- [Development Guide](./development.md) - Building and testing