use anyhow::Result;
use chrono::{Datelike, NaiveDate, TimeDelta};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

const QUOTES: &[&str] = &[
    r#"The only way to do great work is to love what you do.

—— Steve Jobs"#,
    r#"Be yourself; everyone else is already taken.

—— Oscar Wilde"#,
    r#"In the middle of difficulty lies opportunity.

—— Albert Einstein"#,
    r#"It always seems impossible until it is done.

—— Nelson Mandela"#,
    r#"Do not wait to strike till the iron is hot; but make it hot by striking.

—— William Butler Yeats"#,
    r#"The best way to predict the future is to invent it.

—— Alan Kay"#,
    r#"Code is like humor. When you have to explain it, it is bad.

—— Cory House"#,
    r#"First, solve the problem. Then, write the code.

—— John Johnson"#,
    r#"Simplicity is the soul of efficiency.

—— Austin Freeman"#,
    r#"Make it work, make it right, make it fast.

—— Kent Beck"#,
    r#"The function of good software is to make the complex appear to be simple.

—— Grady Booch"#,
    r#"Any fool can write code that a computer can understand. Good programmers write code that humans can understand.

—— Martin Fowler"#,
    r#"Experience is the name everyone gives to their mistakes.

—— Oscar Wilde"#,
    r#"Knowledge is power.

—— Francis Bacon"#,
    r#"The only true wisdom is in knowing you know nothing.

—— Socrates"#,
    r#"Everything you can imagine is real.

—— Pablo Picasso"#,
    r#"Whatever you are, be a good one.

—— Abraham Lincoln"#,
    r#"If you can dream it, you can do it.

—— Walt Disney"#,
    r#"Well done is better than well said.

—— Benjamin Franklin"#,
    r#"The secret of getting ahead is getting started.

—— Mark Twain"#,
    r#"Don't watch the clock; do what it does. Keep going.

—— Sam Levenson"#,
    r#"The future belongs to those who believe in the beauty of their dreams.

—— Eleanor Roosevelt"#,
    r#"It does not matter how slowly you go as long as you do not stop.

—— Confucius"#,
    r#"The only limit to our realization of tomorrow will be our doubts of today.

—— Franklin D. Roosevelt"#,
    r#"You miss 100% of the shots you don't take.

—— Wayne Gretzky"#,
    r#"Believe you can and you're halfway there.

—— Theodore Roosevelt"#,
    r#"Act as if what you do makes a difference. It does.

—— William James"#,
    r#"Success is not final, failure is not fatal: it is the courage to continue that counts.

—— Winston Churchill"#,
    r#"What you get by achieving your goals is not as important as what you become by achieving your goals.

—— Zig Ziglar"#,
    r#"Hardships often prepare ordinary people for an extraordinary destiny.

—— C.S. Lewis"#,
];

#[derive(Debug, Deserialize)]
struct ZenQuote {
    q: String,
    a: String,
}

#[derive(Debug, Deserialize)]
struct WikiResponse {
    events: Vec<WikiEvent>,
}

#[derive(Debug, Deserialize)]
struct WikiEvent {
    year: i32,
    text: String,
    pages: Vec<WikiPage>,
}

#[derive(Debug, Deserialize)]
struct WikiPage {
    #[allow(dead_code)]
    title: String,
    content_urls: WikiContentUrls,
}

#[derive(Debug, Deserialize)]
struct WikiContentUrls {
    desktop: WikiDesktop,
}

