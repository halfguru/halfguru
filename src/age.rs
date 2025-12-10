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

use chrono::{Datelike, NaiveDate};

/// Returns a human age as a string
pub fn age_string(birthdate: NaiveDate, today: NaiveDate) -> String {
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
