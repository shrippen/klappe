//! Where the sun stands over a location at a moment (LH-52), offline.
//!
//! The low-precision series of the NOAA solar calculator (after Meeus, Astronomical
//! Algorithms, ch. 25 and 28): the position of the sun within about 0.1° for the years 1900
//! to 2100, far more than a plan shows. No refraction: the sun is up when its centre is above
//! the geometric horizon.

use klappe_model::{Point, Site, SunTime};

const JULIAN_DAY_UNIX_EPOCH: f64 = 2_440_587.5;
const JULIAN_DAY_J2000: f64 = 2_451_545.0;
const DAYS_PER_CENTURY: f64 = 36_525.0;
const MINUTES_PER_DAY: f64 = 1_440.0;
const MINUTES_PER_HOUR: f64 = 60.0;
const MARCH: u8 = 3;
const OCTOBER: u8 = 10;
/// Both months end on the 31st.
const MONTH_END_DAY: u8 = 31;
/// 1970-01-01 was a Thursday; Sunday is 0.
const UNIX_EPOCH_WEEKDAY: i64 = 4;
const DAYS_PER_WEEK: i64 = 7;
const MINUTES_PER_DAY_INT: i64 = 1_440;
const MINUTES_PER_HOUR_INT: i64 = 60;
/// Summer time switches at 01:00 UTC.
const SWITCH_UTC_MIN: i64 = 60;
const CET_MIN: i64 = 60;
const CEST_MIN: i64 = 120;
/// The Earth turns 15° an hour: one degree of longitude is four minutes of solar time.
const MINUTES_PER_DEG: f64 = 4.0;
const HALF_TURN_DEG: f64 = 180.0;
const FULL_TURN_DEG: f64 = 360.0;

/// The sun's direction from a place: azimuth clockwise from north (0 north, 90 east, 180
/// south), elevation above the horizon (negative below it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sun {
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
}

impl Sun {
    /// Whether any of the sun is above the horizon.
    pub fn is_up(&self) -> bool {
        self.elevation_deg > 0.0
    }

    /// The unit vector on the plan that points from `site` towards the sun's foot point.
    /// The compass bearing is turned by the plan's north: 0° bearing is `north_deg` clockwise of
    /// the plan's top, and the plan's y points down.
    pub fn toward_on_plan(&self, site: &Site) -> Point {
        let (sin, cos) = (self.azimuth_deg + site.north_deg).to_radians().sin_cos();

        Point::new(sin, -cos)
    }
}

/// The sun over `site` at the wall-clock moment `time`.
pub fn sun_at(site: &Site, time: &SunTime) -> Sun {
    let offset_min = time
        .utc_offset_min
        .map_or_else(|| central_european_offset_min(time), f64::from);
    let utc_min = f64::from(time.hour) * MINUTES_PER_HOUR + f64::from(time.minute) - offset_min;
    let day = days_from_civil(time.year, time.month, time.day) as f64;
    let julian_day = day + JULIAN_DAY_UNIX_EPOCH + utc_min / MINUTES_PER_DAY;

    let (declination_deg, equation_min) = solar_terms(julian_day);

    // Solar time runs ahead of UTC by four minutes per degree east, corrected by the equation
    // of time; at 12:00 solar time the hour angle is 0.
    let solar_min =
        (utc_min + equation_min + MINUTES_PER_DEG * site.longitude_deg).rem_euclid(MINUTES_PER_DAY);
    let hour_angle = (solar_min / MINUTES_PER_DEG - HALF_TURN_DEG).to_radians();

    let (lat, decl) = (site.latitude_deg.to_radians(), declination_deg.to_radians());
    let sin_elevation = lat.sin() * decl.sin() + lat.cos() * decl.cos() * hour_angle.cos();

    // Measured from the south towards the west, then turned to the north.
    let from_south = hour_angle
        .sin()
        .atan2(hour_angle.cos() * lat.sin() - decl.tan() * lat.cos());

    Sun {
        azimuth_deg: (from_south.to_degrees() + HALF_TURN_DEG).rem_euclid(FULL_TURN_DEG),
        elevation_deg: sin_elevation.asin().to_degrees(),
    }
}

/// The clock of Central Europe at `time` when the user gave no offset: +01:00, and +02:00 while
/// summer time runs, from the last Sunday of March 01:00 UTC to the last Sunday of October
/// 01:00 UTC. Pure rule, no zone database; elsewhere the user types the offset. The clock is
/// read as winter time first: 02:30 on the March switch day, which no clock shows, counts as
/// summer time (00:30 UTC); 02:30 on the October day, shown twice, is the second time (01:30
/// UTC). The first one needs "+02:00".
fn central_european_offset_min(time: &SunTime) -> f64 {
    let last_sunday_utc_min = |month: u8| {
        let last = days_from_civil(time.year, month, MONTH_END_DAY);
        let weekday = (last + UNIX_EPOCH_WEEKDAY).rem_euclid(DAYS_PER_WEEK);

        (last - weekday) * MINUTES_PER_DAY_INT + SWITCH_UTC_MIN
    };
    let clock_min = days_from_civil(time.year, time.month, time.day) * MINUTES_PER_DAY_INT
        + i64::from(time.hour) * MINUTES_PER_HOUR_INT
        + i64::from(time.minute);
    // The instant the clock shows if it is winter time.
    let utc_min = clock_min - CET_MIN;
    let summer = (last_sunday_utc_min(MARCH)..last_sunday_utc_min(OCTOBER)).contains(&utc_min);

    match summer {
        true => CEST_MIN as f64,
        false => CET_MIN as f64,
    }
}

