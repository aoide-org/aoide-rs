// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::{
    track::PlayCount,
    util::canonical::{Canonical, CanonicalizeInto as _},
};

use crate::{entity::EntityRevision, media::Source, prelude::*, tag::*};

pub mod actor;
pub mod album;
pub mod cue;
pub mod index;
pub mod metric;
pub mod title;

use self::{actor::*, album::*, cue::*, index::*, metric::*, title::*};

mod _core {
    pub use aoide_core::{tag::Tags, track::*};
}

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    pub media_source: Source,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synchronized_rev: Option<EntityRevision>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    released_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    released_orig_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    released_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,

    #[serde(skip_serializing_if = "Album::is_default", default)]
    pub album: Album,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Indexes::is_default", default)]
    pub indexes: Indexes,

    #[serde(skip_serializing_if = "Tags::is_empty", default)]
    pub tags: Tags,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    #[serde(skip_serializing_if = "Metrics::is_default", default)]
    pub metrics: Metrics,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cues: Vec<Cue>,

    #[serde(skip_serializing_if = "PlayCounter::is_default", default)]
    pub play_counter: PlayCounter,
}

impl From<_core::Track> for Track {
    fn from(from: _core::Track) -> Self {
        let _core::Track {
            media_source,
            last_synchronized_rev,
            recorded_at,
            released_at,
            released_orig_at,
            released_by,
            copyright,
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
            last_synchronized_rev: last_synchronized_rev.map(Into::into),
            recorded_at: recorded_at.map(Into::into),
            released_at: released_at.map(Into::into),
            released_orig_at: released_orig_at.map(Into::into),
            released_by,
            copyright,
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
            last_synchronized_rev,
            recorded_at,
            released_at,
            released_orig_at,
            released_by,
            copyright,
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
            last_synchronized_rev: last_synchronized_rev.map(Into::into),
            recorded_at: recorded_at.map(Into::into),
            released_at: released_at.map(Into::into),
            released_orig_at: released_orig_at.map(Into::into),
            released_by,
            copyright,
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

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayCounter {
    #[serde(skip_serializing_if = "Option::is_none")]
    last_played_at: Option<DateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    times_played: Option<PlayCount>,
}

impl PlayCounter {
    pub(crate) fn is_default(&self) -> bool {
        let Self {
            last_played_at,
            times_played,
        } = self;
        last_played_at.is_none() && times_played.is_none()
    }
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

#[cfg(test)]
mod tests;
