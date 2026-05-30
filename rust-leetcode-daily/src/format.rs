use crate::types::{Difficulty, Platform, ProblemResult};

pub fn get_problem_emoji(platform: &Platform, difficulty: &Difficulty) -> &'static str {
    match (platform, difficulty) {
        (Platform::LeetCode, Difficulty::Easy) => "🟢",
        (Platform::LeetCode, Difficulty::Medium) => "🟡",
        (Platform::LeetCode, Difficulty::Hard) => "🔴",
        (Platform::DeepML, Difficulty::Easy) => "🔵",
        (Platform::DeepML, Difficulty::Medium) => "🟣",
        (Platform::DeepML, Difficulty::Hard) => "🟠",
    }
}

pub fn build_formatted_message(
    greeting: &str,
    timestamp: &str,
    year_progress_text: Option<&str>,
    problems: &[ProblemResult],
    running_text: Option<&str>,
    history_text: Option<&str>,
    quote_text: Option<&str>,
    quote_author: Option<&str>,
) -> String {
    let mut parts = vec![format!("{} — {}", greeting, timestamp)];

    if let Some(text) = year_progress_text {
        parts.push(format!("\n{}", text));
    }

    if !problems.is_empty() {
        parts.push("\n📚 Today's Problems".to_string());
        for problem in problems {
            let emoji = get_problem_emoji(&problem.platform, &problem.problem.difficulty);
            let daily_hint = if problem.is_daily_challenge {
                " (daily challenge)"
            } else {
                ""
            };
            parts.push(format!(
                "\n{} {} {}: {}. {}{}\n{}",
                emoji,
                problem.platform,
                problem.problem.difficulty,
                problem.problem.id,
                problem.problem.title,
                daily_hint,
                problem.url
            ));
        }
    }

    if let Some(text) = running_text {
        parts.push(format!("\n{}", text));
    }

    if let Some(text) = history_text {
        parts.push(format!("\n{}", text));
    }

    if let (Some(text), Some(author)) = (quote_text, quote_author) {
        parts.push(format!("\n💬 Today's Quote\n{}\n\n—— {}", text, author));
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_emoji() {
        assert_eq!(get_problem_emoji(&Platform::LeetCode, &Difficulty::Easy), "🟢");
        assert_eq!(get_problem_emoji(&Platform::LeetCode, &Difficulty::Medium), "🟡");
        assert_eq!(get_problem_emoji(&Platform::LeetCode, &Difficulty::Hard), "🔴");
        assert_eq!(get_problem_emoji(&Platform::DeepML, &Difficulty::Easy), "🔵");
        assert_eq!(get_problem_emoji(&Platform::DeepML, &Difficulty::Medium), "🟣");
        assert_eq!(get_problem_emoji(&Platform::DeepML, &Difficulty::Hard), "🟠");
    }
}
