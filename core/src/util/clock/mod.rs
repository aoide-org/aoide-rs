// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use chrono::{DateTime, TimeZone, Utc};
use std::fmt;

pub type TickType = i64;

const MILLIS_PER_SEC: TickType = 1_000;
const MICROS_PER_SEC: TickType = 1_000_000;
const NANOS_PER_SEC: TickType = 1_000_000_000;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticks(pub TickType);

// Resolution = microseconds
impl Ticks {
    pub const fn per_second() -> Self {
        Self(MICROS_PER_SEC)
    }

    pub const fn per_millisec() -> Self {
        Self(MICROS_PER_SEC / MILLIS_PER_SEC)
    }

    #[allow(clippy::eq_op)]
    pub const fn per_microsec() -> Self {
        Self(MICROS_PER_SEC / MICROS_PER_SEC)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TickDuration(pub Ticks);

impl TickDuration {
    pub fn from_secs(secs: TickType) -> Self {
        Self(Ticks(secs * Ticks::per_second().0))
    }

    pub fn from_millis(millis: Ticks) -> Self {
        Self(Ticks(millis.0 * Ticks::per_millisec().0))
    }
}

impl From<chrono::Duration> for TickDuration {
    fn from(from: chrono::Duration) -> Self {
        // TODO: Check for overflow and use TryFrom?
        debug_assert!(Ticks::per_microsec().0 == 1);
        Self(Ticks(from.num_microseconds().unwrap()))
    }
}

impl From<TickDuration> for chrono::Duration {
    fn from(from: TickDuration) -> Self {
        debug_assert!(Ticks::per_microsec().0 == 1);
        Self::microseconds((from.0).0)
    }
}

impl From<std::time::Duration> for TickDuration {
    fn from(from: std::time::Duration) -> Self {
        // TODO: Check for overflow and use TryFrom?
        debug_assert!(Ticks::per_microsec().0 == 1);
        Self(Ticks(
            from.as_secs() as TickType * MICROS_PER_SEC + i64::from(from.subsec_micros()),
        ))
    }
}

impl From<TickDuration> for std::time::Duration {
    fn from(from: TickDuration) -> Self {
        debug_assert!(Ticks::per_microsec().0 == 1);
        assert!(from >= Default::default());
        Self::from_micros((from.0).0 as u64)
    }
}

impl fmt::Display for TickDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", chrono::Duration::from(*self))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TickInstant(pub Ticks);

impl TickInstant {
    pub fn now() -> Self {
        Self::from(Utc::now())
    }

    pub fn elapsed(self) -> TickDuration {
        let now = Self::now();
        debug_assert!(now >= self);
        TickDuration(Ticks((now.0).0 - (self.0).0))
    }

    /// Time is "tick"ing.
    pub fn tick(self) -> Self {
        Self(Ticks((self.0).0 + 1))
    }
}

impl From<DateTime<Utc>> for TickInstant {
    fn from(from: DateTime<Utc>) -> Self {
        debug_assert!(Ticks::per_second().0 >= MICROS_PER_SEC);
        debug_assert!(Ticks::per_second().0 % MICROS_PER_SEC == 0);
        let secs = from.timestamp();
        let subsec_micros = from.timestamp_subsec_micros();
        let micros = (secs * MICROS_PER_SEC) + TickType::from(subsec_micros);
        let ticks = micros * (Ticks::per_second().0 / MICROS_PER_SEC);
        Self(Ticks(ticks))
    }
}

impl From<TickInstant> for DateTime<Utc> {
    fn from(from: TickInstant) -> Self {
        let secs = (from.0).0 / Ticks::per_second().0;
        let subsec_ticks = (from.0).0 % Ticks::per_second().0;
        debug_assert!(NANOS_PER_SEC >= Ticks::per_second().0);
        debug_assert!(NANOS_PER_SEC % Ticks::per_second().0 == 0);
        let nsecs = subsec_ticks * (NANOS_PER_SEC / Ticks::per_second().0);
        debug_assert!(nsecs >= 0);
        debug_assert!(nsecs < NANOS_PER_SEC);
        Utc.timestamp(secs, nsecs as u32)
    }
}

impl fmt::Display for TickInstant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", DateTime::<Utc>::from(*self))
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

// TODO
