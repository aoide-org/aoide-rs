// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod _core {
    pub use aoide_core::audio::channel::*;
}

///////////////////////////////////////////////////////////////////////
// ChannelCount
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelCount(_core::NumberOfChannels);

impl From<_core::ChannelCount> for ChannelCount {
    fn from(from: _core::ChannelCount) -> Self {
        Self(from.0)
    }
}

impl From<ChannelCount> for _core::ChannelCount {
    fn from(from: ChannelCount) -> Self {
        Self(from.0)
    }
}

impl Default for ChannelCount {
    fn default() -> ChannelCount {
        _core::ChannelCount::default().into()
    }
}

///////////////////////////////////////////////////////////////////////
// ChannelLayout
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ChannelLayout {
    Mono,
    DualMono,
    Stereo,
    // ...to be continued
}

impl From<_core::ChannelLayout> for ChannelLayout {
    fn from(from: _core::ChannelLayout) -> Self {
        use _core::ChannelLayout::*;
        match from {
            Mono => ChannelLayout::Mono,
            DualMono => ChannelLayout::DualMono,
            Stereo => ChannelLayout::Stereo,
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
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Channels
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Channels {
    Count(ChannelCount),
    Layout(ChannelLayout),
}

impl From<_core::Channels> for Channels {
    fn from(from: _core::Channels) -> Self {
        use _core::Channels::*;
        match from {
            Count(count) => Channels::Count(count.into()),
            Layout(layout) => Channels::Layout(layout.into()),
        }
    }
}

impl From<Channels> for _core::Channels {
    fn from(from: Channels) -> Self {
        use _core::Channels::*;
        match from {
            Channels::Count(count) => Count(count.into()),
            Channels::Layout(layout) => Layout(layout.into()),
        }
    }
}
