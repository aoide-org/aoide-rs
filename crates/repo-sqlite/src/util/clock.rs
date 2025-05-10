// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::str::FromStr;

use aoide_core::util::clock::*;

/// Try to parse a `DateTime` value and fallback to the timestamp
/// milliseconds on error (should never happen).
pub(crate) fn parse_datetime(s: &str, timestamp_millis: TimestampMillis) -> OffsetDateTimeMs {
    let res = s.parse();
    debug_assert!(res.is_ok());
    res.unwrap_or_else(|_| OffsetDateTimeMs::from_unix_timestamp_millis(timestamp_millis))
}

pub(crate) fn parse_datetime_opt(
    s: Option<&str>,
    timestamp_millis: Option<TimestampMillis>,
) -> Option<OffsetDateTimeMs> {
    debug_assert_eq!(s.is_some(), timestamp_millis.is_some());
    let res = s.map(FromStr::from_str).transpose();
    debug_assert!(res.is_ok());
    if let Ok(ok) = res {
        ok
    } else {
        timestamp_millis.map(OffsetDateTimeMs::from_unix_timestamp_millis)
    }
}
