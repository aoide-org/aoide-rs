// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn min_max_date_year() {
    assert!(YEAR_MIN <= DateYYYYMMDD::min().year());
    assert!(YEAR_MAX <= DateYYYYMMDD::max().year());
}

#[test]
fn into_release_yyyymmdd() {
    assert_eq!(
        DateYYYYMMDD::new(19_961_219),
        DateYYYYMMDD::from("1996-12-19T02:00:57Z".parse::<DateTime>().unwrap()),
    );
    assert_eq!(
        DateYYYYMMDD::new(19_961_219),
        DateYYYYMMDD::from("1996-12-19T02:00:57-12:00".parse::<DateTime>().unwrap()),
    );
    assert_eq!(
        DateYYYYMMDD::new(19_961_219),
        DateYYYYMMDD::from("1996-12-19T02:00:57+12:00".parse::<DateTime>().unwrap()),
    );
}

#[test]
fn from_to_string() {
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57Z"
            .parse::<DateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57+00:00"
            .parse::<DateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57-00:00"
            .parse::<DateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57-12:00",
        "1996-12-19T02:00:57-12:00"
            .parse::<DateTime>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57+12:00",
        "1996-12-19T02:00:57+12:00"
            .parse::<DateTime>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn validate_date() {
    assert!(DateYYYYMMDD::from_year(YEAR_MIN).is_valid());
    assert!(DateYYYYMMDD::from_year_month(YEAR_MIN, 1).is_valid());
    assert!(DateYYYYMMDD::from_year(YEAR_MAX).is_valid());
    assert!(DateYYYYMMDD::from_year_month(YEAR_MAX, 1).is_valid());
    assert!(DateYYYYMMDD::new(19_960_000).is_valid());
    assert!(DateYYYYMMDD::new(19_960_101).is_valid());
    assert!(DateYYYYMMDD::new(19_961_231).is_valid());
    assert!(!DateYYYYMMDD::new(19_960_230).is_valid()); // 1996-02-30
    assert!(!DateYYYYMMDD::new(19_960_001).is_valid()); // 1996-00-01
    assert!(!DateYYYYMMDD::new(1_996_000).is_valid());
    assert!(!DateYYYYMMDD::new(119_960_001).is_valid());
}
