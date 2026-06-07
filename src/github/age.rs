//! Age calculation. Day-precise (improved over the original month-only version).

use crate::config::{BIRTH_DAY, BIRTH_MONTH, BIRTH_YEAR};
use chrono::{Datelike, Local};

/// Full years elapsed since birth, accounting for the exact birthday (day-precise).
pub fn age_data() -> i64 {
    let now = Local::now();
    age_for(now.year() as i64, now.month() as i64, now.day() as i64)
}

/// Pure arithmetic, testable with fixed inputs. Month/day are 1-indexed.
fn age_for(year: i64, month: i64, day: i64) -> i64 {
    let mut age = year - BIRTH_YEAR;
    // Subtract a year if the birthday hasn't occurred yet this year.
    if (month, day) < (BIRTH_MONTH, BIRTH_DAY) {
        age -= 1;
    }
    age
}

#[cfg(test)]
mod tests {
    use super::age_for;

    #[test]
    fn birthday_logic() {
        // Born 2007-03-21.
        assert_eq!(age_for(2026, 6, 7), 19); // today: well after birthday
        assert_eq!(age_for(2027, 3, 20), 19); // day before 20th birthday
        assert_eq!(age_for(2027, 3, 21), 20); // exactly on 20th birthday
        assert_eq!(age_for(2027, 1, 1), 19); // early in the year, not yet
        assert_eq!(age_for(2026, 3, 21), 19); // 19th birthday
    }
}
