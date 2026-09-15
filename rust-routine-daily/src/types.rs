use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

impl std::str::FromStr for Difficulty {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            other => Err(format!("Unknown difficulty: {}", other)),
        }
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    LeetCode,
    DeepML,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::LeetCode => write!(f, "LeetCode"),
            Platform::DeepML => write!(f, "Deep-ML"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub difficulty: Difficulty,
    pub is_daily_challenge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemResult {
    pub platform: Platform,
    pub problem: Problem,
    pub url: String,
    pub is_daily_challenge: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_display: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProblemCache {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub difficulty: Difficulty,
}

impl ProblemCache {
    pub fn to_problem(&self, is_daily_challenge: bool) -> Problem {
        Problem {
            id: self.id.clone(),
            title: self.title.clone(),
            slug: self.slug.clone(),
            difficulty: self.difficulty,
            is_daily_challenge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_str() {
        assert_eq!("easy".parse::<Difficulty>(), Ok(Difficulty::Easy));
        assert_eq!("EASY".parse::<Difficulty>(), Ok(Difficulty::Easy));
        assert_eq!("Medium".parse::<Difficulty>(), Ok(Difficulty::Medium));
        assert_eq!("HARD".parse::<Difficulty>(), Ok(Difficulty::Hard));
        assert!("garbage".parse::<Difficulty>().is_err());
    }

    #[test]
    fn test_difficulty_display() {
        assert_eq!(format!("{}", Difficulty::Easy), "Easy");
        assert_eq!(format!("{}", Difficulty::Medium), "Medium");
        assert_eq!(format!("{}", Difficulty::Hard), "Hard");
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(format!("{}", Platform::LeetCode), "LeetCode");
        assert_eq!(format!("{}", Platform::DeepML), "Deep-ML");
    }
}
