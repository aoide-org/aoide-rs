// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use super::*;

#[test]
fn min_max_date_year() {
    assert!(YEAR_MIN <= YyyyMmDdDate::MIN.year());
    assert!(YEAR_MAX <= YyyyMmDdDate::MAX.year());
}

#[test]
fn into_release_yyyymmdd() {
    assert_eq!(
        YyyyMmDdDate::new_unchecked(19_961_219),
        YyyyMmDdDate::from("1996-12-19T02:00:57Z".parse::<OffsetDateTimeMs>().unwrap()),
    );
    assert_eq!(
        YyyyMmDdDate::new_unchecked(19_961_219),
        YyyyMmDdDate::from(
            "1996-12-19T02:00:57-12:00"
                .parse::<OffsetDateTimeMs>()
                .unwrap()
        ),
    );
    assert_eq!(
        YyyyMmDdDate::new_unchecked(19_961_219),
        YyyyMmDdDate::from(
            "1996-12-19T02:00:57+12:00"
                .parse::<OffsetDateTimeMs>()
                .unwrap()
        ),
    );
}

#[test]
fn from_to_string() {
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57Z"
            .parse::<OffsetDateTimeMs>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57+00:00"
            .parse::<OffsetDateTimeMs>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57Z",
        "1996-12-19T02:00:57-00:00"
            .parse::<OffsetDateTimeMs>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57-12:00",
        "1996-12-19T02:00:57-12:00"
            .parse::<OffsetDateTimeMs>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "1996-12-19T02:00:57+12:00",
        "1996-12-19T02:00:57+12:00"
            .parse::<OffsetDateTimeMs>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn validate_date() {
    assert!(YyyyMmDdDate::from_year(YEAR_MIN).is_valid());
    assert!(YyyyMmDdDate::from_year_month(YEAR_MIN, 1).is_valid());
    assert!(YyyyMmDdDate::from_year(YEAR_MAX).is_valid());
    assert!(YyyyMmDdDate::from_year_month(YEAR_MAX, 1).is_valid());
    assert!(YyyyMmDdDate::new_unchecked(19_960_000).is_valid());
    assert!(YyyyMmDdDate::new_unchecked(19_960_101).is_valid());
    assert!(YyyyMmDdDate::new_unchecked(19_961_231).is_valid());
    assert!(!YyyyMmDdDate::new_unchecked(19_960_230).is_valid()); // 1996-02-30
    assert!(!YyyyMmDdDate::new_unchecked(19_960_001).is_valid()); // 1996-00-01
    assert!(!YyyyMmDdDate::new_unchecked(1_996_000).is_valid());
    assert!(!YyyyMmDdDate::new_unchecked(119_960_001).is_valid());
}

#[cfg(feature = "serde")]
#[test]
fn deserialize_date_time() {
    use time::macros::datetime;

    assert_eq!(
        OffsetDateTimeMs::new_unchecked(datetime!(2020-12-18 21:27:15.123 UTC)),
        serde_json::from_value::<OffsetDateTimeMs>(serde_json::json!(
            "2020-12-18T21:27:15.123456Z"
        ))
        .unwrap()
    );
}
