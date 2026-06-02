use anyhow::Result;
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

use crate::api::{self, RunningStats};
use crate::format;
use crate::message;
use crate::providers::deepml::DeepMLProvider;
use crate::providers::leetcode::LeetCodeProvider;
use crate::scheduler::get_schedule;
use crate::serialization;
use crate::types::{Platform, ProblemResult};
use crate::utils;

pub use crate::scheduler::Schedule;

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

/// A section of the routine output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Problems,
    Running,
    History,
    Quote,
    YearProgress,
}

/// Options for customizing the routine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineOptions {
    #[serde(default)]
    pub routine_type: RoutineType,
    #[serde(default = "default_sections")]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub format: OutputFormat,
}

fn default_sections() -> Vec<Section> {
    vec![
        Section::Problems,
        Section::Running,
        Section::History,
        Section::Quote,
        Section::YearProgress,
    ]
}

impl Default for RoutineOptions {
    fn default() -> Self {
        Self {
            routine_type: RoutineType::Morning,
            sections: default_sections(),
            format: OutputFormat::Text,
        }
    }
}

impl RoutineOptions {
    pub fn morning() -> Self {
        Self::default()
    }

    pub fn night() -> Self {
        Self {
            routine_type: RoutineType::Night,
            ..Self::default()
        }
    }

    pub fn problems_only() -> Self {
        Self {
            sections: vec![Section::Problems],
            ..Default::default()
        }
    }

    pub fn has_section(&self, section: Section) -> bool {
        self.sections.contains(&section)
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

/// The complete routine result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineResult {
    pub routine_type: RoutineType,
    pub greeting: String,
    pub timestamp: String,
    pub year_progress: Option<YearProgress>,
    pub problems: Vec<ProblemResult>,
    pub running: Option<RunningStats>,
    pub history: Option<Vec<HistoryEvent>>,
    pub quote: Option<QuoteResult>,
    pub formatted_message: String,
}

const RUNNING_FILE: &str = "data/running.parquet";
const USED_FILE: &str = "data/used_problems.txt";

pub async fn run_routine(config: &crate::config::Config, options: &RoutineOptions) -> Result<RoutineResult> {
    let client = reqwest::Client::new();
    let leetcode = LeetCodeProvider::new(config);
    let deepml = DeepMLProvider::new();
    let now = utils::get_local_time(config);
    let current_hour = now.hour();

    let greeting = match options.routine_type {
        RoutineType::Morning => message::get_greeting(current_hour).to_string(),
        RoutineType::Night => "🌙 Good night".to_string(),
    };

    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let year_progress = if options.has_section(Section::YearProgress) {
        let day_of_year = utils::get_day_of_year(&now);
        let bar = utils::get_year_progress(&now);
        let total_days = if chrono::NaiveDate::from_ymd_opt(now.year(), 12, 31).is_some() {
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

    let mut problems = Vec::new();

    if options.has_section(Section::Problems) {
        let lc_schedule = get_schedule(&now, Platform::LeetCode);
        for difficulty in lc_schedule.iter() {
            match leetcode.get_problem(USED_FILE, *difficulty).await {
                Ok(result) => problems.push(result),
                Err(e) => eprintln!("Warning: Failed to get LeetCode problem: {}", e),
            }
        }

        let dm_schedule = get_schedule(&now, Platform::DeepML);
        for difficulty in dm_schedule.iter() {
            match deepml.get_problem(USED_FILE, *difficulty).await {
                Ok(result) => problems.push(result),
                Err(e) => eprintln!("Warning: Failed to get Deep-ML problem: {}", e),
            }
        }
    }

    let running = if options.has_section(Section::Running) {
        api::fetch_running_stats(RUNNING_FILE, now.date_naive())
            .await
            .ok()
    } else {
        None
    };

    let history = if options.has_section(Section::History) {
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

    let quote = if options.has_section(Section::Quote) {
        match api::fetch_quote(&client).await {
            Ok(raw) => Some(parse_quote(&raw)),
            Err(_) => None,
        }
    } else {
        None
    };

    let year_progress_text = year_progress.as_ref().map(|yp| {
        format!("Day {} · {}", yp.day_of_year, yp.bar)
    });

    let running_text = running.as_ref().map(|stats| {
        format!(
            "🏃 Yesterday: {:.2} km · This month: {:.2} km · This year: {:.2} km",
            stats.yesterday_km, stats.month_km, stats.year_km
        )
    });

    let history_text = history.as_ref().and_then(|events| {
        if events.is_empty() {
            return None;
        }
        let mut text = "📜 On this day:".to_string();
        for event in events {
            text.push_str(&format!(
                "\n• {}: {} {}",
                event.year, event.text, event.age_context
            ));
        }
        Some(text)
    });

    let formatted_message = format::build_formatted_message(
        &greeting,
        &timestamp,
        year_progress_text.as_deref(),
        &problems,
        running_text.as_deref(),
        history_text.as_deref(),
        quote.as_ref().map(|q| q.text.as_str()),
        quote.as_ref().map(|q| q.author.as_str()),
    );

    Ok(RoutineResult {
        routine_type: options.routine_type,
        greeting,
        timestamp,
        year_progress,
        problems,
        running,
        history,
        quote,
        formatted_message,
    })
}

fn parse_history_events(raw: &[String]) -> Vec<HistoryEvent> {
    raw.iter()
        .filter_map(|line| {
            let after_bullet = line.strip_prefix("• ").or_else(|| line.strip_prefix("• "))?;
            let colon_pos = after_bullet.find(':')?;
            let year: i32 = after_bullet[..colon_pos].parse().ok()?;
            let rest = after_bullet[colon_pos + 1..].trim();

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

pub fn to_json(result: &RoutineResult) -> Result<String> {
    serialization::to_json(result)
}

pub fn to_xml(result: &RoutineResult) -> Result<String> {
    serialization::to_xml(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routine_options_morning() {
        let opts = RoutineOptions::morning();
        assert_eq!(opts.routine_type, RoutineType::Morning);
        assert!(opts.has_section(Section::Problems));
        assert!(opts.has_section(Section::Running));
    }

    #[test]
    fn test_routine_options_night() {
        let opts = RoutineOptions::night();
        assert_eq!(opts.routine_type, RoutineType::Night);
        assert!(opts.has_section(Section::Problems));
    }

    #[test]
    fn test_routine_options_problems_only() {
        let opts = RoutineOptions::problems_only();
        assert!(opts.has_section(Section::Problems));
        assert!(!opts.has_section(Section::Running));
        assert!(!opts.has_section(Section::History));
        assert!(!opts.has_section(Section::Quote));
        assert!(!opts.has_section(Section::YearProgress));
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
            problems: vec![],
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
            problems: vec![],
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
