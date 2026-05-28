use anyhow::Result;
use chrono::{NaiveDate, TimeDelta};
use polars::prelude::*;
use serde::Deserialize;

const DEFAULT_QUOTE: &str = r#"The only way to do great work is to love what you do.

—— Steve Jobs"#;

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    content: String,
    author: String,
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

#[derive(Debug, Clone, Default)]
pub struct RunningStats {
    pub yesterday_km: f64,
    pub yesterday_count: i32,
    pub month_km: f64,
    pub month_count: i32,
    pub year_km: f64,
    pub year_count: i32,
}

pub async fn fetch_quote(client: &reqwest::Client) -> Result<String> {
    let result = client
        .get("https://api.quotable.io/random")
        .send()
        .await;

    match result {
        Ok(resp) => match resp.json::<QuoteResponse>().await {
            Ok(response) => Ok(format!("{}\n\n—— {}", response.content, response.author)),
            Err(_) => Ok(DEFAULT_QUOTE.to_string()),
        },
        Err(_) => Ok(DEFAULT_QUOTE.to_string()),
    }
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
                (event.year, event.text, page.content_urls.desktop.page.clone())
            })
        })
        .collect();

    events.sort_by_key(|b| std::cmp::Reverse(b.0));

    let result: Vec<String> = events
        .into_iter()
        .take(2)
        .map(|(year, text, wiki_url)| {
            let age_text = if year >= birth_year {
                let age = year - birth_year;
                format!("(I was {} years old)", age)
            } else {
                let years_before = birth_year - year;
                format!("({} years before I was born)", years_before)
            };
            format!("• {}: [{}]({}) {}", year, text, wiki_url, age_text)
        })
        .collect();

    Ok(result)
}

pub async fn fetch_running_stats(parquet_file: &str, today: NaiveDate) -> Result<RunningStats> {
    let yesterday = today - TimeDelta::days(1);

    if !std::path::Path::new(parquet_file).exists() {
        return Ok(RunningStats::default());
    }

    let df = match LazyFrame::scan_parquet(parquet_file, Default::default()) {
        Ok(df) => df,
        Err(_) => return Ok(RunningStats::default()),
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
        Err(_) => return Ok(RunningStats::default()),
    };

    let yesterday_df = all_data.clone().lazy()
        .filter(col("_date_str").eq(lit(yesterday_str.as_str())))
        .select([
            col("distance_km").sum().alias("total_distance"),
            col("_date_str").count().alias("session_count"),
        ])
        .collect();

    let month_df = all_data.clone().lazy()
        .filter(col("_month_str").eq(lit(month_str.as_str())))
        .select([
            col("distance_km").sum().alias("total_distance"),
            col("_month_str").count().alias("session_count"),
        ])
        .collect();

    let year_df = all_data.lazy()
        .filter(col("_year_str").eq(lit(year_str.as_str())))
        .select([
            col("distance_km").sum().alias("total_distance"),
            col("_year_str").count().alias("session_count"),
        ])
        .collect();

    let (yesterday_km, yesterday_count) = extract_pair(yesterday_df);
    let (month_km, month_count) = extract_pair(month_df);
    let (year_km, year_count) = extract_pair(year_df);

    Ok(RunningStats {
        yesterday_km,
        yesterday_count,
        month_km,
        month_count,
        year_km,
        year_count,
    })
}

fn extract_pair(df_result: std::result::Result<DataFrame, PolarsError>) -> (f64, i32) {
    match df_result {
        Ok(df) => {
            let dist = df.column("total_distance")
                .ok().and_then(|c| c.f64().ok()).and_then(|ca| ca.get(0)).unwrap_or(0.0);
            let count = df.column("session_count")
                .ok().and_then(|c| c.u32().ok()).and_then(|ca| ca.get(0)).map(|v| v as i32).unwrap_or(0);
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
    fn test_quote_response_deserialization() {
        let json = r#"{"content": "Be yourself", "author": "Unknown"}"#;
        let resp: QuoteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content, "Be yourself");
        assert_eq!(resp.author, "Unknown");
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
        assert_eq!(resp.events[0].pages[0].content_urls.desktop.page, "https://en.wikipedia.org/wiki/Main");
    }
}