/// The sun's declination in degrees and the equation of time in minutes at `julian_day`.
fn solar_terms(julian_day: f64) -> (f64, f64) {
    let t = (julian_day - JULIAN_DAY_J2000) / DAYS_PER_CENTURY;

    let mean_longitude = (280.46646 + t * (36000.76983 + t * 0.0003032)).rem_euclid(FULL_TURN_DEG);
    let anomaly = (357.52911 + t * (35999.05029 - 0.0001537 * t)).to_radians();
    let eccentricity = 0.016708634 - t * (0.000042037 + 0.0000001267 * t);
    let centre = anomaly.sin() * (1.914602 - t * (0.004817 + 0.000014 * t))
        + (2.0 * anomaly).sin() * (0.019993 - 0.000101 * t)
        + (3.0 * anomaly).sin() * 0.000289;

    // Apparent longitude: corrected for the nutation and the aberration.
    let node = (125.04 - 1934.136 * t).to_radians();
    let apparent = (mean_longitude + centre - 0.00569 - 0.00478 * node.sin()).to_radians();
    let obliquity_mean =
        23.0 + (26.0 + (21.448 - t * (46.815 + t * (0.00059 - t * 0.001813))) / 60.0) / 60.0;
    let obliquity = (obliquity_mean + 0.00256 * node.cos()).to_radians();

    let declination = (obliquity.sin() * apparent.sin()).asin().to_degrees();

    let y = (obliquity / 2.0).tan().powi(2);
    let l0 = mean_longitude.to_radians();
    let equation_rad = y * (2.0 * l0).sin() - 2.0 * eccentricity * anomaly.sin()
        + 4.0 * eccentricity * y * anomaly.sin() * (2.0 * l0).cos()
        - 0.5 * y * y * (4.0 * l0).sin()
        - 1.25 * eccentricity * eccentricity * (2.0 * anomaly).sin();

    (declination, equation_rad.to_degrees() * MINUTES_PER_DEG)
}

