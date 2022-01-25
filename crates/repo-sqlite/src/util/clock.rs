// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aoide_core::util::clock::*;

use std::str::FromStr;

/// Try to parse a DateTime value and fallback to the timestamp
/// milliseconds on error (should never happen).
pub fn parse_datetime(s: &str, timestamp_millis: TimestampMillis) -> DateTime {
    let res = s.parse();
    debug_assert!(res.is_ok());
    res.unwrap_or_else(|_| DateTime::new_timestamp_millis(timestamp_millis))
}

pub fn parse_datetime_opt(
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
