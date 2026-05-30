use chrono::{Datelike, Weekday};
use rand::rngs::StdRng;
use rand::{seq::SliceRandom, SeedableRng};

use crate::types::{Difficulty, Platform};

/// A day's problem schedule for a single platform.
pub enum Schedule {
    Weekday { difficulty: Difficulty },
    Weekend { difficulties: [Difficulty; 2] },
}

impl Schedule {
    pub fn iter(&self) -> impl Iterator<Item = &Difficulty> {
        match self {
            Schedule::Weekday { difficulty } => std::slice::from_ref(difficulty).iter(),
            Schedule::Weekend { difficulties } => difficulties.iter(),
        }
    }
}

/// Generates a deterministic weekday difficulty pattern.
/// Returns 5 difficulties (3 Easy + 2 Medium) in random order.
fn generate_weekday_difficulties(seed: u64) -> Vec<Difficulty> {
    let mut difficulties = vec![
        Difficulty::Easy,
        Difficulty::Easy,
        Difficulty::Easy,
        Difficulty::Medium,
        Difficulty::Medium,
    ];
    let mut rng = StdRng::seed_from_u64(seed);
    difficulties.shuffle(&mut rng);
    difficulties
}

/// Determines the week seed for deterministic scheduling.
fn get_week_seed(date: &chrono::DateTime<chrono_tz::Tz>) -> u64 {
    let year = date.year() as u64;
    let week = date.iso_week().week() as u64;
    year * 100 + week
}

/// Produces a schedule for a platform on a given date.
pub fn get_schedule(date: &chrono::DateTime<chrono_tz::Tz>, platform: Platform) -> Schedule {
    match date.weekday() {
        Weekday::Sat | Weekday::Sun => Schedule::Weekend {
            difficulties: [Difficulty::Medium, Difficulty::Hard],
        },
        _ => {
            let week_seed = get_week_seed(date);
            let day_index = match date.weekday() {
                Weekday::Mon => 0,
                Weekday::Tue => 1,
                Weekday::Wed => 2,
                Weekday::Thu => 3,
                Weekday::Fri => 4,
                _ => 0,
            };
            let platform_offset = match platform {
                Platform::LeetCode => 0,
                Platform::DeepML => 5,
            };
            let combined_seed = week_seed + platform_offset;
            let difficulties = generate_weekday_difficulties(combined_seed);
            Schedule::Weekday {
                difficulty: difficulties[day_index],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_weekday_difficulties_length() {
        let diffs = generate_weekday_difficulties(123);
        assert_eq!(diffs.len(), 5);
        let easy_count = diffs.iter().filter(|d| **d == Difficulty::Easy).count();
        let medium_count = diffs.iter().filter(|d| **d == Difficulty::Medium).count();
        assert_eq!(easy_count, 3);
        assert_eq!(medium_count, 2);
    }

    #[test]
    fn test_weekday_difficulties_deterministic() {
        let diffs1 = generate_weekday_difficulties(42);
        let diffs2 = generate_weekday_difficulties(42);
        assert_eq!(diffs1, diffs2);
    }

    #[test]
    fn test_weekday_difficulties_different_seeds() {
        let diffs1 = generate_weekday_difficulties(1);
        let diffs2 = generate_weekday_difficulties(2);
        assert_ne!(diffs1, diffs2);
    }

    #[test]
    fn test_weekend_schedule() {
        let tz: chrono_tz::Tz = "UTC".parse().unwrap();
        let saturday = tz.with_ymd_and_hms(2024, 1, 6, 0, 0, 0).unwrap();
        let sunday = tz.with_ymd_and_hms(2024, 1, 7, 0, 0, 0).unwrap();

        let sat_lc = get_schedule(&saturday, Platform::LeetCode);
        let sun_dm = get_schedule(&sunday, Platform::DeepML);

        let sat_items: Vec<_> = sat_lc.iter().collect();
        let sun_items: Vec<_> = sun_dm.iter().collect();

        assert_eq!(sat_items.len(), 2);
        assert_eq!(sun_items.len(), 2);
        assert!(sat_items.contains(&&Difficulty::Medium));
        assert!(sat_items.contains(&&Difficulty::Hard));
    }

    #[test]
    fn test_weekday_schedule() {
        let tz: chrono_tz::Tz = "UTC".parse().unwrap();
        let monday = tz.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let schedule = get_schedule(&monday, Platform::LeetCode);
        let items: Vec<_> = schedule.iter().collect();
        assert_eq!(items.len(), 1);
        assert!(matches!(*items[0], Difficulty::Easy | Difficulty::Medium));
    }
}
