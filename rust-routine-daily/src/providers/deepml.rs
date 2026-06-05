use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::providers::select_problem;
use crate::types::{Difficulty, Platform, ProblemCache, ProblemResult};

const DEEPML_REPO_OWNER: &str = "Open-Deep-ML";
const DEEPML_REPO_NAME: &str = "DML-OpenProblem";
const DEEPML_RAW_BASE: &str =
    "https://raw.githubusercontent.com/Open-Deep-ML/DML-OpenProblem/main/build";
const DEEPML_FILE: &str = "data/deepml_problems.txt";

#[derive(Debug, Deserialize)]
struct DeepMLProblemMeta {
    id: String,
    title: String,
    difficulty: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTreeResponse {
    tree: Vec<GitHubTreeEntry>,
    truncated: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubTreeEntry {
    path: String,
}

pub struct DeepMLProvider {
    client: Client,
    github_token: Option<String>,
}

impl DeepMLProvider {
    fn build_client() -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client build")
    }

    pub fn new() -> Self {
        Self {
            client: Self::build_client(),
            github_token: None,
        }
    }

    pub fn with_token(token: String) -> Self {
        Self {
            client: Self::build_client(),
            github_token: Some(token),
        }
    }

    fn authorize(req: reqwest::RequestBuilder, token: &Option<String>) -> reqwest::RequestBuilder {
        match token {
            Some(token) => req.bearer_auth(token),
            None => req,
        }
    }

    pub async fn sync_problems(&self, output_file: &str) -> Result<()> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/main?recursive=1",
            DEEPML_REPO_OWNER, DEEPML_REPO_NAME
        );

        let response = Self::authorize(
            self.client
                .get(&url)
                .header("User-Agent", "get-up-daily/0.2.0")
                .header("Accept", "application/vnd.github.v3+json"),
            &self.github_token,
        )
        .send()
        .await
        .context("Failed to fetch Deep-ML problem tree from GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "GitHub API returned {} for git tree: {}",
                status,
                body
            ));
        }

        let tree_resp: GitHubTreeResponse = response
            .json()
            .await
            .context("Failed to parse git tree response")?;

        if tree_resp.truncated {
            return Err(anyhow!(
                "Git tree response was truncated; repo is too large for recursive=1"
            ));
        }

        let file_names: Vec<String> = tree_resp
            .tree
            .into_iter()
            .filter_map(|entry| {
                let path = entry.path;
                if path.starts_with("build/") && path.ends_with(".json") {
                    let name = path.strip_prefix("build/")?.to_string();
                    if !name.contains('/') {
                        Some(name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let total = file_names.len();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(16));
        let mut joinset: tokio::task::JoinSet<(String, Option<ProblemCache>)> =
            tokio::task::JoinSet::new();

        for name in file_names {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore not closed");
            let client = self.client.clone();
            let token = self.github_token.clone();
            joinset.spawn(async move {
                let _permit = permit;
                let problem_id = name.trim_end_matches(".json").to_string();
                let url = format!("{}/{}", DEEPML_RAW_BASE, name);
                let req = Self::authorize(
                    client.get(&url).header("User-Agent", "get-up-daily/0.2.0"),
                    &token,
                );
                let res = req.send().await;
                let result = match res {
                    Ok(resp) if resp.status().is_success() => {
                        match resp.json::<DeepMLProblemMeta>().await {
                            Ok(meta) => Difficulty::from_str(&meta.difficulty).map(|difficulty| {
                                ProblemCache {
                                    id: meta.id,
                                    title: meta.title,
                                    slug: format!("deep-ml-problem-{}", problem_id),
                                    difficulty,
                                }
                            }),
                            Err(e) => {
                                eprintln!("Warning: parse failed for {}: {}", problem_id, e);
                                None
                            }
                        }
                    }
                    Ok(resp) => {
                        eprintln!("Warning: HTTP {} for {}", resp.status(), problem_id);
                        None
                    }
                    Err(e) => {
                        eprintln!("Warning: request failed for {}: {}", problem_id, e);
                        None
                    }
                };
                (problem_id, result)
            });
        }

        let mut problems: Vec<ProblemCache> = Vec::with_capacity(total);
        while let Some(joined) = joinset.join_next().await {
            if let Ok((_, Some(cache))) = joined {
                problems.push(cache);
            }
        }
        problems.sort_by(|a, b| a.id.cmp(&b.id));

        let mut output = String::new();
        for p in &problems {
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
            |cache| format!("https://deep-ml.com/problems/{}", cache.id),
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
