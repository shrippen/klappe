//! The times of the scene sheet as typed: the pre-stop as `m:ss` or seconds, the planned
//! shooting time as `h:mm` or minutes.

use crate::error::ServiceError;
use crate::field::Field;

const SECONDS_PER_MINUTE: u32 = 60;
const MINUTES_PER_HOUR: u32 = 60;

/// `text` as a count of lower units: `a:bb` is `a * base + bb`, a plain number is already
/// that count. Empty is none; anything else is a `BadValue` for `field`.
fn read_count(field: Field, text: &str, base: u32) -> Result<Option<u32>, ServiceError> {
    let typed = text.trim();
    if typed.is_empty() {
        return Ok(None);
    }

    let bad = || ServiceError::BadValue {
        field,
        value: typed.to_owned(),
    };
    let digits = |part: &str| {
        let numeric = !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
        numeric.then(|| part.parse::<u32>().ok()).flatten()
    };
    let Some((high, low)) = typed.split_once(':') else {
        return digits(typed).map(Some).ok_or_else(bad);
    };

    let low_ok = low.len() == 2 && digits(low).is_some_and(|n| n < base);
    let count = digits(high)
        .filter(|_| low_ok)
        .and_then(|high| high.checked_mul(base))
        .and_then(|high| high.checked_add(digits(low)?));

    count.map(Some).ok_or_else(bad)
}

/// The pre-stop in seconds.
pub fn read_prestop(text: &str) -> Result<Option<u32>, ServiceError> {
    read_count(Field::Prestop, text, SECONDS_PER_MINUTE)
}

/// The planned shooting time in minutes.
pub fn read_planned(text: &str) -> Result<Option<u32>, ServiceError> {
    read_count(Field::Planned, text, MINUTES_PER_HOUR)
}

/// `count` as `a:bb`; none is empty.
fn show(count: Option<u32>, base: u32) -> String {
    match count {
        Some(count) => format!("{}:{:02}", count / base, count % base),
        None => String::new(),
    }
}

/// The pre-stop as `m:ss`.
pub fn show_prestop(seconds: Option<u32>) -> String {
    show(seconds, SECONDS_PER_MINUTE)
}

/// The planned shooting time as `h:mm`.
pub fn show_planned(minutes: Option<u32>) -> String {
    show(minutes, MINUTES_PER_HOUR)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_of(result: Result<Option<u32>, ServiceError>) -> Field {
        match result {
            Err(ServiceError::BadValue { field, .. }) => field,
            other => panic!("not refused: {other:?}"),
        }
    }

    #[test]
    fn the_pre_stop_reads_m_ss_or_seconds() {
        assert_eq!(read_prestop("1:30").unwrap(), Some(90));
        assert_eq!(read_prestop(" 0:05 ").unwrap(), Some(5));
        assert_eq!(read_prestop("95").unwrap(), Some(95));
    }

    #[test]
    fn the_planned_time_reads_h_mm_or_minutes() {
        assert_eq!(read_planned("1:45").unwrap(), Some(105));
        assert_eq!(read_planned("2:00").unwrap(), Some(120));
        assert_eq!(read_planned("45").unwrap(), Some(45));
    }

    #[test]
    fn empty_is_none() {
        assert_eq!(read_prestop("").unwrap(), None);
        assert_eq!(read_planned("  ").unwrap(), None);
    }

    #[test]
    fn text_that_is_no_time_is_refused_by_its_field() {
        assert_eq!(field_of(read_prestop("1:7x")), Field::Prestop);
        assert_eq!(field_of(read_prestop("1:99")), Field::Prestop);
        assert_eq!(field_of(read_prestop("-5")), Field::Prestop);
        assert_eq!(field_of(read_planned("1:5")), Field::Planned);
        assert_eq!(field_of(read_planned("abc")), Field::Planned);
    }

    #[test]
    fn a_full_minute_is_no_second() {
        assert_eq!(field_of(read_prestop("0:60")), Field::Prestop);
        assert_eq!(field_of(read_planned("0:60")), Field::Planned);
    }

    #[test]
    fn a_count_too_big_is_refused_without_a_panic() {
        assert_eq!(field_of(read_prestop("4294967296")), Field::Prestop);
        assert_eq!(field_of(read_prestop("71582789:00")), Field::Prestop);
        assert_eq!(field_of(read_planned("71582789:00")), Field::Planned);
    }

    #[test]
    fn a_time_shows_as_it_is_typed() {
        assert_eq!(show_prestop(Some(125)), "2:05");
        assert_eq!(show_planned(Some(240)), "4:00");
        assert_eq!(show_prestop(None), "");
    }
}
