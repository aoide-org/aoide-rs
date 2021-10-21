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

use crate::{media::Source, prelude::*, tag::*};

pub mod actor;
pub mod album;
pub mod cue;
pub mod index;
pub mod metric;
pub mod release;
pub mod title;

use self::{actor::*, album::*, cue::*, index::*, metric::*, release::*, title::*};

mod _core {
    pub use aoide_core::{tag::Tags, track::*};
}

use aoide_core::{
    track::PlayCount,
    util::{Canonical, CanonicalizeInto as _, IsDefault},
};

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    pub media_source: Source,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub release: Release,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub album: Album,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub indexes: Indexes,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub tags: Tags,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub metrics: Metrics,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub cues: Vec<Cue>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub play_counter: PlayCounter,
}

impl From<_core::Track> for Track {
    fn from(from: _core::Track) -> Self {
        let _core::Track {
            media_source,
            release,
            album,
            titles,
            actors,
            indexes,
            tags,
            color,
            metrics,
            cues,
            play_counter,
        } = from;
        Self {
            media_source: media_source.into(),
            release: release.into(),
            album: album.untie().into(),
            titles: titles.untie().into_iter().map(Into::into).collect(),
            actors: actors.untie().into_iter().map(Into::into).collect(),
            indexes: indexes.into(),
            tags: tags.untie().into(),
            color: color.map(Into::into),
            metrics: metrics.into(),
            cues: cues.untie().into_iter().map(Into::into).collect(),
            play_counter: play_counter.into(),
        }
    }
}

impl TryFrom<Track> for _core::Track {
    type Error = anyhow::Error;

    fn try_from(from: Track) -> anyhow::Result<Self> {
        let Track {
            media_source,
            release,
            album,
            titles,
            actors,
            indexes,
            tags,
            color,
            metrics,
            cues,
            play_counter,
        } = from;
        let media_source = media_source.try_into()?;
        let into = Self {
            media_source,
            release: release.into(),
            album: album.into(),
            titles: Canonical::tie(
                titles
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .canonicalize_into(),
            ),
            actors: Canonical::tie(
                actors
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .canonicalize_into(),
            ),
            indexes: indexes.into(),
            tags: Canonical::tie(_core::Tags::from(tags).canonicalize_into()),
            color: color.map(Into::into),
            metrics: metrics.into(),
            cues: Canonical::tie(
                cues.into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .canonicalize_into(),
            ),
            play_counter: play_counter.into(),
        };
        Ok(into)
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

pub type Entity = crate::entity::Entity<Track>;

impl TryFrom<Entity> for _core::Entity {
    type Error = anyhow::Error;

    fn try_from(from: Entity) -> anyhow::Result<Self> {
        Self::try_new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}

///////////////////////////////////////////////////////////////////////
// PlayCounter
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayCounter {
    #[serde(skip_serializing_if = "Option::is_none")]
    last_played_at: Option<DateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    times_played: Option<PlayCount>,
}

impl From<_core::PlayCounter> for PlayCounter {
    fn from(from: _core::PlayCounter) -> Self {
        let _core::PlayCounter {
            last_played_at,
            times_played,
        } = from;
        Self {
            last_played_at: last_played_at.map(Into::into),
            times_played: times_played.map(Into::into),
        }
    }
}

impl From<PlayCounter> for _core::PlayCounter {
    fn from(from: PlayCounter) -> Self {
        let PlayCounter {
            last_played_at,
            times_played,
        } = from;
        Self {
            last_played_at: last_played_at.map(Into::into),
            times_played: times_played.map(Into::into),
        }
    }
}
