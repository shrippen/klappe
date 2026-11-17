//! The sun's fields as a form hands them over (LH-52): text, blank meaning "none".

use klappe_model::{Site, SunTime, MAX_UTC_OFFSET_MIN};

use crate::error::ServiceError;
use crate::field::Field;
use crate::number::{read_number, Range, Unit};

const MAX_LATITUDE_DEG: f64 = 90.0;
const MAX_LONGITUDE_DEG: f64 = 180.0;
const MINUTES_PER_HOUR: i16 = 60;
const HOURS_PER_DAY: u8 = 24;

/// Latitude, longitude and the turn of the plan to north as typed (`"52,5"`, `"13.4°"`). All
/// three blank is no place; latitude and longitude are needed together; a blank north is 0.
pub fn read_site(
    latitude: &str,
    longitude: &str,
    north: &str,
) -> Result<Option<Site>, ServiceError> {
    if [latitude, longitude, north]
        .iter()
        .all(|t| t.trim().is_empty())
    {
        return Ok(None);
    }

    let latitude_deg = read_number(
        Field::Latitude,
        latitude,
        Unit::Degree,
        Range::Within(-MAX_LATITUDE_DEG, MAX_LATITUDE_DEG),
    )?;
    let longitude_deg = read_number(
        Field::Longitude,
        longitude,
        Unit::Degree,
        Range::Within(-MAX_LONGITUDE_DEG, MAX_LONGITUDE_DEG),
    )?;
    let north_deg = match north.trim().is_empty() {
        true => 0.0,
        false => read_number(Field::North, north, Unit::Degree, Range::Any)?,
    };

    Ok(Some(Site {
        latitude_deg,
        longitude_deg,
        north_deg,
    }))
}

/// Date (`2026-06-21` or `21.6.2026`), time (`14:30`) and offset from UTC (`+02:00`, `-5`; blank
/// follows the location's zone) as typed. All blank is no date; a date needs its time and the
/// other way round.
pub fn read_sun_time(
    date: &str,
    time: &str,
    offset: &str,
) -> Result<Option<SunTime>, ServiceError> {
    if [date, time, offset].iter().all(|t| t.trim().is_empty()) {
        return Ok(None);
    }

    let bad = |field, text: &str| ServiceError::BadValue {
        field,
        value: text.trim().to_owned(),
    };
    let (year, month, day) = read_date(date).ok_or_else(|| bad(Field::Date, date))?;
    let (hour, minute) = read_clock(time).ok_or_else(|| bad(Field::Time, time))?;
    let utc_offset_min = match offset.trim().is_empty() {
        true => None,
        false => Some(read_offset(offset).ok_or_else(|| bad(Field::UtcOffset, offset))?),
    };
    let moment = SunTime {
        year,
        month,
        day,
        hour,
        minute,
        utc_offset_min,
    };
    if !moment.is_valid() {
        let text = format!("{date} {time} {offset}");
        return Err(bad(Field::Date, &text));
    }

    Ok(Some(moment))
}

/// `2026-06-21` (year first) or `21.6.2026` (day first).
fn read_date(text: &str) -> Option<(i32, u8, u8)> {
    let text = text.trim();
    let number = |part: &str| part.trim().parse::<u32>().ok();

    if let Some([year, month, day]) = split3(text, '-') {
        return Some((
            number(year)?.try_into().ok()?,
            number(month)?.try_into().ok()?,
            number(day)?.try_into().ok()?,
        ));
    }
    let [day, month, year] = split3(text, '.')?;

    Some((
        number(year)?.try_into().ok()?,
        number(month)?.try_into().ok()?,
        number(day)?.try_into().ok()?,
    ))
}

/// `14:30` or `9:05`.
fn read_clock(text: &str) -> Option<(u8, u8)> {
    let (hour, minute) = text.trim().split_once(':')?;

    let (hour, minute) = (
        hour.trim().parse::<u8>().ok()?,
        minute.trim().parse::<u8>().ok()?,
    );

    (hour < HOURS_PER_DAY && minute < MINUTES_PER_HOUR as u8).then_some((hour, minute))
}

