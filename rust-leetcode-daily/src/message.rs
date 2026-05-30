pub fn get_greeting(hour: u32) -> &'static str {
    match hour {
        3..=11 => "☀️ Good morning",
        12..=17 => "⛅ Good afternoon",
        _ => "🌙 Good evening",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
