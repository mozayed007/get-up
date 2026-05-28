use crate::api::RunningStats;
use crate::config::LeetCodeVariant;
use crate::leetcode::Question;

#[allow(clippy::too_many_arguments)]
pub fn build_message(
    greeting: &str,
    get_up_time: &str,
    day_of_year: u32,
    year_progress: &str,
    leetcode: &str,
    running_info: &str,
    history_today: &str,
    quote: &str,
) -> String {
    format!(
        r#"{} — {}

Day {} · {}

{}

{}

{}

💬 Today's Quote
{}"#,
        greeting,
        get_up_time,
        day_of_year,
        year_progress,
        leetcode,
        running_info,
        history_today,
        quote
    )
}

pub fn get_greeting(hour: u32) -> &'static str {
    match hour {
        3..=11 => "☀️ Good morning",
        12..=17 => "⛅ Good afternoon",
        _ => "🌙 Good evening",
    }
}

pub fn format_problem_message(problem: &Question, variant: &LeetCodeVariant) -> String {
    let url = match variant {
        LeetCodeVariant::Cn => format!("https://leetcode.cn/problems/{}/", problem.slug),
        LeetCodeVariant::Com => format!("https://leetcode.com/problems/{}/", problem.slug),
    };

    let daily_hint = if problem.is_daily_challenge {
        " (daily challenge)"
    } else {
        ""
    };

    format!(
        "🟢 LeetCode EASY: {}. {}{}\n{}",
        problem.id, problem.title, daily_hint, url
    )
}

pub fn format_running(stats: &RunningStats) -> String {
    format!(
        "🏃 Yesterday: {:.2} km · This month: {:.2} km · This year: {:.2} km",
        stats.yesterday_km, stats.month_km, stats.year_km
    )
}

pub fn format_history(history: &[String]) -> String {
    if history.is_empty() {
        String::new()
    } else {
        let mut out = "📜 On this day:".to_string();
        for h in history {
            out.push('\n');
            out.push_str(h);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_message() {
        let msg = build_message(
            "☀️ GM",
            "07:30",
            42,
            "42/365",
            "LC msg",
            "Run info",
            "History",
            "Quote",
        );
        assert!(msg.contains("07:30"));
        assert!(msg.contains("Day 42"));
        assert!(msg.contains("LC msg"));
        assert!(msg.contains("💬 Today's Quote"));
        assert!(msg.contains("Quote"));
    }

    #[test]
    fn test_greeting_morning() {
        assert_eq!(get_greeting(3), "☀️ Good morning");
        assert_eq!(get_greeting(8), "☀️ Good morning");
        assert_eq!(get_greeting(11), "☀️ Good morning");
    }

    #[test]
    fn test_greeting_afternoon() {
        assert_eq!(get_greeting(12), "⛅ Good afternoon");
        assert_eq!(get_greeting(14), "⛅ Good afternoon");
        assert_eq!(get_greeting(17), "⛅ Good afternoon");
    }

    #[test]
    fn test_greeting_evening() {
        assert_eq!(get_greeting(18), "🌙 Good evening");
        assert_eq!(get_greeting(0), "🌙 Good evening");
        assert_eq!(get_greeting(2), "🌙 Good evening");
    }

    #[test]
    fn test_format_problem_message_com() {
        let q = Question {
            id: "42".into(),
            title: "Test".into(),
            slug: "test".into(),
            difficulty: "EASY".into(),
            is_daily_challenge: false,
        };
        let msg = format_problem_message(&q, &LeetCodeVariant::Com);
        assert!(msg.contains("https://leetcode.com/problems/test/"));
        assert!(msg.contains("42. Test"));
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
        assert!(msg.contains("(daily challenge)"));
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
        assert!(msg.contains("42.00 km"));
        assert!(msg.contains("300.00 km"));
    }

    #[test]
    fn test_format_running_zero() {
        let stats = RunningStats::default();
        let msg = format_running(&stats);
        assert!(msg.contains("0.00 km"));
    }

    #[test]
    fn test_format_history_empty() {
        assert_eq!(format_history(&[]), "");
    }

    #[test]
    fn test_format_history_non_empty() {
        let h = vec!["• 2020: Event".into()];
        let msg = format_history(&h);
        assert!(msg.contains("On this day"));
        assert!(msg.contains("2020: Event"));
    }
}
