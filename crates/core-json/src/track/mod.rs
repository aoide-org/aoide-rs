// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use nonicle::CanonicalizeInto as _;
use url::Url;

use aoide_core::{track::PlayCount, util::clock::OffsetDateTimeMs};

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

/// Advisory rating for content(s):
///
/// * 0 = Inoffensive/Unrated
/// * 1 = Explicit (offensive)
/// * 2 = Clean (inoffensive after edited)
#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema_repr))]
#[repr(u8)]
pub enum AdvisoryRating {
    Unrated = _core::AdvisoryRating::Unrated as u8,
    Explicit = _core::AdvisoryRating::Explicit as u8,
    Clean = _core::AdvisoryRating::Clean as u8,
}

impl From<_core::AdvisoryRating> for AdvisoryRating {
    fn from(from: _core::AdvisoryRating) -> Self {
        use _core::AdvisoryRating as From;
        match from {
            From::Unrated => Self::Unrated,
            From::Explicit => Self::Explicit,
            From::Clean => Self::Clean,
        }
    }
}

impl From<AdvisoryRating> for _core::AdvisoryRating {
    fn from(from: AdvisoryRating) -> Self {
        use AdvisoryRating as From;
        match from {
            From::Unrated => Self::Unrated,
            From::Explicit => Self::Explicit,
            From::Clean => Self::Clean,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Track {
    media_source: Source,

    /// The recording date.
    ///
    /// This field resembles what is commonly known as `year` in many applications, i.e. a generic
    /// year, date or time stamp for chronological ordering. If in doubt or nothing else is
    /// available then use this field.
    ///
    /// Proposed tag mapping:
    ///
    /// * ID3v2.4: "TDRC"
    /// * MP4: "©day"
    /// * Vorbis: "DATE" ("YEAR")
    #[serde(skip_serializing_if = "Option::is_none")]
    recorded_at: Option<DateOrDateTime>,

    /// The release date.
    ///
    /// Stores the distinguished release date if available.
    ///
    /// Proposed tag mapping:
    ///
    /// * ID3v2.4: "TDRL"
    /// * MP4: n/a
    /// * Vorbis: "RELEASEDATE" ("RELEASEYEAR")
    #[serde(skip_serializing_if = "Option::is_none")]
    released_at: Option<DateOrDateTime>,

    /// The original release date.
    ///
    /// Stores the original or first release date if available.
    ///
    /// The original release date is supposed to be not later than the release date.
    ///
    /// Proposed tag mapping:
    ///
    /// * ID3v2.4: "TDOR"
    /// * MP4: n/a
    /// * Vorbis: "ORIGINALDATE" ("ORIGINALYEAR")
    #[serde(skip_serializing_if = "Option::is_none")]
    released_orig_at: Option<DateOrDateTime>,

    /// The publisher or record label.
    #[serde(skip_serializing_if = "Option::is_none")]
    publisher: Option<String>,

    /// Copyright.
    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    advisory_rating: Option<AdvisoryRating>,

    #[serde(skip_serializing_if = "Album::is_default", default)]
    album: Album,

    /// An array of titles.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    titles: Vec<Title>,

    /// An array of actors.
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

    /// An array of cues.
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
            advisory_rating,
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
            advisory_rating: advisory_rating.map(Into::into),
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
            advisory_rating,
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
        let metrics = metrics
            .try_into()
            .map_err(|()| anyhow::anyhow!("invalid metrics"))?;
        let into = Self {
            media_source,
            recorded_at: recorded_at.map(Into::into),
            released_at: released_at.map(Into::into),
            released_orig_at: released_orig_at.map(Into::into),
            publisher,
            copyright,
            advisory_rating: advisory_rating.map(Into::into),
            album: album.into(),
            titles: titles
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .canonicalize_into(),
            actors: actors
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .canonicalize_into(),
            indexes: indexes.into(),
            tags: _core::Tags::from(tags).canonicalize_into(),
            color: color.map(Into::into),
            metrics,
            cues: cues
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .canonicalize_into(),
        };
        Ok(into)
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackBody")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntityBody {
    track: Track,

    updated_at: DateTime,

    /// Last synchronized track entity revision
    ///
    /// The last entity revision of this track that is considered as synchronized with the
    /// underlying media source, or absent if unsynchronized.
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synchronized_rev: Option<EntityRevision>,

    #[serde(skip_serializing_if = "Option::is_none")]
    content_url: Option<Url>,
}

impl TryFrom<EntityBody> for _core::EntityBody {
    type Error = anyhow::Error;

    fn try_from(from: EntityBody) -> anyhow::Result<Self> {
        let EntityBody {
            track,
            updated_at,
            last_synchronized_rev,
            content_url,
        } = from;
        let track = track.try_into()?;
        Ok(Self {
            track,
            updated_at: OffsetDateTimeMs::from(updated_at).to_utc(),
            last_synchronized_rev,
            content_url,
        })
    }
}

impl From<_core::EntityBody> for EntityBody {
    fn from(from: _core::EntityBody) -> Self {
        let _core::EntityBody {
            track,
            updated_at,
            last_synchronized_rev,
            content_url,
        } = from;
        Self {
            track: track.into(),
            updated_at: OffsetDateTimeMs::from_utc(updated_at).into(),
            last_synchronized_rev,
            content_url,
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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
            times_played,
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
            times_played,
        }
    }
}
