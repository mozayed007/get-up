use crate::config::Config;
use crate::notification::Notifier;
use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "telegram")]
pub struct TelegramNotifier {
    bot: teloxide::Bot,
    chat_id: teloxide::types::ChatId,
}

#[cfg(feature = "telegram")]
impl TelegramNotifier {
    pub fn from_config(config: &Config) -> Option<Self> {
        let token = config.telegram_token.clone()?;
        let chat_id: i64 = config.telegram_chat_id.clone()?.parse().ok()?;
        Some(Self {
            bot: teloxide::Bot::new(token),
            chat_id: teloxide::types::ChatId(chat_id),
        })
    }
}

#[cfg(feature = "telegram")]
#[async_trait]
impl Notifier for TelegramNotifier {
    fn name(&self) -> &str {
        "Telegram"
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        use teloxide::prelude::*;
        self.bot.send_message(self.chat_id, message).await?;
        Ok(())
    }
}

#[cfg(not(feature = "telegram"))]
pub struct TelegramNotifier;

#[cfg(not(feature = "telegram"))]
impl TelegramNotifier {
    pub fn from_config(_config: &Config) -> Option<Self> {
        None
    }
}

#[cfg(not(feature = "telegram"))]
#[async_trait]
impl Notifier for TelegramNotifier {
    fn name(&self) -> &str {
        "Telegram"
    }

    async fn send_message(&self, _message: &str) -> Result<()> {
        anyhow::bail!("Telegram feature not enabled. Recompile with --features telegram")
    }
}
