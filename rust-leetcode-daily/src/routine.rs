use anyhow::{Context, Result};
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

use crate::api::{self, RunningStats};
use crate::config::{Config, LeetCodeVariant};
use crate::leetcode::{LeetCode, Question};
use crate::message;
use crate::utils;

/// Output format for the routine result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Xml,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Xml => write!(f, "xml"),
        }
    }
}

/// Routine type: morning or night.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RoutineType {
    #[default]
    Morning,
    Night,
}

impl std::fmt::Display for RoutineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutineType::Morning => write!(f, "morning"),
            RoutineType::Night => write!(f, "night"),
        }
    }
}

/// Options for customizing the routine. Agents can control exactly what they want.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineOptions {
    /// Which routine to run.
    #[serde(default)]
    pub routine_type: RoutineType,

    /// Include LeetCode problem.
    #[serde(default = "default_true")]
    pub include_leetcode: bool,

    /// Include running statistics.
    #[serde(default = "default_true")]
    pub include_running: bool,

    /// Include on-this-day history.
    #[serde(default = "default_true")]
    pub include_history: bool,

    /// Include motivational quote.
    #[serde(default = "default_true")]
    pub include_quote: bool,

    /// Include year progress bar.
    #[serde(default = "default_true")]
    pub include_year_progress: bool,

    /// Output format.
    #[serde(default)]
    pub format: OutputFormat,
}

fn default_true() -> bool {
    true
}

impl Default for RoutineOptions {
    fn default() -> Self {
        Self {
            routine_type: RoutineType::Morning,
            include_leetcode: true,
            include_running: true,
            include_history: true,
            include_quote: true,
            include_year_progress: true,
            format: OutputFormat::Text,
        }
    }
}

impl RoutineOptions {
    /// Morning routine with all sections enabled.
    pub fn morning() -> Self {
        Self::default()
    }

    /// Night routine with all sections enabled.
    pub fn night() -> Self {
        Self {
            routine_type: RoutineType::Night,
            ..Self::default()
        }
    }

    /// LeetCode only — minimal routine for agents that just want the problem.
    pub fn leetcode_only() -> Self {
        Self {
            include_leetcode: true,
            include_running: false,
            include_history: false,
            include_quote: false,
            include_year_progress: false,
            ..Self::default()
        }
    }
}

/// A single historical event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub year: i32,
    pub text: String,
    pub url: String,
    pub age_context: String,
}

/// A motivational quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResult {
    pub text: String,
    pub author: String,
    pub source: String,
}

/// Year progress information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearProgress {
    pub day_of_year: u32,
    pub total_days: u32,
    pub percentage: f64,
    pub bar: String,
}

/// LeetCode problem result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeetCodeResult {
    pub question: Question,
    pub url: String,
    pub is_daily_challenge: bool,
}

/// The complete routine result. Agents get this as structured data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineResult {
    pub routine_type: RoutineType,
    pub greeting: String,
    pub timestamp: String,
    pub year_progress: Option<YearProgress>,
    pub leetcode: Option<LeetCodeResult>,
    pub running: Option<RunningStats>,
    pub history: Option<Vec<HistoryEvent>>,
    pub quote: Option<QuoteResult>,
    pub formatted_message: String,
}

/// Paths to data files used by the routine.
const EASY_FILE: &str = "data/leetcode_easy.txt";
const USED_FILE: &str = "data/used_problems.txt";
const RUNNING_FILE: &str = "data/running.parquet";

