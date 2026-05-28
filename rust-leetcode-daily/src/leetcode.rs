use anyhow::{anyhow, Result};
use chrono::{Datelike, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::{Config, LeetCodeVariant};
use crate::utils::read_lines;

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, LeetCodeVariant};
    use super::{GraphQLResponse, DailyChallengeData, CnDailyChallengeData};

    #[test]
    fn test_get_day_seed_structure() {
        let seed = LeetCode::get_day_seed();
        // seed = year * 1000 + day_of_year, so for 2024+ it's >= 2024001
        assert!(seed >= 2024001, "seed {} looks wrong (expected >= 2024001)", seed);
        // year * 1000 -> last 3 digits are day_of_year (1-366)
        let year_part = seed / 1000;
        let day_part = seed % 1000;
        assert!(year_part >= 2024, "year part {} looks wrong", year_part);
        assert!(day_part >= 1 && day_part <= 366, "day part {} out of range", day_part);
    }

    #[test]
    fn test_pick_seeded_random_deterministic() {
        let items = vec!["a", "b", "c", "d", "e"];
        let result1 = LeetCode::pick_seeded_random(&items, 42);
        let result2 = LeetCode::pick_seeded_random(&items, 42);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_pick_seeded_random_empty() {
        let empty: Vec<i32> = vec![];
        assert!(LeetCode::pick_seeded_random(&empty, 0).is_none());
    }

    #[test]
    fn test_pick_seeded_random_different_seeds() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let results: std::collections::HashSet<i32> = (0..50)
            .map(|s| LeetCode::pick_seeded_random(&items, s).unwrap())
            .collect();
        // With 10 items and 50 different seeds we should see at least 3 distinct values
        assert!(results.len() >= 3, "only {} unique values from 50 seeds", results.len());
    }

    #[test]
    fn test_pick_seeded_random_single_item() {
        let items = vec!["only"];
        assert_eq!(LeetCode::pick_seeded_random(&items, 999), Some("only"));
    }

    #[test]
    fn test_graphql_response_deserialization() {
        let json = r#"{
            "data": {
                "activeDailyCodingChallengeQuestion": {
                    "question": {
                        "questionFrontendId": "123",
                        "title": "Test Title",
                        "titleSlug": "test-title",
                        "difficulty": "EASY",
                        "isPaidOnly": false
                    }
                }
            }
        }"#;
        let resp: GraphQLResponse<DailyChallengeData> = serde_json::from_str(json).unwrap();
        let daily = resp.data.active_daily_coding_challenge_question.unwrap();
        assert_eq!(daily.question.question_frontend_id, "123");
        assert_eq!(daily.question.title, "Test Title");
        assert_eq!(daily.question.title_slug, "test-title");
        assert!(!daily.question.is_paid_only);
    }

    #[test]
    fn test_cn_graphql_response_deserialization() {
        let json = r#"{
            "data": {
                "todayRecord": [
                    {
                        "date": "2024-01-01",
                        "question": {
                            "questionFrontendId": "42",
                            "title": "CN Problem",
                            "titleSlug": "cn-problem",
                            "difficulty": "MEDIUM",
                            "isPaidOnly": false
                        }
                    }
                ]
            }
        }"#;
        let resp: GraphQLResponse<CnDailyChallengeData> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.today_record.len(), 1);
        let q = &resp.data.today_record[0].question;
        assert_eq!(q.title, "CN Problem");
        assert_eq!(q.title_slug, "cn-problem");
    }

    #[tokio::test]
    async fn test_pick_daily_problem() {
        let tmp = std::env::temp_dir();
        let easy_file = tmp.join("test_easy.txt");
        let used_file = tmp.join("test_used.txt");

        // Write easy file: id|title|slug
        tokio::fs::write(&easy_file, "1|First|first\n2|Second|second\n3|Third|third").await.unwrap();
        // Write used file: slug of second problem
        tokio::fs::write(&used_file, "second").await.unwrap();

        let config = Config {
            github_token: "x".into(),
            repo_owner: "x".into(),
            repo_name: "x".into(),
            telegram_token: None,
            telegram_chat_id: None,
            discord_token: None,
            discord_channel_id: None,
            discord_user_id: None,
            birth_year: 1990,
            timezone: "UTC".parse().unwrap(),
            leetcode_endpoint: "http://localhost".into(),
            leetcode_variant: LeetCodeVariant::Com,
        };
        let lc = LeetCode::new(&config);

        let problem = lc.pick_daily_problem(
            easy_file.to_str().unwrap(),
            used_file.to_str().unwrap(),
        ).await.unwrap();

        // Should not pick "second" (used)
        assert_ne!(problem.slug, "second");
        // Should be one of the available ones
        assert!(problem.slug == "first" || problem.slug == "third");

        let _ = tokio::fs::remove_file(&easy_file).await;
        let _ = tokio::fs::remove_file(&used_file).await;
    }

    #[tokio::test]
    async fn test_pick_daily_problem_all_used_error() {
        let tmp = std::env::temp_dir();
        let easy_file = tmp.join("test_easy_all_used.txt");
        let used_file = tmp.join("test_used_all_used.txt");

        tokio::fs::write(&easy_file, "1|Only|only-one").await.unwrap();
        tokio::fs::write(&used_file, "only-one").await.unwrap();

        let config = Config {
            github_token: "x".into(),
            repo_owner: "x".into(),
            repo_name: "x".into(),
            telegram_token: None,
            telegram_chat_id: None,
            discord_token: None,
            discord_channel_id: None,
            discord_user_id: None,
            birth_year: 1990,
            timezone: "UTC".parse().unwrap(),
            leetcode_endpoint: "http://localhost".into(),
            leetcode_variant: LeetCodeVariant::Com,
        };
        let lc = LeetCode::new(&config);

        let result = lc.pick_daily_problem(
            easy_file.to_str().unwrap(),
            used_file.to_str().unwrap(),
        ).await;

        assert!(result.is_err());

        let _ = tokio::fs::remove_file(&easy_file).await;
        let _ = tokio::fs::remove_file(&used_file).await;
    }

    #[tokio::test]
    async fn test_pick_daily_problem_no_used_file() {
        let tmp = std::env::temp_dir();
        let easy_file = tmp.join("test_easy_no_used.txt");

        tokio::fs::write(&easy_file, "1|First|first").await.unwrap();

        let config = Config {
            github_token: "x".into(),
            repo_owner: "x".into(),
            repo_name: "x".into(),
            telegram_token: None,
            telegram_chat_id: None,
            discord_token: None,
            discord_channel_id: None,
            discord_user_id: None,
            birth_year: 1990,
            timezone: "UTC".parse().unwrap(),
            leetcode_endpoint: "http://localhost".into(),
            leetcode_variant: LeetCodeVariant::Com,
        };
        let lc = LeetCode::new(&config);

        let problem = lc.pick_daily_problem(
            easy_file.to_str().unwrap(),
            "/tmp/nonexistent_used_file_xyz.txt",
        ).await.unwrap();

        assert_eq!(problem.slug, "first");

        let _ = tokio::fs::remove_file(&easy_file).await;
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub difficulty: String,
    pub is_daily_challenge: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DailyChallengeData {
    active_daily_coding_challenge_question: Option<ActiveDailyChallenge>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActiveDailyChallenge {
    question: QuestionRaw,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuestionRaw {
    question_frontend_id: String,
    title: String,
    title_slug: String,
    difficulty: String,
    #[allow(dead_code)]
    is_paid_only: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CnDailyChallengeData {
    today_record: Vec<CnDailyChallengeItem>,
}

#[derive(Debug, Deserialize)]
struct CnDailyChallengeItem {
    question: QuestionRaw,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: T,
}

pub struct LeetCode {
    client: Client,
    endpoint: String,
    variant: LeetCodeVariant,
}

impl LeetCode {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            endpoint: config.leetcode_endpoint.clone(),
            variant: config.leetcode_variant,
        }
    }

    pub async fn fetch_easy_list(&self, output_file: &str) -> Result<()> {
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct ApiProblem {
            stat: ApiStat,
            difficulty: ApiDifficulty,
            paid_only: bool,
        }
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct ApiStat {
            frontend_question_id: i32,
            question__title: String,
            question__title_slug: String,
        }
        #[derive(Deserialize)]
        struct ApiDifficulty {
            level: i32,
        }
        #[derive(Deserialize)]
        struct ApiResponse {
            stat_status_pairs: Vec<ApiProblem>,
        }

        let resp: ApiResponse = self
            .client
            .get("https://leetcode.com/api/problems/algorithms/")
            .header("User-Agent", "LeetCodeDaily/0.1.0")
            .send()
            .await?
            .json()
            .await?;

        let mut output = String::new();
        for p in resp.stat_status_pairs {
            if p.difficulty.level == 1 && !p.paid_only {
                output.push_str(&format!(
                    "{}|{}|{}\n",
                    p.stat.frontend_question_id, p.stat.question__title, p.stat.question__title_slug
                ));
            }
        }

        tokio::fs::write(output_file, output.trim()).await?;
        Ok(())
    }

    pub async fn get_daily_challenge(&self) -> Result<Option<Question>> {
        match self.variant {
            LeetCodeVariant::Cn => self.get_daily_challenge_cn().await,
            LeetCodeVariant::Com => self.get_daily_challenge_com().await,
        }
    }

    async fn get_daily_challenge_com(&self) -> Result<Option<Question>> {
        let query = r#"
            query activeDailyCodingChallengeQuestion {
                activeDailyCodingChallengeQuestion {
                    question {
                        questionFrontendId
                        title
                        titleSlug
                        difficulty
                        isPaidOnly
                    }
                }
            }
        "#;

        let request = GraphQLRequest {
            query: query.to_string(),
            variables: serde_json::json!({}),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("User-Agent", "LeetCodeDaily/0.1.0")
            .json(&request)
            .send()
            .await?;

        let graphql_response: GraphQLResponse<DailyChallengeData> = response.json().await?;

        if let Some(daily) = graphql_response.data.active_daily_coding_challenge_question {
            let q = daily.question;
            Ok(Some(Question {
                id: q.question_frontend_id,
                title: q.title,
                slug: q.title_slug,
                difficulty: q.difficulty,
                is_daily_challenge: true,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_daily_challenge_cn(&self) -> Result<Option<Question>> {
        let query = r#"
            query todayRecord {
                todayRecord {
                    date
                    question {
                        questionFrontendId
                        title
                        titleSlug
                        difficulty
                        isPaidOnly
                    }
                }
            }
        "#;

        let request = GraphQLRequest {
            query: query.to_string(),
            variables: serde_json::json!({}),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("User-Agent", "LeetCodeDaily/0.1.0")
            .json(&request)
            .send()
            .await?;

        let graphql_response: GraphQLResponse<CnDailyChallengeData> = response.json().await?;

        if let Some(daily) = graphql_response.data.today_record.into_iter().next() {
            let q = daily.question;
            Ok(Some(Question {
                id: q.question_frontend_id,
                title: q.title,
                slug: q.title_slug,
                difficulty: q.difficulty,
                is_daily_challenge: true,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_day_seed() -> u64 {
        let now = Utc::now();
        let day_of_year = u64::from(now.ordinal());
        now.year() as u64 * 1000 + day_of_year
    }

    fn pick_seeded_random<T: Clone>(items: &[T], seed: u64) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let mut rng = StdRng::seed_from_u64(seed);
        let index = rng.gen_range(0..items.len());
        Some(items[index].clone())
    }

    pub async fn pick_daily_problem(&self, easy_file: &str, used_file: &str) -> Result<Question> {
        let easy_lines = read_lines(easy_file).await?;
        let used_lines = read_lines(used_file).await?;
        let used_slugs: HashSet<String> = used_lines
            .iter()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        let available: Vec<Question> = easy_lines
            .iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 3 {
                    let id = parts[0].trim().to_string();
                    let title = parts[1].trim().to_string();
                    let slug = parts[2].trim().to_string();
                    if !used_slugs.contains(&slug) {
                        Some(Question {
                            id,
                            title,
                            slug,
                            difficulty: "EASY".to_string(),
                            is_daily_challenge: false,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if available.is_empty() {
            return Err(anyhow!("No available EASY problems found"));
        }

        let day_seed = Self::get_day_seed();
        Self::pick_seeded_random(&available, day_seed)
            .ok_or_else(|| anyhow!("No problem selected"))
    }

    pub async fn get_today_problem(&self, easy_file: &str, used_file: &str) -> Result<Question> {
        if let Some(daily) = self.get_daily_challenge().await? {
            if daily.difficulty.to_uppercase() == "EASY" {
                let used_lines = read_lines(used_file).await?;
                let used_slugs: HashSet<String> = used_lines
                    .iter()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();

                if !used_slugs.contains(&daily.slug) {
                    return Ok(daily);
                }
            }
        }

        self.pick_daily_problem(easy_file, used_file).await
    }

}