#[derive(Debug, Deserialize)]
struct WikiDesktop {
    page: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunningStats {
    pub yesterday_km: f64,
    pub yesterday_count: i32,
    pub month_km: f64,
    pub month_count: i32,
    pub year_km: f64,
    pub year_count: i32,
}

fn get_daily_quote_index() -> usize {
    let now = chrono::Utc::now();
    let day_of_year = now.ordinal() as usize;
    let year = now.year() as usize;
    let seed = year * 1000 + day_of_year;
    seed % QUOTES.len()
}

fn format_zen_quote(quotes: Vec<ZenQuote>) -> Option<String> {
    let first = quotes.into_iter().next()?;
    let text = first.q.trim();
    let author = first.a.trim();
    if text.is_empty() || author.is_empty() {
        return None;
    }
    Some(format!("{}\n\n—— {} (zenquotes.io)", text, author))
}

pub async fn fetch_quote(client: &reqwest::Client) -> Result<String> {
    let result = client
        .get("https://zenquotes.io/api/random")
        .header("User-Agent", "get-up-daily/0.2.0")
        .send()
        .await;

    match result {
        Ok(resp) => match resp.json::<Vec<ZenQuote>>().await {
            Ok(quotes) => match format_zen_quote(quotes) {
                Some(quote) => Ok(quote),
                None => Ok(QUOTES[get_daily_quote_index()].to_string()),
            },
            Err(_) => Ok(QUOTES[get_daily_quote_index()].to_string()),
        },
        Err(_) => Ok(QUOTES[get_daily_quote_index()].to_string()),
    }
}

const TRAGIC_WORDS: &[&str] = &[
    "war",
    "wars",
    "died",
    "death",
    "deaths",
    "dead",
    "fire",
    "flood",
    "plague",
    "drought",
    "riot",
    "coup",
    "sank",
    "crash",
    "bomb",
    "bombing",
    "attack",
    "attacks",
    "shot",
    "slain",
    "murder",
    "suicide",
    "disaster",
    "shooting",
    "collapse",
    "destroyed",
    "invasion",
    "executed",
    "famine",
    "tsunami",
    "typhoon",
    "hurricane",
    "massacre",
    "aeroplane",
    "airliner",
];

const TRAGIC_STEMS: &[&str] = &[
    "kill",
    "massacr",
    "assassinat",
    "genocid",
    "holocaust",
    "tortur",
    "hijack",
    "pandemi",
    "epidemi",
    "casualt",
    "victim",
    "airstrik",
    "bombard",
    "earthquak",
    "explos",
    "invad",
    "wounded",
];

fn is_tragic(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.split(|c: char| !c.is_ascii_alphanumeric()).any(|w| {
        !w.is_empty()
            && (TRAGIC_WORDS.contains(&w) || TRAGIC_STEMS.iter().any(|s| w.starts_with(s)))
    })
}

pub async fn fetch_history(
    client: &reqwest::Client,
    birth_year: i32,
    current_year: i32,
    month: u32,
    day: u32,
) -> Result<Vec<String>> {
    let url = format!(
        "https://api.wikimedia.org/feed/v1/wikipedia/en/onthisday/events/{}/{}",
        month, day
    );

    let response = match client
        .get(&url)
        .header("User-Agent", "LeetCodeDaily/0.1.0")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return Ok(vec![]),
    };

    let wiki_response = match response.json::<WikiResponse>().await {
        Ok(data) => data,
        Err(_) => return Ok(vec![]),
    };

    let mut events: Vec<(i32, String, String)> = wiki_response
        .events
        .into_iter()
        .filter(|event| event.year >= birth_year && event.year <= current_year)
        .filter_map(|event| {
            event.pages.first().map(|page| {
                (
                    event.year,
                    event.text,
                    page.content_urls.desktop.page.clone(),
                )
            })
        })
        .collect();

    events.sort_by_key(|b| std::cmp::Reverse(b.0));

    let (non_tragic, tragic): (Vec<_>, Vec<_>) = events
        .into_iter()
        .partition(|(_, text, _)| !is_tragic(text));

    let result: Vec<String> = non_tragic
        .into_iter()
        .chain(tragic)
        .take(2)
        .map(|(year, text, wiki_url)| {
            let age_text = if year >= birth_year {
                let age = year - birth_year;
                format!("(you were {})", age)
            } else {
                let years_before = birth_year - year;
                format!("({} years before you were born)", years_before)
            };
            format!("• {}: [{}]({}) {}", year, text, wiki_url, age_text)
        })
        .collect();

    Ok(result)
}

