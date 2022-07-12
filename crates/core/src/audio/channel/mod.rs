// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

use std::u16;

///////////////////////////////////////////////////////////////////////
// ChannelCount
///////////////////////////////////////////////////////////////////////

pub type NumberOfChannels = u16;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelCount(pub NumberOfChannels);

impl ChannelCount {
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(1)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(u16::MAX)
    }

    #[must_use]
    pub fn default_layout(self) -> Option<ChannelLayout> {
        match self {
            ChannelCount(1) => Some(ChannelLayout::Mono),
            ChannelCount(2) => Some(ChannelLayout::Stereo),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ChannelCountInvalidity {
    Min(ChannelCount),
    Max(ChannelCount),
}

impl Validate for ChannelCount {
    type Invalidity = ChannelCountInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::min(), Self::Invalidity::Min(Self::min()))
            .invalidate_if(*self > Self::max(), Self::Invalidity::Max(Self::max()))
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelLayout {
    Mono,
    DualMono,
    Stereo,
    Three,
    Four,
    Five,
    FiveOne,
    SevenOne,
    // ...to be continued
}

impl ChannelLayout {
    #[must_use]
    pub const fn channel_count(self) -> ChannelCount {
        use ChannelLayout::*;
        match self {
            Mono => ChannelCount(1),
            DualMono | Stereo => ChannelCount(2),
            Three => ChannelCount(3),
            Four => ChannelCount(4),
            Five => ChannelCount(5),
            FiveOne => ChannelCount(6),
            SevenOne => ChannelCount(8),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ChannelLayoutInvalidity {
    ChannelCount(ChannelCountInvalidity),
}

impl Validate for ChannelLayout {
    type Invalidity = ChannelLayoutInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&(*self).channel_count(), Self::Invalidity::ChannelCount)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Channels {
    Count(ChannelCount),
    Layout(ChannelLayout),
}

impl Channels {
    #[must_use]
    pub const fn count(self) -> ChannelCount {
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

#[derive(Copy, Clone, Debug)]
pub enum ChannelsInvalidity {
    Count(ChannelCountInvalidity),
    Layout(ChannelLayoutInvalidity),
}

impl Validate for Channels {
    type Invalidity = ChannelsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            Channels::Count(ref count) => context.validate_with(count, Self::Invalidity::Count),
            Channels::Layout(ref layout) => context.validate_with(layout, Self::Invalidity::Layout),
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
