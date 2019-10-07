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

pub type NumberOfChannels = u16;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChannelCount(pub NumberOfChannels);

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChannelCountInvalidity {
    Min(ChannelCount),
    Max(ChannelCount),
}

impl Validate for ChannelCount {
    type Invalidity = ChannelCountInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                *self < Self::min(),
                ChannelCountInvalidity::Min(Self::min()),
            )
            .invalidate_if(
                *self > Self::max(),
                ChannelCountInvalidity::Max(Self::max()),
            )
            .into()
    }
}

impl From<NumberOfChannels> for ChannelCount {
    fn from(from: NumberOfChannels) -> Self {
        Self(from)
    }
}

impl From<ChannelCount> for NumberOfChannels {
    fn from(from: ChannelCount) -> Self {
        from.0
    }
}

///////////////////////////////////////////////////////////////////////
// ChannelLayout
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChannelLayoutInvalidity {
    ChannelCount(ChannelCountInvalidity),
}

impl Validate for ChannelLayout {
    type Invalidity = ChannelLayoutInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(
                &(*self).channel_count(),
                ChannelLayoutInvalidity::ChannelCount,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChannelsInvalidity {
    Count(ChannelCountInvalidity),
    Layout(ChannelLayoutInvalidity),
}

impl Validate for Channels {
    type Invalidity = ChannelsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            Channels::Count(ref count) => {
                context.validate_with(count, ChannelsInvalidity::Count)
            }
            Channels::Layout(ref layout) => {
                context.validate_with(layout, ChannelsInvalidity::Layout)
            }
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
