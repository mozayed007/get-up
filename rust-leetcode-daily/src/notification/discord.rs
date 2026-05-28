use crate::config::Config;
use crate::notification::Notifier;
use anyhow::Result;
use async_trait::async_trait;

#[cfg(feature = "discord")]
enum DiscordTarget {
    Channel(serenity::model::id::ChannelId),
    User(serenity::model::id::UserId),
}

#[cfg(feature = "discord")]
pub struct DiscordNotifier {
    http: serenity::http::Http,
    target: DiscordTarget,
}

#[cfg(feature = "discord")]
impl DiscordNotifier {
    pub fn from_config(config: &Config) -> Option<Self> {
        let token = config.discord_token.clone()?;
        let http = serenity::http::Http::new(&token);

        if let Some(uid) = &config.discord_user_id {
            let id: u64 = uid.parse().ok()?;
            return Some(Self {
                http,
                target: DiscordTarget::User(serenity::model::id::UserId::new(id)),
            });
        }

        if let Some(cid) = &config.discord_channel_id {
            let id: u64 = cid.parse().ok()?;
            return Some(Self {
                http,
                target: DiscordTarget::Channel(serenity::model::id::ChannelId::new(id)),
            });
        }

        None
    }
}

#[cfg(feature = "discord")]
#[async_trait]
impl Notifier for DiscordNotifier {
    fn name(&self) -> &str {
        match self.target {
            DiscordTarget::User(_) => "Discord DM",
            DiscordTarget::Channel(_) => "Discord",
        }
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        use serenity::builder::CreateMessage;
        match &self.target {
            DiscordTarget::User(uid) => {
                uid.dm(&self.http, CreateMessage::new().content(message)).await?;
            }
            DiscordTarget::Channel(cid) => {
                cid.say(&self.http, message).await?;
            }
        }
        Ok(())
    }
}

#[cfg(not(feature = "discord"))]
pub struct DiscordNotifier;

#[cfg(not(feature = "discord"))]
impl DiscordNotifier {
    pub fn from_config(_config: &Config) -> Option<Self> {
        None
    }
}

#[cfg(not(feature = "discord"))]
#[async_trait]
impl Notifier for DiscordNotifier {
    fn name(&self) -> &str {
        "Discord"
    }

    async fn send_message(&self, _message: &str) -> Result<()> {
        anyhow::bail!("Discord feature not enabled. Recompile with --features discord")
    }
}
