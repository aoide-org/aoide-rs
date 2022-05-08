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
    pub(super) use aoide_core::{tag::Tags, track::*};
}

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    media_source: Source,

    #[serde(skip_serializing_if = "Option::is_none")]
    recorded_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    released_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    released_orig_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    publisher: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,

    #[serde(skip_serializing_if = "Album::is_default", default)]
    album: Album,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Indexes::is_default", default)]
    indexes: Indexes,

    #[serde(skip_serializing_if = "Tags::is_empty", default)]
    tags: Tags,

    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,

    #[serde(skip_serializing_if = "Metrics::is_default", default)]
    metrics: Metrics,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    cues: Vec<Cue>,
}

impl From<_core::Track> for Track {
    fn from(from: _core::Track) -> Self {
        let _core::Track {
            media_source,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            copyright,
            album,
            titles,
            actors,
            indexes,
            tags,
            color,
            metrics,
            cues,
        } = from;
        Self {
            media_source: media_source.into(),
            recorded_at: recorded_at.map(Into::into),
            released_at: released_at.map(Into::into),
            released_orig_at: released_orig_at.map(Into::into),
            publisher,
            copyright,
            album: album.untie().into(),
            titles: titles.untie().into_iter().map(Into::into).collect(),
            actors: actors.untie().into_iter().map(Into::into).collect(),
            indexes: indexes.into(),
            tags: tags.untie().into(),
            color: color.map(Into::into),
            metrics: metrics.into(),
            cues: cues.untie().into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Track> for _core::Track {
    type Error = anyhow::Error;

    fn try_from(from: Track) -> anyhow::Result<Self> {
        let Track {
            media_source,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            copyright,
            album,
            titles,
            actors,
            indexes,
            tags,
            color,
            metrics,
            cues,
        } = from;
        let media_source = media_source.try_into()?;
        let into = Self {
            media_source,
            recorded_at: recorded_at.map(Into::into),
            released_at: released_at.map(Into::into),
            released_orig_at: released_orig_at.map(Into::into),
            publisher,
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
        };
        Ok(into)
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntityBody {
    track: Track,

    updated_at: DateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_synchronized_rev: Option<EntityRevision>,
}

impl TryFrom<EntityBody> for _core::EntityBody {
    type Error = anyhow::Error;

    fn try_from(from: EntityBody) -> anyhow::Result<Self> {
        let EntityBody {
            track,
            updated_at,
            last_synchronized_rev,
        } = from;
        let track = track.try_into()?;
        Ok(Self {
            track,
            updated_at: updated_at.into(),
            last_synchronized_rev: last_synchronized_rev.map(Into::into),
        })
    }
}

impl From<_core::EntityBody> for EntityBody {
    fn from(from: _core::EntityBody) -> Self {
        let _core::EntityBody {
            track,
            updated_at,
            last_synchronized_rev,
        } = from;
        Self {
            track: track.into(),
            updated_at: updated_at.into(),
            last_synchronized_rev: last_synchronized_rev.map(Into::into),
        }
    }
}

pub type Entity = crate::entity::Entity<EntityBody>;

impl TryFrom<Entity> for _core::Entity {
    type Error = anyhow::Error;

    fn try_from(from: Entity) -> anyhow::Result<Self> {
        Self::try_new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        let (hdr, body) = from.into();
        Self(hdr.into(), body.into())
    }
}

///////////////////////////////////////////////////////////////////////
// PlayCounter
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
