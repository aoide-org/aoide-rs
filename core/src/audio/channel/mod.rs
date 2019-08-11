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

use super::*;

use std::u16;

///////////////////////////////////////////////////////////////////////
// ChannelCount
///////////////////////////////////////////////////////////////////////

type ChannelCountValue = u16;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct ChannelCount(pub ChannelCountValue);

impl ChannelCount {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn min() -> Self {
        Self(1)
    }

    pub const fn max() -> Self {
        Self(u16::MAX)
    }

    pub fn default_layout(self) -> Option<ChannelLayout> {
        match self {
            ChannelCount(1) => Some(ChannelLayout::Mono),
            ChannelCount(2) => Some(ChannelLayout::Stereo),
            _ => None,
        }
    }
}

impl Validate<()> for ChannelCount {
    fn validate(&self) -> ValidationResult<()> {
        let mut errors = ValidationErrors::default();
        if *self < Self::min() || *self > Self::max() {
            errors.add_error((), Violation::OutOfBounds);
        }
        errors.into_result()
    }
}

impl From<ChannelCountValue> for ChannelCount {
    fn from(from: ChannelCountValue) -> Self {
        Self(from)
    }
}

impl From<ChannelCount> for ChannelCountValue {
    fn from(from: ChannelCount) -> Self {
        from.0
    }
}

///////////////////////////////////////////////////////////////////////
// ChannelLayout
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelLayout {
    Mono,

    DualMono,

    Stereo,
    // ...to be continued
}

impl ChannelLayout {
    pub fn channel_count(self) -> ChannelCount {
        match self {
            ChannelLayout::Mono => ChannelCount(1),
            ChannelLayout::DualMono => ChannelCount(2),
            ChannelLayout::Stereo => ChannelCount(2),
        }
    }
}

impl Validate<()> for ChannelLayout {
    fn validate(&self) -> ValidationResult<()> {
        (*self).channel_count().validate()
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channels {
    Count(ChannelCount),
    Layout(ChannelLayout),
}

impl Channels {
    pub fn count(self) -> ChannelCount {
        use Channels::*;
        match self {
            Count(count) => count,
            Layout(layout) => layout.channel_count(),
        }
    }
}

impl Default for Channels {
    fn default() -> Self {
        Channels::Count(ChannelCount::zero())
    }
}

impl From<ChannelCount> for Channels {
    fn from(count: ChannelCount) -> Self {
        Channels::Count(count)
    }
}

impl From<ChannelLayout> for Channels {
    fn from(layout: ChannelLayout) -> Self {
        Channels::Layout(layout)
    }
}

impl Validate<()> for Channels {
    fn validate(&self) -> ValidationResult<()> {
        match self {
            Channels::Count(count) => count.validate(),
            Channels::Layout(layout) => layout.validate(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