/// Run the routine with the given options. Returns structured data.
/// This is the main entry point for agents. It is idempotent — calling it
/// multiple times with the same config produces the same result (no side effects
/// like posting or marking problems as used).
pub async fn run_routine(config: &Config, options: &RoutineOptions) -> Result<RoutineResult> {
    let client = reqwest::Client::new();
    let leetcode = LeetCode::new(config);
    let now = utils::get_local_time(config);
    let current_hour = now.hour();

    let greeting = match options.routine_type {
        RoutineType::Morning => message::get_greeting(current_hour).to_string(),
        RoutineType::Night => "🌙 Good night".to_string(),
    };

    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let year_progress = if options.include_year_progress {
        let day_of_year = utils::get_day_of_year(&now);
        let bar = utils::get_year_progress(&now);
        let total_days = if chrono::NaiveDate::from_ymd_opt(now.year(), 12, 31).is_some() {
            // Check if leap year
            if chrono::NaiveDate::from_ymd_opt(now.year(), 2, 29).is_some() {
                366
            } else {
                365
            }
        } else {
            365
        };
        let percentage = (day_of_year as f64 / total_days as f64) * 100.0;
        Some(YearProgress {
            day_of_year,
            total_days,
            percentage,
            bar,
        })
    } else {
        None
    };

    let leetcode_result = if options.include_leetcode {
        match leetcode.get_today_problem(EASY_FILE, USED_FILE).await {
            Ok(problem) => {
                let url = match config.leetcode_variant {
                    LeetCodeVariant::Cn => {
                        format!("https://leetcode.cn/problems/{}/", problem.slug)
                    }
                    LeetCodeVariant::Com => {
                        format!("https://leetcode.com/problems/{}/", problem.slug)
                    }
                };
                let is_daily = problem.is_daily_challenge;
                Some(LeetCodeResult {
                    question: problem,
                    url,
                    is_daily_challenge: is_daily,
                })
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let running = if options.include_running {
        api::fetch_running_stats(RUNNING_FILE, now.date_naive())
            .await
            .ok()
    } else {
        None
    };

    let history = if options.include_history {
        match api::fetch_history(
            &client,
            config.birth_year,
            now.year(),
            now.month(),
            now.day(),
        )
        .await
        {
            Ok(raw_events) => Some(parse_history_events(&raw_events)),
            Err(_) => None,
        }
    } else {
        None
    };

    let quote = if options.include_quote {
        match api::fetch_quote(&client).await {
            Ok(raw) => Some(parse_quote(&raw)),
            Err(_) => None,
        }
    } else {
        None
    };

    let formatted_message = build_formatted_message(
        options,
        &greeting,
        &timestamp,
        year_progress.as_ref(),
        leetcode_result.as_ref(),
        running.as_ref(),
        history.as_deref(),
        quote.as_ref(),
        &config.leetcode_variant,
    );

    Ok(RoutineResult {
        routine_type: options.routine_type,
        greeting,
        timestamp,
        year_progress,
        leetcode: leetcode_result,
        running,
        history,
        quote,
        formatted_message,
    })
}

fn parse_history_events(raw: &[String]) -> Vec<HistoryEvent> {
    raw.iter()
        .filter_map(|line| {
            // Format: "• 1990: [text](url) (you were N)"
            let after_bullet = line.strip_prefix("• ").or_else(|| line.strip_prefix("• "))?;
            let colon_pos = after_bullet.find(':')?;
            let year: i32 = after_bullet[..colon_pos].parse().ok()?;
            let rest = after_bullet[colon_pos + 1..].trim();

            // Extract [text](url) and age context
            let (text, url, age_context) = if let Some(bracket_start) = rest.find('[') {
                let bracket_end = rest[bracket_start..].find(']').unwrap_or(0) + bracket_start;
                let text = rest[bracket_start + 1..bracket_end].to_string();
                let (url, age) = if let Some(paren_start) = rest[bracket_end..].find('(') {
                    let actual_start = bracket_end + paren_start;
                    if let Some(paren_end) = rest[actual_start..].find(')') {
                        let url = rest[actual_start + 1..actual_start + paren_end].to_string();
                        let age = rest[actual_start + paren_end + 1..].trim().to_string();
                        (url, age)
                    } else {
                        (String::new(), String::new())
                    }
                } else {
                    (String::new(), String::new())
                };
                (text, url, age)
            } else {
                (rest.to_string(), String::new(), String::new())
            };

            Some(HistoryEvent {
                year,
                text,
                url,
                age_context,
            })
        })
        .collect()
}

fn parse_quote(raw: &str) -> QuoteResult {
    // Format: "text\n\n—— author" or "text\n\n-- author"
    let parts: Vec<&str> = raw.splitn(2, "\n\n").collect();
    if parts.len() == 2 {
        let text = parts[0].trim().to_string();
        let author_line = parts[1].trim();
        let author = author_line
            .trim_start_matches("——")
            .trim_start_matches("--")
            .trim_start_matches('—')
            .trim()
            .to_string();
        QuoteResult {
            text,
            author,
            source: "api".to_string(),
        }
    } else {
        QuoteResult {
            text: raw.to_string(),
            author: "Unknown".to_string(),
            source: "fallback".to_string(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn build_formatted_message(
    _options: &RoutineOptions,
    greeting: &str,
    timestamp: &str,
    year_progress: Option<&YearProgress>,
    leetcode: Option<&LeetCodeResult>,
    running: Option<&RunningStats>,
    history: Option<&[HistoryEvent]>,
    quote: Option<&QuoteResult>,
    _variant: &LeetCodeVariant,
) -> String {
    let mut parts = vec![format!("{} — {}", greeting, timestamp)];

    if let Some(yp) = year_progress {
        parts.push(format!("\nDay {} · {}", yp.day_of_year, yp.bar));
    }

    if let Some(lc) = leetcode {
        let daily_hint = if lc.is_daily_challenge {
            " (daily challenge)"
        } else {
            ""
        };
        parts.push(format!(
            "\n🟢 LeetCode EASY: {}. {}{}\n{}",
            lc.question.id, lc.question.title, daily_hint, lc.url
        ));
    }

    if let Some(stats) = running {
        parts.push(format!(
            "\n🏃 Yesterday: {:.2} km · This month: {:.2} km · This year: {:.2} km",
            stats.yesterday_km, stats.month_km, stats.year_km
        ));
    }

    if let Some(events) = history {
        if !events.is_empty() {
            let mut hist = "\n📜 On this day:".to_string();
            for event in events {
                hist.push_str(&format!(
                    "\n• {}: {} {}",
                    event.year, event.text, event.age_context
                ));
            }
            parts.push(hist);
        }
    }

    if let Some(q) = quote {
        parts.push(format!("\n💬 Today's Quote\n{}\n\n—— {}", q.text, q.author));
    }

    parts.join("\n")
}

/// Serialize a RoutineResult to JSON string.
pub fn to_json(result: &RoutineResult) -> Result<String> {
    serde_json::to_string_pretty(result).context("Failed to serialize to JSON")
}

/// Serialize a RoutineResult to XML string.
pub fn to_xml(result: &RoutineResult) -> Result<String> {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str("<routine>\n");
    xml.push_str(&format!(
        "  <type>{}</type>\n",
        result.routine_type
    ));
    xml.push_str(&format!("  <greeting><![CDATA[{}]]></greeting>\n", result.greeting));
    xml.push_str(&format!("  <timestamp>{}</timestamp>\n", result.timestamp));

    if let Some(ref yp) = result.year_progress {
        xml.push_str("  <year_progress>\n");
        xml.push_str(&format!(
            "    <day_of_year>{}</day_of_year>\n",
            yp.day_of_year
        ));
        xml.push_str(&format!(
            "    <total_days>{}</total_days>\n",
            yp.total_days
        ));
        xml.push_str(&format!(
            "    <percentage>{:.1}</percentage>\n",
            yp.percentage
        ));
        xml.push_str(&format!("    <bar><![CDATA[{}]]></bar>\n", yp.bar));
        xml.push_str("  </year_progress>\n");
    }

    if let Some(ref lc) = result.leetcode {
        xml.push_str("  <leetcode>\n");
        xml.push_str(&format!(
            "    <id>{}</id>\n",
            lc.question.id
        ));
        xml.push_str(&format!(
            "    <title><![CDATA[{}]]></title>\n",
            lc.question.title
        ));
        xml.push_str(&format!(
            "    <slug>{}</slug>\n",
            lc.question.slug
        ));
        xml.push_str(&format!(
            "    <difficulty>{}</difficulty>\n",
            lc.question.difficulty
        ));
        xml.push_str(&format!("    <url>{}</url>\n", lc.url));
        xml.push_str(&format!(
            "    <is_daily_challenge>{}</is_daily_challenge>\n",
            lc.is_daily_challenge
        ));
        xml.push_str("  </leetcode>\n");
    }

    if let Some(ref stats) = result.running {
        xml.push_str("  <running>\n");
        xml.push_str(&format!(
            "    <yesterday_km>{:.2}</yesterday_km>\n",
            stats.yesterday_km
        ));
        xml.push_str(&format!(
            "    <yesterday_count>{}</yesterday_count>\n",
            stats.yesterday_count
        ));
        xml.push_str(&format!(
            "    <month_km>{:.2}</month_km>\n",
            stats.month_km
        ));
        xml.push_str(&format!(
            "    <month_count>{}</month_count>\n",
            stats.month_count
        ));
        xml.push_str(&format!(
            "    <year_km>{:.2}</year_km>\n",
            stats.year_km
        ));
        xml.push_str(&format!(
            "    <year_count>{}</year_count>\n",
            stats.year_count
        ));
        xml.push_str("  </running>\n");
    }

    if let Some(ref events) = result.history {
        xml.push_str("  <history>\n");
        for event in events {
            xml.push_str("    <event>\n");
            xml.push_str(&format!("      <year>{}</year>\n", event.year));
            xml.push_str(&format!(
                "      <text><![CDATA[{}]]></text>\n",
                event.text
            ));
            xml.push_str(&format!("      <url>{}</url>\n", event.url));
            xml.push_str(&format!(
                "      <age_context><![CDATA[{}]]></age_context>\n",
                event.age_context
            ));
            xml.push_str("    </event>\n");
        }
        xml.push_str("  </history>\n");
    }

    if let Some(ref q) = result.quote {
        xml.push_str("  <quote>\n");
        xml.push_str(&format!("    <text><![CDATA[{}]]></text>\n", q.text));
        xml.push_str(&format!(
            "    <author><![CDATA[{}]]></author>\n",
            q.author
        ));
        xml.push_str(&format!("    <source>{}</source>\n", q.source));
        xml.push_str("  </quote>\n");
    }

    xml.push_str(&format!(
        "  <formatted_message><![CDATA[{}]]></formatted_message>\n",
        result.formatted_message
    ));
    xml.push_str("</routine>");

    Ok(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routine_options_morning() {
        let opts = RoutineOptions::morning();
        assert_eq!(opts.routine_type, RoutineType::Morning);
        assert!(opts.include_leetcode);
        assert!(opts.include_running);
    }

    #[test]
    fn test_routine_options_night() {
        let opts = RoutineOptions::night();
        assert_eq!(opts.routine_type, RoutineType::Night);
        assert!(opts.include_leetcode);
    }

    #[test]
    fn test_routine_options_leetcode_only() {
        let opts = RoutineOptions::leetcode_only();
        assert!(opts.include_leetcode);
        assert!(!opts.include_running);
        assert!(!opts.include_history);
        assert!(!opts.include_quote);
        assert!(!opts.include_year_progress);
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Text.to_string(), "text");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Xml.to_string(), "xml");
    }

    #[test]
    fn test_parse_quote() {
        let raw = "Be yourself; everyone else is already taken.\n\n—— Oscar Wilde";
        let q = parse_quote(raw);
        assert_eq!(q.text, "Be yourself; everyone else is already taken.");
        assert_eq!(q.author, "Oscar Wilde");
        assert_eq!(q.source, "api");
    }

    #[test]
    fn test_parse_quote_fallback() {
        let raw = "Just do it.";
        let q = parse_quote(raw);
        assert_eq!(q.text, "Just do it.");
        assert_eq!(q.author, "Unknown");
        assert_eq!(q.source, "fallback");
    }

    #[test]
    fn test_parse_history_events() {
        let raw = vec![
            "• 1990: [Something happened](https://en.wikipedia.org/wiki/Something) (you were 0)"
                .to_string(),
        ];
        let events = parse_history_events(&raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].year, 1990);
        assert_eq!(events[0].text, "Something happened");
        assert!(events[0].url.contains("wikipedia"));
    }

    #[test]
    fn test_to_json_roundtrip() {
        let result = RoutineResult {
            routine_type: RoutineType::Morning,
            greeting: "Good morning".to_string(),
            timestamp: "2025-01-01 08:00:00".to_string(),
            year_progress: Some(YearProgress {
                day_of_year: 1,
                total_days: 365,
                percentage: 0.27,
                bar: "█░░░░░░░░░░░░░░░░░░░░".to_string(),
            }),
            leetcode: None,
            running: None,
            history: None,
            quote: None,
            formatted_message: "test".to_string(),
        };
        let json = to_json(&result).unwrap();
        assert!(json.contains("morning"));
        assert!(json.contains("Good morning"));

        let parsed: RoutineResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.routine_type, RoutineType::Morning);
    }

    #[test]
    fn test_to_xml() {
        let result = RoutineResult {
            routine_type: RoutineType::Morning,
            greeting: "Good morning".to_string(),
            timestamp: "2025-01-01 08:00:00".to_string(),
            year_progress: None,
            leetcode: None,
            running: None,
            history: None,
            quote: Some(QuoteResult {
                text: "Test quote".to_string(),
                author: "Author".to_string(),
                source: "api".to_string(),
            }),
            formatted_message: "test".to_string(),
        };
        let xml = to_xml(&result).unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<routine>"));
        assert!(xml.contains("<morning/>") || xml.contains(">morning<"));
        assert!(xml.contains("Test quote"));
    }
}
