// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use std::u16;
use std::fmt;

///////////////////////////////////////////////////////////////////////
/// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationValue = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Duration {
    pub ms: DurationValue,
}

impl Duration {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    pub const EMPTY: Duration = Duration { ms: 0 as DurationValue };

    pub fn ms(ms: DurationValue) -> Self {
        Self { ms }
    }

    pub fn is_valid(&self) -> bool {
        *self >= Self::EMPTY
    }

    pub fn is_empty(&self) -> bool {
        *self <= Self::EMPTY
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.ms, Duration::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// Channels
///////////////////////////////////////////////////////////////////////

pub type ChannelCount = u16;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
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
            ChannelLayout::Mono => 1,
            ChannelLayout::DualMono => 2,
            ChannelLayout::Stereo => 2,
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
    pub const COUNT_MIN: ChannelCount = 1;

    pub const COUNT_MAX: ChannelCount = u16::MAX;

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
        match count {
            1 => Some(ChannelLayout::Mono),
            2 => Some(ChannelLayout::Stereo),
            _ => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.count < Self::COUNT_MIN {
            false
        } else {
            match self.layout {
                None => true,
                Some(layout) => layout.channel_count() == self.count,
            }
        }
    }
}