/// Days from 1970-01-01 to the date (proleptic Gregorian; H. Hinnant's `days_from_civil`).
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let month = i64::from(month);
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    const BERLIN: Site = Site {
        latitude_deg: 52.52,
        longitude_deg: 13.405,
        north_deg: 0.0,
    };

    fn at(
        (year, month, day): (i32, u8, u8),
        (hour, minute): (u8, u8),
        offset_min: Option<i16>,
    ) -> SunTime {
        SunTime {
            year,
            month,
            day,
            hour,
            minute,
            utc_offset_min: offset_min,
        }
    }

    #[test]
    fn the_day_count_starts_at_the_unix_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(days_from_civil(2026, 6, 21), 20_625);
    }

    /// Meeus, Astronomical Algorithms, example 25.a / 28.a: 1992 October 13, 0h TD
    /// (JD 2448908.5): apparent declination -7.78507°, equation of time +13 min 42.7 s.
    #[test]
    fn declination_and_equation_match_the_published_example() {
        let (declination_deg, equation_min) = solar_terms(2_448_908.5);

        assert!(
            (declination_deg - -7.78507).abs() < 0.01,
            "declination {declination_deg}"
        );
        assert!(
            (equation_min - 13.7117).abs() < 0.1,
            "equation of time {equation_min}"
        );
    }

    /// At the June solstice the sun at its noon stands 90° - latitude + 23.44° high, due south.
    #[test]
    fn berlin_at_solar_noon_of_the_solstice_stands_south_and_high() {
        // Solar noon: 12:00 UTC - 13.405° * 4 min - equation of time (-1.7 min) = 11:08 UTC.
        let noon = at((2026, 6, 21), (13, 8), Some(120));
        let sun = sun_at(&BERLIN, &noon);

        assert!((sun.elevation_deg - 60.92).abs() < 0.3, "{sun:?}");
        assert!((sun.azimuth_deg - 180.0).abs() < 1.0, "{sun:?}");
    }

    /// On the equinox at the equator the sun rises in the east and sets in the west, and stands
    /// overhead at noon.
    #[test]
    fn on_the_equator_the_sun_goes_from_east_to_west() {
        let equator = Site {
            latitude_deg: 0.0,
            longitude_deg: 0.0,
            north_deg: 0.0,
        };
        let day = (2026, 3, 20);

        let morning = sun_at(&equator, &at(day, (6, 0), Some(0)));
        let noon = sun_at(&equator, &at(day, (12, 7), Some(0)));
        let evening = sun_at(&equator, &at(day, (18, 0), Some(0)));

        assert!((morning.azimuth_deg - 90.0).abs() < 2.0, "{morning:?}");
        assert!(noon.elevation_deg > 88.0, "{noon:?}");
        assert!((evening.azimuth_deg - 270.0).abs() < 2.0, "{evening:?}");
    }

    #[test]
    fn at_midnight_the_sun_is_below_the_horizon_and_in_the_north() {
        let sun = sun_at(&BERLIN, &at((2026, 11, 17), (0, 0), Some(60)));

        assert!(!sun.is_up(), "{sun:?}");
        assert!(
            sun.elevation_deg < -50.0 && sun.elevation_deg > -80.0,
            "{sun:?}"
        );
    }

    /// A clock that runs two hours ahead shows the same sun two hours later.
    #[test]
    fn the_offset_moves_the_clock_not_the_sun() {
        let summer = sun_at(&BERLIN, &at((2026, 6, 21), (14, 0), Some(120)));
        let winter_clock = sun_at(&BERLIN, &at((2026, 6, 21), (13, 0), Some(60)));

        assert!((summer.azimuth_deg - winter_clock.azimuth_deg).abs() < 1e-9);
        assert!((summer.elevation_deg - winter_clock.elevation_deg).abs() < 1e-9);
    }

    fn same(a: &Sun, b: &Sun) -> bool {
        (a.azimuth_deg - b.azimuth_deg).abs() < 1e-9
            && (a.elevation_deg - b.elevation_deg).abs() < 1e-9
    }

    /// Without an offset the clock is Central European Time: +02:00 in summer, +01:00 in winter,
    /// wherever the place lies.
    #[test]
    fn without_an_offset_the_clock_is_central_european() {
        let madrid = Site {
            latitude_deg: 40.42,
            longitude_deg: -3.70,
            north_deg: 0.0,
        };
        let summer = sun_at(&madrid, &at((2026, 7, 15), (14, 0), None));
        let summer_given = sun_at(&madrid, &at((2026, 7, 15), (14, 0), Some(120)));
        let winter = sun_at(&BERLIN, &at((2026, 1, 15), (14, 0), None));
        let winter_given = sun_at(&BERLIN, &at((2026, 1, 15), (14, 0), Some(60)));

        assert_eq!(summer, summer_given);
        assert_eq!(winter, winter_given);
    }

    /// Summer time starts on the last Sunday of March at 01:00 UTC and ends on the last Sunday
    /// of October at 01:00 UTC (2026: 29 March, 25 October).
    #[test]
    fn summer_time_switches_at_one_utc() {
        let blank = |date, time| sun_at(&BERLIN, &at(date, time, None));
        let given = |date, time, offset| sun_at(&BERLIN, &at(date, time, Some(offset)));

        // 00:59 UTC is still winter, 01:00 UTC is summer; the clock of the instant is +01:00.
        assert!(same(
            &blank((2026, 3, 29), (1, 59)),
            &given((2026, 3, 29), (1, 59), 60)
        ));
        assert!(same(
            &blank((2026, 3, 29), (2, 0)),
            &given((2026, 3, 29), (2, 0), 120)
        ));
        assert!(same(
            &blank((2026, 10, 25), (1, 59)),
            &given((2026, 10, 25), (1, 59), 120)
        ));
        assert!(same(
            &blank((2026, 10, 25), (2, 0)),
            &given((2026, 10, 25), (2, 0), 60)
        ));
        // The hour of the switch: missing in March, read as summer; twice in October, the second.
        assert!(same(
            &blank((2026, 3, 29), (2, 30)),
            &given((2026, 3, 29), (2, 30), 120)
        ));
        assert!(same(
            &blank((2026, 10, 25), (2, 30)),
            &given((2026, 10, 25), (2, 30), 60)
        ));
        // The days around the switches.
        assert!(same(
            &blank((2026, 3, 28), (12, 0)),
            &given((2026, 3, 28), (12, 0), 60)
        ));
        assert!(same(
            &blank((2026, 3, 30), (12, 0)),
            &given((2026, 3, 30), (12, 0), 120)
        ));
        assert!(same(
            &blank((2026, 10, 24), (12, 0)),
            &given((2026, 10, 24), (12, 0), 120)
        ));
        assert!(same(
            &blank((2026, 10, 26), (12, 0)),
            &given((2026, 10, 26), (12, 0), 60)
        ));
    }

    /// The compass turned by the plan's north: sun due south with north at the plan's top is
    /// at the bottom (+y); with north to the right it is to the left (-x).
    #[test]
    fn the_sun_on_the_plan_follows_the_plans_north() {
        let sun = Sun {
            azimuth_deg: 180.0,
            elevation_deg: 30.0,
        };
        let mut site = BERLIN;

        let down = sun.toward_on_plan(&site);
        assert!(
            down.x_m.abs() < 1e-9 && (down.y_m - 1.0).abs() < 1e-9,
            "{down:?}"
        );

        site.north_deg = 90.0;
        let left = sun.toward_on_plan(&site);
        assert!(
            (left.x_m - -1.0).abs() < 1e-9 && left.y_m.abs() < 1e-9,
            "{left:?}"
        );
    }
}
