use anyhow::{anyhow, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

use crate::types::{Difficulty, Platform, ProblemCache, ProblemResult};
use crate::utils::read_lines;

pub mod deepml;
pub mod leetcode;

/// Select a problem from a cache file with deterministic seeding.
pub async fn select_problem(
    cache_file: &str,
    used_file: &str,
    difficulty: Difficulty,
    platform: Platform,
    url_generator: impl Fn(&ProblemCache) -> String,
    seed: u64,
) -> Result<ProblemResult> {
    let lines = read_lines(cache_file).await?;
    let used_lines = read_lines(used_file).await?;
    let used_slugs: HashSet<String> = used_lines
        .iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let available: Vec<ProblemCache> = lines
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                let id = parts[0].trim().to_string();
                let title = parts[1].trim().to_string();
                let slug = parts[2].trim().to_string();
                let line_difficulty = Difficulty::from_str(parts[3].trim())?;
                if !used_slugs.contains(&slug) && line_difficulty == difficulty {
                    Some(ProblemCache {
                        id,
                        title,
                        slug,
                        difficulty,
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
        return Err(anyhow!(
            "No available {:?} problems found for difficulty: {}",
            platform,
            difficulty
        ));
    }

    let selected = pick_seeded_random(&available, seed)
        .ok_or_else(|| anyhow!("No {:?} problem selected", platform))?;

    let url = url_generator(&selected);

    Ok(ProblemResult {
        platform,
        problem: selected.to_problem(false),
        url,
        is_daily_challenge: false,
    })
}

fn pick_seeded_random<T: Clone>(items: &[T], seed: u64) -> Option<T> {
    if items.is_empty() {
        return None;
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let index = rng.gen_range(0..items.len());
    Some(items[index].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_seeded_random_deterministic() {
        let items = vec!["a", "b", "c", "d", "e"];
        let result1 = pick_seeded_random(&items, 42);
        let result2 = pick_seeded_random(&items, 42);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_pick_seeded_random_empty() {
        let empty: Vec<i32> = vec![];
        assert!(pick_seeded_random(&empty, 0).is_none());
    }

    #[test]
    fn test_pick_seeded_random_different_seeds() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let results: std::collections::HashSet<i32> = (0..50)
            .map(|s| pick_seeded_random(&items, s).unwrap())
            .collect();
        assert!(results.len() >= 3, "only {} unique values from 50 seeds", results.len());
    }
}
