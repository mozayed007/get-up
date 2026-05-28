use crate::api::RunningStats;
use crate::config::LeetCodeVariant;
use crate::leetcode::Question;

pub fn build_message(
    get_up_time: &str,
    day_of_year: u32,
    year_progress: &str,
    leetcode: &str,
    running_info: &str,
    history_today: &str,
    quote: &str,
) -> String {
    format!(
        r#"Wake up time: {}

Good morning!

Day {} of the year.

{}

{}

{}

{}

Today's Quote:
{}"#,
        get_up_time,
        day_of_year,
        year_progress,
        leetcode,
        running_info,
        history_today,
        quote,
    )
}

pub fn format_problem_message(problem: &Question, variant: &LeetCodeVariant) -> String {
    let url = match variant {
        LeetCodeVariant::Cn => format!("https://leetcode.cn/problems/{}/", problem.slug),
        LeetCodeVariant::Com => format!("https://leetcode.com/problems/{}/", problem.slug),
    };

    let difficulty_emoji = if problem.difficulty.to_uppercase() == "EASY" {
        "🟢"
    } else {
        ""
    };

    let daily_hint = if problem.is_daily_challenge {
        "\nOfficial daily challenge!"
    } else {
        ""
    };

    let markdown_link = format!("[{}. {}]({})", problem.id, problem.title, url);

    format!(
        "{} Today's LeetCode EASY:\n{}{}\nKeep going! 🚀",
        difficulty_emoji, markdown_link, daily_hint
    )
}

pub fn format_running(stats: &RunningStats) -> String {
    format!(
        "🏃 Running Stats:\nYesterday: {:.2} km ({} sessions)\nThis month: {:.2} km ({} sessions)\nThis year: {:.2} km ({} sessions)",
        stats.yesterday_km,
        stats.yesterday_count,
        stats.month_km,
        stats.month_count,
        stats.year_km,
        stats.year_count
    )
}

pub fn format_history(history: &[String]) -> String {
    if history.is_empty() {
        "No historical events found".to_string()
    } else {
        history.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_message() {
        let msg = build_message("07:30", 42, "42/365 (11.5%) ███░░", "LeetCode msg", "Running info", "History line", "Quote text");
        assert!(msg.contains("07:30"));
        assert!(msg.contains("Day 42 of the year"));
        assert!(msg.contains("42/365 (11.5%)"));
        assert!(msg.contains("LeetCode msg"));
        assert!(msg.contains("Running info"));
        assert!(msg.contains("History line"));
        assert!(msg.contains("Quote text"));
    }

    #[test]
    fn test_format_problem_message_com() {
        let q = Question {
            id: "42".into(),
            title: "Test Problem".into(),
            slug: "test-problem".into(),
            difficulty: "EASY".into(),
            is_daily_challenge: false,
        };
        let msg = format_problem_message(&q, &LeetCodeVariant::Com);
        assert!(msg.contains("https://leetcode.com/problems/test-problem/"));
        assert!(msg.contains("🟢"));
        assert!(msg.contains("42. Test Problem"));
        assert!(!msg.contains("Official daily challenge"));
    }

    #[test]
    fn test_format_problem_message_cn() {
        let q = Question {
            id: "1".into(),
            title: "Two Sum".into(),
            slug: "two-sum".into(),
            difficulty: "EASY".into(),
            is_daily_challenge: true,
        };
        let msg = format_problem_message(&q, &LeetCodeVariant::Cn);
        assert!(msg.contains("https://leetcode.cn/problems/two-sum/"));
        assert!(msg.contains("1. Two Sum"));
        assert!(msg.contains("Official daily challenge"));
    }

    #[test]
    fn test_format_problem_message_non_easy_no_emoji() {
        let q = Question {
            id: "10".into(),
            title: "Hard Problem".into(),
            slug: "hard-problem".into(),
            difficulty: "HARD".into(),
            is_daily_challenge: false,
        };
        let msg = format_problem_message(&q, &LeetCodeVariant::Com);
        assert!(!msg.contains("🟢"));
    }

    #[test]
    fn test_format_running() {
        let stats = RunningStats {
            yesterday_km: 5.2,
            yesterday_count: 1,
            month_km: 42.0,
            month_count: 10,
            year_km: 300.0,
            year_count: 72,
        };
        let msg = format_running(&stats);
        assert!(msg.contains("5.20 km"));
        assert!(msg.contains("1 sessions"));
        assert!(msg.contains("42.00 km"));
        assert!(msg.contains("10 sessions"));
        assert!(msg.contains("300.00 km"));
        assert!(msg.contains("72 sessions"));
    }

    #[test]
    fn test_format_running_zero() {
        let stats = RunningStats {
            yesterday_km: 0.0,
            yesterday_count: 0,
            month_km: 0.0,
            month_count: 0,
            year_km: 0.0,
            year_count: 0,
        };
        let msg = format_running(&stats);
        assert!(msg.contains("0.00 km"));
    }

    #[test]
    fn test_format_history_empty() {
        assert_eq!(format_history(&[]), "No historical events found");
    }

    #[test]
    fn test_format_history_non_empty() {
        let h = vec!["• 2020: Event".into(), "• 2021: Another".into()];
        let msg = format_history(&h);
        assert_eq!(msg, "• 2020: Event\n• 2021: Another");
    }
}
