use crate::config::Config;
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub fn get_day_of_year(now: &DateTime<Tz>) -> u32 {
    now.ordinal()
}

pub fn get_year_progress(now: &DateTime<Tz>) -> String {
    let day_of_year = now.ordinal() as usize;
    let year = now.year();
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_year = if is_leap { 366 } else { 365 };
    let percentage = (day_of_year as f64 / days_in_year as f64) * 100.0;

    let bar_length = 20;
    let filled = ((day_of_year as f64 / days_in_year as f64) * bar_length as f64).round() as usize;
    let filled = filled.min(bar_length);
    let empty = bar_length - filled;

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    format!("{}/{} ({:.1}%) {}", day_of_year, days_in_year, percentage, bar)
}

pub async fn read_lines(filename: &str) -> Result<Vec<String>> {
    let file = match tokio::fs::File::open(filename).await {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(anyhow::Error::from(e)).with_context(|| format!("Failed to open file: {}", filename)),
    };
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut lines_iter = reader.lines();
    while let Some(line) = lines_iter.next_line().await? {
        if !line.trim().is_empty() {
            lines.push(line);
        }
    }
    Ok(lines)
}

pub async fn append_line(filename: &str, line: &str) -> Result<()> {
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .await
        .with_context(|| format!("Failed to open file for appending: {}", filename))?;
    file.write_all(format!("{}\n", line).as_bytes())
        .await
        .with_context(|| format!("Failed to write to file: {}", filename))?;
    Ok(())
}

pub fn get_local_time(config: &Config) -> DateTime<Tz> {
    let now = Utc::now();
    now.with_timezone(&config.timezone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_get_day_of_year() {
        let tz: Tz = "UTC".parse().unwrap();
        let date = tz.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
        assert_eq!(get_day_of_year(&date), 61);
    }

    #[test]
    fn test_get_day_of_year_jan1() {
        let tz: Tz = "UTC".parse().unwrap();
        let date = tz.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(get_day_of_year(&date), 1);
    }

    #[test]
    fn test_get_year_progress_non_leap() {
        let tz: Tz = "America/New_York".parse().unwrap();
        let date = tz.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let result = get_year_progress(&date);
        assert!(result.starts_with("1/365"));
        assert!(result.contains("0.3%"));
    }

    #[test]
    fn test_get_year_progress_leap() {
        let tz: Tz = "America/New_York".parse().unwrap();
        let date = tz.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
        let result = get_year_progress(&date);
        assert!(result.starts_with("366/366"));
        assert!(result.contains("100.0%"));
    }

    #[tokio::test]
    async fn test_read_lines_file_not_found() {
        let lines = read_lines("/tmp/nonexistent_file_xyz_123.txt").await.unwrap();
        assert!(lines.is_empty());
    }

    #[tokio::test]
    async fn test_append_and_read_lines() {
        let tmp = std::env::temp_dir().join("test_append_lines.txt");
        // Clean up if exists
        let _ = tokio::fs::remove_file(&tmp).await;

        append_line(tmp.to_str().unwrap(), "hello").await.unwrap();
        append_line(tmp.to_str().unwrap(), "world").await.unwrap();

        let lines = read_lines(tmp.to_str().unwrap()).await.unwrap();
        assert_eq!(lines, vec!["hello", "world"]);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_read_lines_skips_empty() {
        let tmp = std::env::temp_dir().join("test_read_skips_empty.txt");
        tokio::fs::write(&tmp, "a\n\nb\n\n\nc\n").await.unwrap();

        let lines = read_lines(tmp.to_str().unwrap()).await.unwrap();
        assert_eq!(lines, vec!["a", "b", "c"]);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[test]
    fn test_get_local_time_returns_correct_tz() {
        let cfg = Config {
            github_token: "x".into(),
            repo_owner: "x".into(),
            repo_name: "x".into(),
            telegram_token: None,
            telegram_chat_id: None,
            discord_token: None,
            discord_channel_id: None,
            discord_user_id: None,
            birth_year: 1990,
            timezone: "Asia/Shanghai".parse().unwrap(),
            leetcode_endpoint: "https://leetcode.com/graphql".into(),
            leetcode_variant: crate::config::LeetCodeVariant::Com,
        };
        let local = get_local_time(&cfg);
        assert_eq!(local.timezone(), cfg.timezone);
    }
}
