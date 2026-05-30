use anyhow::Result;
use chrono::{Datelike, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::{Config, LeetCodeVariant};
use crate::providers::select_problem;
use crate::types::{Difficulty, Platform, Problem, ProblemResult};

const LEETCODE_EASY_FILE: &str = "data/leetcode_easy.txt";
const LEETCODE_MEDIUM_FILE: &str = "data/leetcode_medium.txt";
const LEETCODE_HARD_FILE: &str = "data/leetcode_hard.txt";

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
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

pub struct LeetCodeProvider {
    client: Client,
    endpoint: String,
    variant: LeetCodeVariant,
}

impl LeetCodeProvider {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            endpoint: config.leetcode_endpoint.clone(),
            variant: config.leetcode_variant,
        }
    }

    pub async fn fetch_problem_list(
        &self,
        difficulty: Difficulty,
        output_file: &str,
    ) -> Result<()> {
        let level = match difficulty {
            Difficulty::Easy => 1,
            Difficulty::Medium => 2,
            Difficulty::Hard => 3,
        };

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
            .header("User-Agent", "LeetCodeDaily/0.2.0")
            .send()
            .await?
            .json()
            .await?;

        let mut output = String::new();
        for p in resp.stat_status_pairs {
            if p.difficulty.level == level && !p.paid_only {
                output.push_str(&format!(
                    "{}|{}|{}|{}\n",
                    p.stat.frontend_question_id,
                    p.stat.question__title,
                    p.stat.question__title_slug,
                    difficulty.as_str()
                ));
            }
        }

        tokio::fs::write(output_file, output.trim()).await?;
        Ok(())
    }

    pub async fn fetch_easy_list(&self, output_file: &str) -> Result<()> {
        self.fetch_problem_list(Difficulty::Easy, output_file).await
    }

    pub async fn fetch_medium_list(&self, output_file: &str) -> Result<()> {
        self.fetch_problem_list(Difficulty::Medium, output_file).await
    }

    pub async fn fetch_hard_list(&self, output_file: &str) -> Result<()> {
        self.fetch_problem_list(Difficulty::Hard, output_file).await
    }

    async fn get_daily_challenge(&self) -> Result<Option<Problem>> {
        match self.variant {
            LeetCodeVariant::Cn => self.get_daily_challenge_cn().await,
            LeetCodeVariant::Com => self.get_daily_challenge_com().await,
        }
    }

    async fn get_daily_challenge_com(&self) -> Result<Option<Problem>> {
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
            .header("User-Agent", "LeetCodeDaily/0.2.0")
            .json(&request)
            .send()
            .await?;

        let graphql_response: GraphQLResponse<DailyChallengeData> = response.json().await?;

        if let Some(daily) = graphql_response.data.active_daily_coding_challenge_question {
            let q = daily.question;
            let difficulty = Difficulty::from_str(&q.difficulty).unwrap_or(Difficulty::Easy);
            Ok(Some(Problem {
                id: q.question_frontend_id,
                title: q.title,
                slug: q.title_slug,
                difficulty,
                is_daily_challenge: true,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_daily_challenge_cn(&self) -> Result<Option<Problem>> {
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
            .header("User-Agent", "LeetCodeDaily/0.2.0")
            .json(&request)
            .send()
            .await?;

        let graphql_response: GraphQLResponse<CnDailyChallengeData> = response.json().await?;

        if let Some(daily) = graphql_response.data.today_record.into_iter().next() {
            let q = daily.question;
            let difficulty = Difficulty::from_str(&q.difficulty).unwrap_or(Difficulty::Easy);
            Ok(Some(Problem {
                id: q.question_frontend_id,
                title: q.title,
                slug: q.title_slug,
                difficulty,
                is_daily_challenge: true,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_cache_file(difficulty: Difficulty) -> &'static str {
        match difficulty {
            Difficulty::Easy => LEETCODE_EASY_FILE,
            Difficulty::Medium => LEETCODE_MEDIUM_FILE,
            Difficulty::Hard => LEETCODE_HARD_FILE,
        }
    }

    fn get_day_seed() -> u64 {
        let now = Utc::now();
        let day_of_year = u64::from(now.ordinal());
        now.year() as u64 * 1000 + day_of_year
    }

    fn make_url(&self, slug: &str) -> String {
        match self.variant {
            LeetCodeVariant::Cn => format!("https://leetcode.cn/problems/{}/", slug),
            LeetCodeVariant::Com => format!("https://leetcode.com/problems/{}/", slug),
        }
    }

    pub async fn get_problem(
        &self,
        used_file: &str,
        difficulty: Difficulty,
    ) -> Result<ProblemResult> {
        if let Some(daily) = self.get_daily_challenge().await? {
            if daily.difficulty == difficulty {
                let used_lines = read_lines(used_file).await?;
                let used_slugs: HashSet<String> = used_lines
                    .iter()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();

                if !used_slugs.contains(&daily.slug) {
                    let url = self.make_url(&daily.slug);
                    return Ok(ProblemResult {
                        platform: Platform::LeetCode,
                        problem: daily,
                        url,
                        is_daily_challenge: true,
                    });
                }
            }
        }

        let cache_file = Self::get_cache_file(difficulty);
        let variant = self.variant;
        select_problem(
            cache_file,
            used_file,
            difficulty,
            Platform::LeetCode,
            |cache| match variant {
                LeetCodeVariant::Cn => format!("https://leetcode.cn/problems/{}/", cache.slug),
                LeetCodeVariant::Com => format!("https://leetcode.com/problems/{}/", cache.slug),
            },
            Self::get_day_seed(),
        )
        .await
    }
}

use std::collections::HashSet;
use crate::utils::read_lines;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_day_seed_structure() {
        let seed = LeetCodeProvider::get_day_seed();
        assert!(seed >= 2024001, "seed {} looks wrong (expected >= 2024001)", seed);
        let year_part = seed / 1000;
        let day_part = seed % 1000;
        assert!(year_part >= 2024, "year part {} looks wrong", year_part);
        assert!(day_part >= 1 && day_part <= 366, "day part {} out of range", day_part);
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

    #[tokio::test]
    async fn test_select_problem() {
        let tmp = std::env::temp_dir();
        let easy_file = tmp.join("test_easy_provider.txt");
        let used_file = tmp.join("test_used_provider.txt");

        tokio::fs::write(&easy_file, "1|First|first|Easy\n2|Second|second|Easy\n3|Third|third|Easy")
            .await
            .unwrap();
        tokio::fs::write(&used_file, "second").await.unwrap();

        let result = crate::providers::select_problem(
            easy_file.to_str().unwrap(),
            used_file.to_str().unwrap(),
            Difficulty::Easy,
            Platform::LeetCode,
            |cache| format!("https://leetcode.com/problems/{}/", cache.slug),
            42,
        )
        .await
        .unwrap();

        assert_ne!(result.problem.slug, "second");
        assert!(result.problem.slug == "first" || result.problem.slug == "third");
        assert_eq!(result.problem.difficulty, Difficulty::Easy);

        let _ = tokio::fs::remove_file(&easy_file).await;
        let _ = tokio::fs::remove_file(&used_file).await;
    }
}
