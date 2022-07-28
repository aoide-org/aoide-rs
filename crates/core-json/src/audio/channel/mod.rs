// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::audio::channel::*;
}

///////////////////////////////////////////////////////////////////////
// ChannelCount
///////////////////////////////////////////////////////////////////////

pub use aoide_core::audio::channel::ChannelCount;

///////////////////////////////////////////////////////////////////////
// ChannelLayout
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
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

impl From<_core::ChannelLayout> for ChannelLayout {
    fn from(from: _core::ChannelLayout) -> Self {
        use _core::ChannelLayout::*;
        match from {
            Mono => ChannelLayout::Mono,
            DualMono => ChannelLayout::DualMono,
            Stereo => ChannelLayout::Stereo,
            Three => ChannelLayout::Three,
            Four => ChannelLayout::Four,
            Five => ChannelLayout::Five,
            FiveOne => ChannelLayout::FiveOne,
            SevenOne => ChannelLayout::SevenOne,
        }
    }
}

impl From<ChannelLayout> for _core::ChannelLayout {
    fn from(from: ChannelLayout) -> Self {
        use _core::ChannelLayout::*;
        match from {
            ChannelLayout::Mono => Mono,
            ChannelLayout::DualMono => DualMono,
            ChannelLayout::Stereo => Stereo,
            ChannelLayout::Three => Three,
            ChannelLayout::Four => Four,
            ChannelLayout::Five => Five,
            ChannelLayout::FiveOne => FiveOne,
            ChannelLayout::SevenOne => SevenOne,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
pub enum Channels {
    Count(ChannelCount),
    Layout(ChannelLayout),
}

impl From<_core::Channels> for Channels {
    fn from(from: _core::Channels) -> Self {
        use _core::Channels::*;
        match from {
            Count(count) => Channels::Count(count),
            Layout(layout) => Channels::Layout(layout.into()),
        }
    }
}

impl From<Channels> for _core::Channels {
    fn from(from: Channels) -> Self {
        use _core::Channels::*;
        match from {
            Channels::Count(count) => Count(count),
            Channels::Layout(layout) => Layout(layout.into()),
        }
    }
}
