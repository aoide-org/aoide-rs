// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::clock::*;

use std::str::FromStr;

/// Try to parse a `DateTime` value and fallback to the timestamp
/// milliseconds on error (should never happen).
pub(crate) fn parse_datetime(s: &str, timestamp_millis: TimestampMillis) -> DateTime {
    let res = s.parse();
    debug_assert!(res.is_ok());
    res.unwrap_or_else(|_| DateTime::new_timestamp_millis(timestamp_millis))
}

pub(crate) fn parse_datetime_opt(
    s: Option<&str>,
    timestamp_millis: Option<TimestampMillis>,
) -> Option<DateTime> {
    debug_assert_eq!(s.is_some(), timestamp_millis.is_some());
    let res = s.map(FromStr::from_str).transpose();
    debug_assert!(res.is_ok());
    if let Ok(ok) = res {
        ok
    } else {
        timestamp_millis.map(DateTime::new_timestamp_millis)
    }
}
