use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::providers::select_problem;
use crate::types::{Difficulty, Platform, ProblemCache, ProblemResult};

const DEEPML_REPO_OWNER: &str = "Open-Deep-ML";
const DEEPML_REPO_NAME: &str = "DML-OpenProblem";
const DEEPML_RAW_BASE: &str = "https://raw.githubusercontent.com/Open-Deep-ML/DML-OpenProblem/main/build";
const DEEPML_FILE: &str = "data/deepml_problems.txt";

#[derive(Debug, Deserialize)]
struct DeepMLProblemMeta {
    id: String,
    title: String,
    difficulty: String,
}

#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
}

pub struct DeepMLProvider {
    client: Client,
}

impl DeepMLProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn sync_problems(&self, output_file: &str) -> Result<()> {
        let mut problems = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "https://api.github.com/repos/{}/{}/contents/build?per_page={}&page={}",
                DEEPML_REPO_OWNER, DEEPML_REPO_NAME, per_page, page
            );

            let response = self
                .client
                .get(&url)
                .header("User-Agent", "get-up-daily/0.2.0")
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await
                .context("Failed to fetch Deep-ML problem list from GitHub API")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(anyhow!(
                    "GitHub API returned {} for problem list: {}",
                    status,
                    body
                ));
            }

            let contents: Vec<GitHubContent> = response
                .json()
                .await
                .context("Failed to parse GitHub API response")?;

            if contents.is_empty() {
                break;
            }

            for item in &contents {
                if item.content_type != "file" || !item.name.ends_with(".json") {
                    continue;
                }

                let problem_id = item.name.trim_end_matches(".json");
                let raw_url = format!("{}/{}", DEEPML_RAW_BASE, item.name);

                match self.fetch_problem_meta(&raw_url).await {
                    Ok(meta) => {
                        if let Some(difficulty) = Difficulty::from_str(&meta.difficulty) {
                            problems.push(ProblemCache {
                                id: meta.id,
                                title: meta.title,
                                slug: format!("deep-ml-problem-{}", problem_id),
                                difficulty,
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to fetch problem {}: {}", problem_id, e);
                    }
                }
            }

            if contents.len() < per_page {
                break;
            }
            page += 1;
        }

        let mut output = String::new();
        for p in problems {
            output.push_str(&format!(
                "{}|{}|{}|{}\n",
                p.id,
                p.title,
                p.slug,
                p.difficulty.as_str()
            ));
        }

        tokio::fs::write(output_file, output.trim()).await?;
        Ok(())
    }

    async fn fetch_problem_meta(&self, url: &str) -> Result<DeepMLProblemMeta> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "get-up-daily/0.2.0")
            .send()
            .await
            .context("Failed to fetch problem metadata")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch problem metadata: HTTP {}",
                response.status()
            ));
        }

        let meta: DeepMLProblemMeta = response
            .json()
            .await
            .context("Failed to parse problem metadata")?;

        Ok(meta)
    }

    fn get_day_seed() -> u64 {
        let now = Utc::now();
        let day_of_year = u64::from(now.ordinal());
        now.year() as u64 * 1000 + day_of_year + 1000000
    }

    pub async fn get_problem(
        &self,
        used_file: &str,
        difficulty: Difficulty,
    ) -> Result<ProblemResult> {
        select_problem(
            DEEPML_FILE,
            used_file,
            difficulty,
            Platform::DeepML,
            |cache| format!("https://deep-ml.com/problem/{}", cache.id),
            Self::get_day_seed(),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_day_seed_structure() {
        let seed = DeepMLProvider::get_day_seed();
        assert!(seed >= 2024001 + 1000000, "seed {} looks wrong", seed);
    }
}
