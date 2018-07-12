// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub mod sample;
pub mod signal;

#[cfg(test)]
mod tests;

use std::fmt;
use std::ops::Deref;
use std::time::Duration;
use std::u16;

///////////////////////////////////////////////////////////////////////
/// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationInMilliseconds = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DurationMs(DurationInMilliseconds);

impl DurationMs {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    pub const EMPTY: DurationMs = DurationMs(0 as DurationInMilliseconds);

    pub fn from_ms(ms: DurationInMilliseconds) -> Self {
        DurationMs(ms)
    }

    pub fn ms(&self) -> DurationInMilliseconds {
        self.0
    }

    pub fn is_valid(&self) -> bool {
        *self >= Self::EMPTY
    }

    pub fn is_empty(&self) -> bool {
        *self <= Self::EMPTY
    }
}

impl From<Duration> for DurationMs {
    fn from(duration: Duration) -> Self {
        let secs = duration.as_secs() as DurationInMilliseconds;
        let subsec_nanos = duration.subsec_nanos() as DurationInMilliseconds;
        Self::from_ms(
            secs * 1_000 as DurationInMilliseconds
                + subsec_nanos / 1_000_000 as DurationInMilliseconds,
        )
    }
}

impl fmt::Display for DurationMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.ms(), DurationMs::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// Channels
///////////////////////////////////////////////////////////////////////

pub type ChannelCountValue = u16;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChannelCount(ChannelCountValue);

impl ChannelCount {
    pub const MIN: Self = ChannelCount(1);
    pub const MAX: Self = ChannelCount(u16::MAX);

    pub fn new(count: ChannelCountValue) -> Self {
        ChannelCount(count)
    }

    pub fn is_valid(&self) -> bool {
        debug_assert!(*self <= Self::MAX);
        *self >= Self::MIN
    }
}

impl Deref for ChannelCount {
    type Target = ChannelCountValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ChannelLayout {
    Mono,

    DualMono,

    Stereo,
    // ...to be continued
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Channels {
    pub count: ChannelCount,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ChannelLayout>,
}

impl ChannelLayout {
    pub fn channel_count(&self) -> ChannelCount {
        match *self {
            ChannelLayout::Mono => ChannelCount::new(1),
            ChannelLayout::DualMono => ChannelCount::new(2),
            ChannelLayout::Stereo => ChannelCount::new(2),
        }
    }

    pub fn channels(&self) -> Channels {
        Channels {
            count: self.channel_count(),
            layout: Some(*self),
        }
    }
}

impl Channels {
    pub fn count(count: ChannelCount) -> Self {
        Self {
            count,
            layout: None,
        }
    }

    pub fn layout(layout: ChannelLayout) -> Self {
        Self {
            count: layout.channel_count(),
            layout: Some(layout),
        }
    }

    pub fn default_layout(count: ChannelCount) -> Option<ChannelLayout> {
        match *count {
            1 => Some(ChannelLayout::Mono),
            2 => Some(ChannelLayout::Stereo),
            _ => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.count.is_valid() && match self.layout {
            None => true,
            Some(layout) => layout.channel_count() == self.count,
        }
    }
}
