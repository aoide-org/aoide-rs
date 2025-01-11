// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use bitflags::bitflags;
use semval::prelude::*;

///////////////////////////////////////////////////////////////////////
// ChannelCount
///////////////////////////////////////////////////////////////////////

pub type NumberOfChannels = u16;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct ChannelCount(NumberOfChannels);

impl ChannelCount {
    pub const ZERO: Self = Self(0);
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(u16::MAX);

    #[must_use]
    pub const fn new(value: NumberOfChannels) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> NumberOfChannels {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
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
            .invalidate_if(*self < Self::MIN, Self::Invalidity::Min(Self::MIN))
            .invalidate_if(*self > Self::MAX, Self::Invalidity::Max(Self::MAX))
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// ChannelMask/-Flags
///////////////////////////////////////////////////////////////////////

pub type ChannelMask = u32;

bitflags! {
    /// Channel flags
    ///
    /// A mask of (at least) 18 bits, one for each channel.
    ///
    /// Standard speaker channels: <https://www.wikipedia.org/wiki/Surround_sound>
    /// CAF channel bitmap: <https://developer.apple.com/library/archive/documentation/MusicAudio/Reference/CAFSpec/CAF_spec/CAF_spec.html#//apple_ref/doc/uid/TP40001862-CH210-BCGBHHHI>
    /// WAV default channel ordering: <https://learn.microsoft.com/en-us/previous-versions/windows/hardware/design/dn653308(v=vs.85)?redirectedfrom=MSDN#default-channel-ordering>
    /// FFmpeg: <https://ffmpeg.org/doxygen/trunk/group__channel__masks.html>
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct ChannelFlags: ChannelMask {
        /// FL
        const FRONT_LEFT = 1u32 << 0;
        /// FR
        const FRONT_RIGHT = 1u32 << 1;
        /// FC
        const FRONT_CENTER = 1u32 << 2;
        /// LF
        const LOW_FREQUENCY = 1u32 << 3;
        /// BL
        const BACK_LEFT = 1u32 << 4;
        /// BR
        const BACK_RIGHT = 1u32 << 5;
        /// FLC
        const FRONT_LEFT_OF_CENTER = 1u32 << 6;
        /// FLR
        const FRONT_RIGHT_OF_CENTER = 1u32 << 7;
        /// BC
        const BACK_CENTER = 1u32 << 8;
        /// SL
        const SIDE_LEFT = 1u32 << 9;
        /// SR
        const SIDE_RIGHT = 1u32 << 10;
        /// TC
        const TOP_CENTER = 1u32 << 11;
        /// TFL
        const TOP_FRONT_LEFT = 1u32 << 12;
        /// TFC
        const TOP_FRONT_CENTER = 1u32 << 13;
        /// TFR
        const TOP_FRONT_RIGHT = 1u32 << 14;
        /// TBL
        const TOP_BACK_LEFT = 1u32 << 15;
        /// TBC
        const TOP_BACK_CENTER = 1u32 << 16;
        /// TBR
        const TOP_BACK_RIGHT = 1u32 << 17;
    }
}

impl Default for ChannelFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl ChannelFlags {
    pub const MONO: Self = Self::FRONT_CENTER;

    pub const STEREO: Self = Self::FRONT_LEFT.union(Self::FRONT_RIGHT);

    #[must_use]
    pub const fn channel_count(self) -> ChannelCount {
        ChannelCount(self.bits().count_ones() as NumberOfChannels)
    }
}

impl From<ChannelFlags> for ChannelCount {
    fn from(from: ChannelFlags) -> Self {
        from.channel_count()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ChannelFlagsInvalidity {
    Empty,
    Invalid,
}

impl Validate for ChannelFlags {
    type Invalidity = ChannelFlagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                self.bits() & !Self::all().bits() != 0,
                Self::Invalidity::Invalid,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Channels {
    Count(ChannelCount),
    Flags(ChannelFlags),
}

impl Channels {
    #[must_use]
    pub const fn count(self) -> ChannelCount {
        match self {
            Self::Count(count) => count,
            Self::Flags(flags) => flags.channel_count(),
        }
    }

    #[must_use]
    pub const fn flags(self) -> Option<ChannelFlags> {
        match self {
            Self::Count(_) => None,
            Self::Flags(flags) => Some(flags),
        }
    }

    #[must_use]
    pub fn try_from_flags_or_count(
        flags: Option<ChannelFlags>,
        count: Option<ChannelCount>,
    ) -> Option<Self> {
        if let Some(flags) = flags {
            if flags.channel_count() > ChannelCount(0) {
                // Valid flags are prioritized over count
                return Some(Self::Flags(flags));
            }
        }
        count.map(Self::Count)
    }
}

impl From<ChannelCount> for Channels {
    fn from(count: ChannelCount) -> Self {
        Self::Count(count)
    }
}

impl From<ChannelFlags> for Channels {
    fn from(flags: ChannelFlags) -> Self {
        Self::Flags(flags)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ChannelsInvalidity {
    Count(ChannelCountInvalidity),
    Flags(ChannelFlagsInvalidity),
}

impl Validate for Channels {
    type Invalidity = ChannelsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            Channels::Count(count) => context.validate_with(count, Self::Invalidity::Count),
            Channels::Flags(flags) => context.validate_with(flags, Self::Invalidity::Flags),
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
