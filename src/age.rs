//! age.rs
//!
//! This module provides a human-readable age calculation in the format:
//!     "X years, Y months, Z days"
//!
//! Chrono does not provide a built-in year/month/day diff (unlike Python’s
//! relativedelta), so we implement the calendar-aware borrowing rules manually.
//!
//! This logic correctly handles:
//!   • month underflow (borrowing from years)
//!   • day underflow (borrowing from previous month)
//!   • leap years
//!   • varying month lengths

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate, Utc};

/// Returns a human age as a string given a birthday date string in YYYY-MM-DD format.
pub fn age_from_birthday(birthday: &str) -> Result<String> {
    let birthdate = NaiveDate::parse_from_str(birthday, "%Y-%m-%d")
        .context("Invalid birthday format, expected YYYY-MM-DD")?;
    let today = Utc::now().date_naive();
    Ok(age_string(birthdate, today))
}

/// Returns a human age as a string
pub(crate) fn age_string(birthdate: NaiveDate, today: NaiveDate) -> String {
    let mut years = today.year() - birthdate.year();
    let mut months = today.month() as i32 - birthdate.month() as i32;
    let mut days = today.day() as i32 - birthdate.day() as i32;

    // Fix day underflow
    if days < 0 {
        months -= 1;

        // Determine the previous month relative to `today`.
        let (prev_year, prev_month) = if today.month() == 1 {
            (today.year() - 1, 12)
        } else {
            (today.year(), today.month() - 1)
        };

        // Add days from the previous month (28–31 depending on month & leap year)
        let days_in_prev_month = days_in_month(prev_year, prev_month);
        days += days_in_prev_month as i32;
    }

    // Fix month underflow
    if months < 0 {
        years -= 1;
        months += 12;
    }

    format!(
        "{} year{}, {} month{}, {} day{}",
        years,
        plural(years),
        months,
        plural(months),
        days,
        plural(days)
    )
}

fn plural(n: i32) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Returns number of days in a given year/month (handles leap years)
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30, // should never occur but keeps function total
    }
}

/// Leap-year rule (Gregorian):
///   - divisible by 4 → leap year
///   - except divisible by 100 → not leap year
///   - except divisible by 400 → leap year
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2000));
        assert!(is_leap_year(1600));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(1800));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 4), 30);
        assert_eq!(days_in_month(2024, 6), 30);
        assert_eq!(days_in_month(2024, 9), 30);
        assert_eq!(days_in_month(2024, 11), 30);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn test_plural() {
        assert_eq!(plural(0), "s");
        assert_eq!(plural(1), "");
        assert_eq!(plural(2), "s");
    }

    #[test]
    fn test_age_string_same_day() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        assert_eq!(age_string(birthdate, today), "0 years, 0 months, 0 days");
    }

    #[test]
    fn test_age_string_one_day_later() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
        assert_eq!(age_string(birthdate, today), "0 years, 0 months, 1 day");
    }

    #[test]
    fn test_age_string_one_month_later() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2020, 2, 1).unwrap();
        assert_eq!(age_string(birthdate, today), "0 years, 1 month, 0 days");
    }

    #[test]
    fn test_age_string_one_year_later() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        assert_eq!(age_string(birthdate, today), "1 year, 0 months, 0 days");
    }

    #[test]
    fn test_age_string_year_month_day() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 1, 15).unwrap();
        let today = NaiveDate::from_ymd_opt(2022, 3, 20).unwrap();
        assert_eq!(age_string(birthdate, today), "2 years, 2 months, 5 days");
    }

    #[test]
    fn test_age_string_month_underflow() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 3, 15).unwrap();
        let today = NaiveDate::from_ymd_opt(2021, 1, 10).unwrap();
        assert_eq!(age_string(birthdate, today), "0 years, 9 months, 26 days");
    }

    #[test]
    fn test_age_string_day_underflow() {
        let birthdate = NaiveDate::from_ymd_opt(2020, 3, 15).unwrap();
        let today = NaiveDate::from_ymd_opt(2020, 4, 10).unwrap();
        assert_eq!(age_string(birthdate, today), "0 years, 0 months, 26 days");
    }

    #[test]
    fn test_age_from_birthday_valid_format() {
        let result = age_from_birthday("1990-01-01");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("year"));
    }

    #[test]
    fn test_age_from_birthday_invalid_format() {
        let result = age_from_birthday("1990-13-32");
        assert!(result.is_err());

        let result = age_from_birthday("not-a-date");
        assert!(result.is_err());
    }
}