/// `+02:00`, `-5`, `+5:30`, `2`: hours, and minutes after a colon, in minutes.
fn read_offset(text: &str) -> Option<i16> {
    let text = text.trim();
    let (sign, rest) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.strip_prefix('+').unwrap_or(text)),
    };
    let (hours, minutes) = rest.split_once(':').unwrap_or((rest, "0"));
    let hours = i32::from(hours.trim().parse::<i16>().ok()?);
    let minutes = i32::from(minutes.trim().parse::<i16>().ok()?);
    let total = sign * (hours * i32::from(MINUTES_PER_HOUR) + minutes);

    let in_zone = (0..i32::from(MINUTES_PER_HOUR)).contains(&minutes)
        && total.abs() <= i32::from(MAX_UTC_OFFSET_MIN);

    in_zone.then_some(total as i16)
}

fn split3(text: &str, separator: char) -> Option<[&str; 3]> {
    let mut parts = text.split(separator);
    let found = [parts.next()?, parts.next()?, parts.next()?];

    parts.next().is_none().then_some(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_form_has_no_site() {
        assert!(read_site("", " ", "").unwrap().is_none());
    }

    #[test]
    fn a_site_reads_commas_and_units_and_a_blank_north() {
        let site = read_site("52,52", "13.405°", "").unwrap().unwrap();

        assert_eq!(site.latitude_deg, 52.52);
        assert_eq!(site.longitude_deg, 13.405);
        assert_eq!(site.north_deg, 0.0);
        assert_eq!(
            read_site("-33.9", "151.2", "-90")
                .unwrap()
                .unwrap()
                .north_deg,
            -90.0
        );
    }

    #[test]
    fn a_place_off_the_globe_or_half_a_place_is_refused() {
        for (lat, lon) in [
            ("91", "0"),
            ("0", "181"),
            ("52", ""),
            ("", "13"),
            ("x", "1"),
        ] {
            assert!(read_site(lat, lon, "").is_err(), "{lat} {lon}");
        }
        assert!(read_site("52", "13", "nord").is_err());
    }

    #[test]
    fn a_blank_form_has_no_date() {
        assert!(read_sun_time("", "", "").unwrap().is_none());
    }

    #[test]
    fn date_time_and_offset_read_in_both_orders_of_a_date() {
        let moment = read_sun_time("2026-06-21", "14:30", "+02:00")
            .unwrap()
            .unwrap();

        assert_eq!(
            (
                moment.year,
                moment.month,
                moment.day,
                moment.hour,
                moment.minute
            ),
            (2026, 6, 21, 14, 30)
        );
        assert_eq!(moment.utc_offset_min, Some(120));
        assert_eq!(
            read_sun_time("21.6.2026", "9:05", "")
                .unwrap()
                .unwrap()
                .hour,
            9
        );
        assert_eq!(
            read_sun_time("21.06.2026", "9:05", "")
                .unwrap()
                .unwrap()
                .utc_offset_min,
            None
        );
    }

    #[test]
    fn an_offset_may_be_short_negative_or_with_minutes() {
        let offset = |text| {
            read_sun_time("2026-06-21", "12:00", text)
                .unwrap()
                .unwrap()
                .utc_offset_min
        };

        assert_eq!(offset("-5"), Some(-300));
        assert_eq!(offset("+5:30"), Some(330));
        assert_eq!(offset("2"), Some(120));
    }

    #[test]
    fn a_wrong_date_time_or_offset_names_its_field() {
        let field = |date, time, offset| match read_sun_time(date, time, offset) {
            Err(ServiceError::BadValue { field, .. }) => field,
            other => panic!("{other:?}"),
        };

        assert_eq!(field("2026-02-30", "12:00", ""), Field::Date);
        assert_eq!(field("morgen", "12:00", ""), Field::Date);
        assert_eq!(field("", "12:00", ""), Field::Date);
        assert_eq!(field("2026-06-21", "25:00", ""), Field::Time);
        assert_eq!(field("2026-06-21", "", ""), Field::Time);
        assert_eq!(field("2026-06-21", "12:00", "+99"), Field::UtcOffset);
        assert_eq!(field("2026-06-21", "12:00", "99999999"), Field::UtcOffset);
        assert_eq!(field("2026-06-21", "12:00", "x"), Field::UtcOffset);
        assert_eq!(field("2026-06-21", "12:00", "+2:75"), Field::UtcOffset);
    }
}
