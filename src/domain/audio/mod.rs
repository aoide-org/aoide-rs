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

use std::u16;
use std::fmt;

///////////////////////////////////////////////////////////////////////
/// Channels
///////////////////////////////////////////////////////////////////////

pub type ChannelCount = u16;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChannelLayout {
    Mono,
    DualMono,
    Stereo,
    // ...to be continued
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Channels {
    pub count: ChannelCount,
    #[serde(skip_serializing_if = "Option::is_none")] pub layout: Option<ChannelLayout>,
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
        Channels { count: self.channel_count(), layout: Some(*self) }
    }
}

impl Channels {
    pub const COUNT_MIN: ChannelCount = 1;
    pub const COUNT_MAX: ChannelCount = u16::MAX;

    pub fn count(count: ChannelCount) -> Self {
        Self { count, layout: None }
    }

    pub fn layout(layout: ChannelLayout) -> Self {
        Self { count: layout.channel_count(), layout: Some(layout) }
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

///////////////////////////////////////////////////////////////////////
/// Duration
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    pub millis: u64,
}

impl Duration {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    pub fn millis(millis: u64) -> Self {
        Self { millis }
    }

}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.millis, Duration::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_layout_channel_count() {
        assert_eq!(ChannelLayout::Mono.channel_count(), 1);
        assert_eq!(ChannelLayout::DualMono.channel_count(), 2);
        assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
    }

    #[test]
    fn channels_default() {
        assert_eq!(0, Channels::default().count);
    }

    #[test]
    fn channels_is_valid() {
        assert!(!Channels::default().is_valid());
        assert!(Channels::layout(ChannelLayout::Mono).is_valid());
        assert!(Channels::layout(ChannelLayout::DualMono).is_valid());
        assert!(Channels::layout(ChannelLayout::Stereo).is_valid());
        assert!(Channels::count(Channels::COUNT_MIN).is_valid());
        assert!(Channels::count(Channels::COUNT_MAX).is_valid());
        assert!(!Channels { count: 1, layout: Some(ChannelLayout::DualMono) }.is_valid());
        assert!(!Channels { count: 2, layout: Some(ChannelLayout::Mono) }.is_valid());
        assert!(!Channels { count: 3, layout: Some(ChannelLayout::Stereo) }.is_valid());
    }

    #[test]
    fn channel_count_default_layout() {
        assert_eq!(None, Channels::default_layout(0));
        assert_eq!(Some(ChannelLayout::Mono), Channels::default_layout(1));
        assert_eq!(Some(ChannelLayout::Stereo), Channels::default_layout(2));
        assert_eq!(None, Channels::default_layout(3));
    }
}