pub async fn fetch_running_stats(
    running_file: &str,
    today: NaiveDate,
) -> Result<Option<RunningStats>> {
    let yesterday = today - TimeDelta::days(1);

    if !std::path::Path::new(running_file).exists() {
        return Ok(None);
    }

    let df: LazyFrame = if running_file.ends_with(".csv") {
        let f = match std::fs::File::open(running_file) {
            Ok(f) => std::io::BufReader::new(f),
            Err(_) => return Ok(None),
        };
        let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(f);
        let mut dates = Vec::new();
        let mut distances = Vec::new();
        for result in rdr.records() {
            let record = match result {
                Ok(r) => r,
                Err(_) => continue,
            };
            if record.len() < 2 {
                continue;
            }
            if let (Ok(date), Ok(dist)) = (
                chrono::NaiveDate::parse_from_str(&record[0], "%Y-%m-%d"),
                record[1].parse::<f64>(),
            ) {
                dates.push(date);
                distances.push(dist);
            }
        }
        if dates.is_empty() {
            return Ok(None);
        }
        let frame = match polars::prelude::DataFrame::new(vec![
            polars::prelude::Series::new("date", dates),
            polars::prelude::Series::new("distance_km", distances),
        ]) {
            Ok(f) => f,
            Err(_) => return Ok(None),
        };
        frame.lazy()
    } else {
        match LazyFrame::scan_parquet(running_file, Default::default()) {
            Ok(df) => df,
            Err(_) => return Ok(None),
        }
    };

    let yesterday_str = yesterday.format("%Y-%m-%d").to_string();
    let month_str = today.format("%Y-%m").to_string();
    let year_str = today.format("%Y").to_string();

    let all_data = match df
        .with_column(col("date").dt().strftime("%Y-%m-%d").alias("_date_str"))
        .with_column(col("date").dt().strftime("%Y-%m").alias("_month_str"))
        .with_column(col("date").dt().strftime("%Y").alias("_year_str"))
        .collect()
    {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let yesterday_df = all_data
        .clone()
        .lazy()
        .filter(col("_date_str").eq(lit(yesterday_str.as_str())))
        .select([
            col("distance_km").sum().alias("total_distance"),
            col("_date_str").count().alias("session_count"),
        ])
        .collect();

    let month_df = all_data
        .clone()
        .lazy()
        .filter(col("_month_str").eq(lit(month_str.as_str())))
        .select([
            col("distance_km").sum().alias("total_distance"),
            col("_month_str").count().alias("session_count"),
        ])
        .collect();

    let year_df = all_data
        .lazy()
        .filter(col("_year_str").eq(lit(year_str.as_str())))
        .select([
            col("distance_km").sum().alias("total_distance"),
            col("_year_str").count().alias("session_count"),
        ])
        .collect();

    let (yesterday_km, yesterday_count) = extract_pair(yesterday_df);
    let (month_km, month_count) = extract_pair(month_df);
    let (year_km, year_count) = extract_pair(year_df);

    Ok(Some(RunningStats {
        yesterday_km,
        yesterday_count,
        month_km,
        month_count,
        year_km,
        year_count,
    }))
}

fn extract_pair(df_result: std::result::Result<DataFrame, PolarsError>) -> (f64, i32) {
    match df_result {
        Ok(df) => {
            let dist = df
                .column("total_distance")
                .ok()
                .and_then(|c| c.f64().ok())
                .and_then(|ca| ca.get(0))
                .unwrap_or(0.0);
            let count = df
                .column("session_count")
                .ok()
                .and_then(|c| c.u32().ok())
                .and_then(|ca| ca.get(0))
                .map(|v| v as i32)
                .unwrap_or(0);
            (dist, count)
        }
        Err(_) => (0.0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_stats_debug_clone() {
        let stats = RunningStats {
            yesterday_km: 1.0,
            yesterday_count: 1,
            month_km: 10.0,
            month_count: 5,
            year_km: 100.0,
            year_count: 50,
        };
        let cloned = stats.clone();
        assert!((cloned.yesterday_km - 1.0).abs() < f64::EPSILON);
        assert_eq!(cloned.year_count, 50);
    }

    #[test]
    fn test_format_zen_quote() {
        let json = r#"[{"q": "Be yourself", "a": "Unknown", "h": "hash"}]"#;
        let quotes: Vec<ZenQuote> = serde_json::from_str(json).unwrap();
        assert_eq!(
            format_zen_quote(quotes),
            Some("Be yourself\n\n—— Unknown (zenquotes.io)".to_string())
        );
    }

    #[test]
    fn test_format_zen_quote_empty() {
        assert_eq!(format_zen_quote(vec![]), None);
    }

    #[test]
    fn test_format_zen_quote_blank_text() {
        let quotes = vec![ZenQuote {
            q: "   ".to_string(),
            a: "Unknown".to_string(),
        }];
        assert_eq!(format_zen_quote(quotes), None);
    }

    #[test]
    fn test_format_zen_quote_blank_author() {
        let quotes = vec![ZenQuote {
            q: "Be yourself".to_string(),
            a: String::new(),
        }];
        assert_eq!(format_zen_quote(quotes), None);
    }

    #[test]
    fn test_is_tragic_detects_tragic_events() {
        assert!(is_tragic("Delhi, India, is hit by a series of bomb blasts"));
        assert!(is_tragic("fire"));
        assert!(is_tragic("The Great Fire of London"));
    }

    #[test]
    fn test_is_tragic_ignores_calm_events() {
        assert!(!is_tragic("Takuma Sato wins the Indy 500"));
        assert!(!is_tragic("warning sign"));
        assert!(!is_tragic("Firefox"));
    }

    #[test]
    fn test_wiki_response_deserialization() {
        let json = r#"{
            "events": [
                {
                    "year": 1990,
                    "text": "Something happened",
                    "pages": [
                        {
                            "title": "Main",
                            "content_urls": {
                                "desktop": {
                                    "page": "https://en.wikipedia.org/wiki/Main"
                                }
                            }
                        }
                    ]
                }
            ]
        }"#;
        let resp: WikiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.events.len(), 1);
        assert_eq!(resp.events[0].year, 1990);
        assert_eq!(
            resp.events[0].pages[0].content_urls.desktop.page,
            "https://en.wikipedia.org/wiki/Main"
        );
    }
}
