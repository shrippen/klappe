//! The angle of a measurement (LH-14): counted like a figure's `facing_deg`.
#![allow(clippy::unwrap_used)]

use klappe_services::{measure_angle_deg, Point};

const DECIMALS: usize = 1;

fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

fn angle(b: Point) -> Option<f64> {
    measure_angle_deg(p(0.0, 0.0), b, DECIMALS)
}

/// 0 right, 90 down the plan, 180 left, 270 up.
#[test]
fn the_angle_counts_like_facing() {
    assert_eq!(angle(p(2.0, 0.0)), Some(0.0));
    assert_eq!(angle(p(0.0, 2.0)), Some(90.0));
    assert_eq!(angle(p(-2.0, 0.0)), Some(180.0));
    assert_eq!(angle(p(0.0, -2.0)), Some(270.0));
    assert_eq!(angle(p(1.0, -1.0)), Some(315.0));
    assert_eq!(angle(p(3.0, 3.0)), Some(45.0));
}

/// Review of PR 86: 359.97 showed as "360°"; a tiny negative angle as "-0°".
#[test]
fn the_angle_rounds_before_it_wraps() {
    assert_eq!(angle(p(1.0, -0.000_523_6)), Some(0.0));
    assert!(angle(p(4.0, -0.0001)).unwrap().is_sign_positive());
    assert_eq!(angle(p(1.0, -1.0)), Some(315.0));
}

#[test]
fn coinciding_points_have_no_angle() {
    let a = p(1.0, 1.0);

    assert_eq!(measure_angle_deg(a, a, DECIMALS), None);
}
