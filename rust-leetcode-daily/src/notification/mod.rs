mod telegram;
mod discord;

pub use telegram::TelegramNotifier;
pub use discord::DiscordNotifier;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Notifier: Send + Sync {
    fn name(&self) -> &str;
    async fn send_message(&self, message: &str) -> Result<()>;
}
