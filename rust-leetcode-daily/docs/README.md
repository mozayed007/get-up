# LeetCode Daily Documentation

Welcome to the **LeetCode Daily** documentation. This Rust CLI tool generates daily motivational messages featuring LeetCode problems, historical events, quotes, and running statistics, with support for multiple notification platforms.

## Overview

LeetCode Daily helps you:
- Track daily wake-up times with motivational messages
- Practice LeetCode problems (daily challenges or random EASY problems)
- View personalized historical events with age context
- Track running/exercise statistics
- Receive notifications via GitHub Issues, Telegram, and Discord

## Documentation Index

| Document | Description |
|----------|-------------|
| [Getting Started](getting-started.md) | Installation, prerequisites, and first-time setup |
| [Configuration](configuration.md) | Environment variables, tokens, and feature setup |
| [API Reference](api-reference.md) | CLI arguments, modules, structs, and traits |
| [Architecture](architecture.md) | Project structure, data flow, and design patterns |
| [Deployment](deployment.md) | Production builds, cron jobs, and CI/CD |
| [Development](development.md) | Development setup, testing, and contributing |
| [FAQ & Troubleshooting](faq.md) | Common issues and solutions |

## Quick Links

### For New Users
1. Read [Getting Started](getting-started.md) to install and configure
2. Follow the [Configuration](configuration.md) guide to set up tokens
3. Try `--dry-run` first to preview messages

### For Developers
1. Check [Architecture](architecture.md) to understand the codebase
2. Read [Development](development.md) for contribution guidelines
3. See [API Reference](api-reference.md) for module documentation

### For Deployment
1. Review [Deployment](deployment.md) for production setup
2. Choose your deployment method (cron, systemd, Docker, GitHub Actions)
3. Configure monitoring and error handling

## Features at a Glance

```
┌─────────────────────────────────────────────────────────────────┐
│                     LeetCode Daily CLI                          │
├─────────────────────────────────────────────────────────────────┤
│  Data Sources              │  Notification Channels             │
│  ─────────────             │  ─────────────────────             │
│  • LeetCode Problems       │  • GitHub Issues (comments)        │
│  • Daily Quotes            │  • Telegram Bot                    │
│  • Historical Events       │  • Discord Webhook                 │
│  • Running Statistics      │  • stdout (--dry-run)              │
├─────────────────────────────────────────────────────────────────┤
│  Features                  │  Configuration                     │
│  ────────                  │  ─────────────                     │
│  • Wake-up detection       │  • Environment variables           │
│  • Daily challenge sync    │  • Feature flags (compile-time)    │
│  • Problem deduplication   │  • Timezone support                │
│  • Seeded random selection │  • LeetCode CN support             │
└─────────────────────────────────────────────────────────────────┘
```

## Version Information

- **Current Version**: 0.1.0
- **Rust Edition**: 2021
- **Minimum Rust Version**: 1.70+

## License

This project is open source. See the repository for license details.

## Contributing

See [Development Guide](development.md) for information on contributing to this project.
