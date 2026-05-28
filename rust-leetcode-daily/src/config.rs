use anyhow::{Context, Result};
use chrono_tz::Tz;
use dotenvy::dotenv;
use std::env;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeetCodeVariant {
    Com,
    Cn,
}

impl FromStr for LeetCodeVariant {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cn" | "leetcode-cn" => Ok(LeetCodeVariant::Cn),
            "com" | "leetcode-com" => Ok(LeetCodeVariant::Com),
            other => Err(format!("Unknown LeetCode variant: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub github_token: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub telegram_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub discord_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub discord_user_id: Option<String>,
    pub birth_year: i32,
    pub timezone: Tz,
    pub leetcode_endpoint: String,
    pub leetcode_variant: LeetCodeVariant,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();

        let github_token = env::var("GITHUB_TOKEN")
            .context("GITHUB_TOKEN must be set")?;
        let repo_owner = env::var("REPO_OWNER")
            .context("REPO_OWNER must be set")?;
        let repo_name = env::var("REPO_NAME")
            .context("REPO_NAME must be set")?;

        let telegram_token = env::var("TELEGRAM_TOKEN").ok();
        let telegram_chat_id = env::var("TELEGRAM_CHAT_ID").ok();

        let discord_token = env::var("DISCORD_TOKEN").ok();
        let discord_channel_id = env::var("DISCORD_CHANNEL_ID").ok();
        let discord_user_id = env::var("DISCORD_USER_ID").ok();

        let birth_year: i32 = env::var("BIRTH_YEAR")
            .context("BIRTH_YEAR must be set")?
            .parse()
            .context("BIRTH_YEAR must be a valid year")?;

        let timezone_str = env::var("TIMEZONE")
            .context("TIMEZONE must be set")?;
        let timezone: Tz = timezone_str
            .parse()
            .with_context(|| format!("Invalid timezone: {}", timezone_str))?;

        let leetcode_endpoint = env::var("LEETCODE_ENDPOINT")
            .unwrap_or_else(|_| "https://leetcode.com/graphql/".to_string());
        let leetcode_variant: LeetCodeVariant = env::var("LEETCODE_VARIANT")
            .unwrap_or_else(|_| "com".to_string())
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid LEETCODE_VARIANT: {}", e))?;

        Ok(Config {
            github_token,
            repo_owner,
            repo_name,
            telegram_token,
            telegram_chat_id,
            discord_token,
            discord_channel_id,
            discord_user_id,
            birth_year,
            timezone,
            leetcode_endpoint,
            leetcode_variant,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variant_com() {
        assert_eq!("com".parse::<LeetCodeVariant>().unwrap(), LeetCodeVariant::Com);
    }

    #[test]
    fn test_variant_com_long() {
        assert_eq!("leetcode-com".parse::<LeetCodeVariant>().unwrap(), LeetCodeVariant::Com);
    }

    #[test]
    fn test_variant_cn() {
        assert_eq!("cn".parse::<LeetCodeVariant>().unwrap(), LeetCodeVariant::Cn);
    }

    #[test]
    fn test_variant_cn_long() {
        assert_eq!("leetcode-cn".parse::<LeetCodeVariant>().unwrap(), LeetCodeVariant::Cn);
    }

    #[test]
    fn test_variant_case_insensitive() {
        assert_eq!("CN".parse::<LeetCodeVariant>().unwrap(), LeetCodeVariant::Cn);
        assert_eq!("COM".parse::<LeetCodeVariant>().unwrap(), LeetCodeVariant::Com);
    }

    #[test]
    fn test_variant_invalid() {
        assert!("garbage".parse::<LeetCodeVariant>().is_err());
        assert!("".parse::<LeetCodeVariant>().is_err());
    }

    #[test]
    fn test_variant_debug_and_clone() {
        let v = LeetCodeVariant::Com;
        let cloned = v;
        assert_eq!(format!("{:?}", cloned), "Com");
    }
}
