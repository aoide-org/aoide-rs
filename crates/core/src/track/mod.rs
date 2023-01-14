// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod actor;
pub mod album;
pub mod cue;
pub mod index;
pub mod metric;
pub mod tag;
pub mod title;

use ::url::Url;
use num_derive::{FromPrimitive, ToPrimitive};

use self::{actor::*, album::*, cue::*, index::*, metric::*, title::*};

use crate::{
    media::*,
    prelude::*,
    tag::*,
    util::canonical::{Canonical, IsCanonical},
};

/// Advisory rating code for content(s)
///
/// Values match the "rtng" MP4 atom containing the advisory rating
/// as written by iTunes.
///
/// Note: Previously Apple used the value 4 for explicit content that
/// has now been replaced by 1.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum AdvisoryRating {
    /// Inoffensive
    #[default]
    Unrated = 0,

    /// Offensive
    Explicit = 1,

    /// Inoffensive (Edited)
    Clean = 2,
}

impl AdvisoryRating {
    #[must_use]
    pub fn is_offensive(self) -> bool {
        match self {
            Self::Unrated | Self::Clean => false,
            Self::Explicit => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub media_source: Source,

    /// The recording date
    ///
    /// This field resembles what is commonly known as `year`in
    /// many applications, i.e. a generic year, date or time stamp
    /// for chronological ordering. If in doubt or nothing else is
    /// available then use this field.
    ///
    /// Proposed tag mapping:
    ///   - ID3v2.4: "TDRC"
    ///   - Vorbis:  "DATE" ("YEAR")
    ///   - MP4:     "©day"
    pub recorded_at: Option<DateOrDateTime>,

    /// The release date
    ///
    /// Stores the distinguished release date if available.
    ///
    /// Proposed tag mapping:
    ///   - ID3v2.4: "TDRL"
    ///   - Vorbis: "RELEASEDATE" ("RELEASEYEAR")
    ///   - MP4:     n/a
    pub released_at: Option<DateOrDateTime>,

    /// The original release date
    ///
    /// Stores the original or first release date if available.
    ///
    /// The original release date is supposed to be not later than
    /// the release date.
    ///
    /// Proposed tag mapping:
    ///   - ID3v2.4: "TDOR"
    ///   - Vorbis:  "ORIGINALDATE" ("ORIGINALYEAR")
    ///   - MP4:     n/a
    pub released_orig_at: Option<DateOrDateTime>,

    /// The publisher, e.g. a record label
    ///
    /// Proposed tag mapping:
    /// <https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html>: Record Label
    pub publisher: Option<String>,

    pub copyright: Option<String>,

    pub advisory_rating: Option<AdvisoryRating>,

    pub album: Canonical<Album>,

    pub indexes: Indexes,

    pub titles: Canonical<Vec<Title>>,

    pub actors: Canonical<Vec<Actor>>,

    pub tags: Canonical<Tags<'static>>,

    pub color: Option<Color>,

    pub metrics: Metrics,

    pub cues: Canonical<Vec<Cue>>,
}

impl Track {
    #[must_use]
    pub fn new_from_media_source(media_source: Source) -> Self {
        Self {
            media_source,
            recorded_at: None,
            released_at: None,
            released_orig_at: None,
            publisher: None,
            copyright: None,
            advisory_rating: None,
            album: Default::default(),
            indexes: Default::default(),
            titles: Default::default(),
            actors: Default::default(),
            tags: Default::default(),
            color: None,
            metrics: Default::default(),
            cues: Default::default(),
        }
    }

    #[must_use]
    pub fn track_title(&self) -> Option<&str> {
        Titles::main_title(self.titles.as_ref()).map(|title| title.name.as_str())
    }

    pub fn set_track_title(&mut self, track_title: impl Into<String>) -> bool {
        let mut titles = std::mem::take(&mut self.titles).untie();
        let res = Titles::set_main_title(&mut titles, track_title);
        drop(std::mem::replace(&mut self.titles, Canonical::tie(titles)));
        res
    }

    #[must_use]
    pub fn track_artist(&self) -> Option<&str> {
        Actors::main_actor(self.actors.iter(), actor::Role::Artist).map(|actor| actor.name.as_str())
    }

    #[must_use]
    pub fn track_composer(&self) -> Option<&str> {
        Actors::main_actor(self.actors.iter(), actor::Role::Composer)
            .map(|actor| actor.name.as_str())
    }

    #[must_use]
    pub fn album_title(&self) -> Option<&str> {
        Titles::main_title(self.album.titles.as_ref()).map(|title| title.name.as_str())
    }

    pub fn set_album_title(&mut self, album_title: impl Into<String>) -> bool {
        let mut album = std::mem::take(&mut self.album).untie();
        let mut titles = album.titles.untie();
        let res = Titles::set_main_title(&mut titles, album_title);
        album.titles = Canonical::tie(titles);
        drop(std::mem::replace(&mut self.album, Canonical::tie(album)));
        res
    }

    #[must_use]
    pub fn album_artist(&self) -> Option<&str> {
        Actors::main_actor(self.album.actors.iter(), actor::Role::Artist)
            .map(|actor| actor.name.as_str())
    }

    #[allow(clippy::too_many_lines)] // TODO
    pub fn merge_newer_from_synchronized_media_source(&mut self, newer: Track) {
        let Self {
            actors,
            advisory_rating,
            album,
            color,
            copyright,
            cues,
            indexes,
            media_source,
            metrics,
            recorded_at,
            released_at,
            released_orig_at,
            publisher,
            tags,
            titles,
        } = self;
        let Self {
            actors: newer_actors,
            advisory_rating: newer_advisory_rating,
            album: newer_album,
            color: newer_color,
            copyright: newer_copyright,
            cues: newer_cues,
            indexes: newer_indexes,
            media_source: mut newer_media_source,
            metrics: newer_metrics,
            recorded_at: newer_recorded_at,
            released_at: newer_released_at,
            released_orig_at: newer_released_orig_at,
            publisher: newer_publisher,
            tags: newer_tags,
            titles: newer_titles,
        } = newer;
        // Replace media source but preserve the earlier collected at
        newer_media_source.collected_at = newer_media_source
            .collected_at
            .min(media_source.collected_at);
        *media_source = newer_media_source;
        // Do not replace existing data with empty data
        if !newer_actors.is_empty() {
            *actors = newer_actors;
        }
        if newer_advisory_rating.is_some() {
            *advisory_rating = newer_advisory_rating;
        }
        if newer_album != Default::default() {
            *album = newer_album;
        }
        *color = color.or(newer_color);
        if newer_copyright.is_some() {
            *copyright = newer_copyright;
        }
        if !newer_cues.is_empty() {
            *cues = newer_cues;
        }
        if newer_indexes != Default::default() {
            *indexes = newer_indexes;
        }
        *recorded_at = recorded_at.or(newer_recorded_at);
        *released_at = released_at.or(newer_released_at);
        *released_orig_at = released_at.or(newer_released_orig_at);
        if newer_publisher.is_some() {
            *publisher = newer_publisher;
        }
        if !newer_tags.is_empty() {
            *tags = newer_tags;
        }
        if !newer_titles.is_empty() {
            *titles = newer_titles;
        }
        if newer_metrics != Default::default() {
            let Metrics {
                tempo_bpm,
                key_signature,
                time_signature,
                flags,
            } = metrics;
            let Metrics {
                tempo_bpm: newer_tempo_bpm,
                key_signature: newer_key_signature,
                time_signature: newer_time_signature,
                flags: newer_flags,
            } = newer_metrics;
            *flags = newer_flags
                & !(MetricsFlags::TEMPO_BPM_LOCKED
                    | MetricsFlags::KEY_SIGNATURE_LOCKED
                    | MetricsFlags::TIME_SIGNATURE_LOCKED);
            if newer_tempo_bpm.is_some() {
                *tempo_bpm = newer_tempo_bpm;
                flags.set(
                    MetricsFlags::TEMPO_BPM_LOCKED,
                    newer_flags.contains(MetricsFlags::TEMPO_BPM_LOCKED),
                );
            }
            if newer_key_signature != Default::default() {
                *key_signature = newer_key_signature;
                flags.set(
                    MetricsFlags::KEY_SIGNATURE_LOCKED,
                    newer_flags.contains(MetricsFlags::KEY_SIGNATURE_LOCKED),
                );
            }
            if newer_time_signature.is_some() {
                *time_signature = newer_time_signature;
                flags.set(
                    MetricsFlags::TIME_SIGNATURE_LOCKED,
                    newer_flags.contains(MetricsFlags::TIME_SIGNATURE_LOCKED),
                );
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TrackInvalidity {
    MediaSource(SourceInvalidity),
    RecordedAt(DateOrDateTimeInvalidity),
    ReleasedAt(DateOrDateTimeInvalidity),
    ReleasedOrigAt(DateOrDateTimeInvalidity),
    ReleasedOrigAtAfterReleasedAt,
    PublisherEmpty,
    CopyrightEmpty,
    Album(AlbumInvalidity),
    Titles(TitlesInvalidity),
    Actors(ActorsInvalidity),
    Indexes(IndexesInvalidity),
    Tags(TagsInvalidity),
    Color(ColorInvalidity),
    Metrics(MetricsInvalidity),
    Cue(CueInvalidity),
}

impl Validate for Track {
    type Invalidity = TrackInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new()
            .validate_with(&self.media_source, Self::Invalidity::MediaSource)
            .validate_with(&self.recorded_at, Self::Invalidity::RecordedAt)
            .validate_with(&self.released_at, Self::Invalidity::ReleasedAt)
            .validate_with(&self.released_orig_at, Self::Invalidity::ReleasedOrigAt)
            .validate_with(self.album.as_ref(), Self::Invalidity::Album)
            .merge_result_with(
                Titles::validate(&self.titles.iter()),
                Self::Invalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(&self.actors.iter()),
                Self::Invalidity::Actors,
            )
            .validate_with(&self.indexes, Self::Invalidity::Indexes)
            .validate_with(self.tags.as_ref(), Self::Invalidity::Tags)
            .validate_with(&self.color, Self::Invalidity::Color)
            .validate_with(&self.metrics, Self::Invalidity::Metrics)
            .merge_result(
                self.cues
                    .iter()
                    .fold(ValidationContext::new(), |context, next| {
                        context.validate_with(next, Self::Invalidity::Cue)
                    })
                    .into(),
            );
        if let (Some(released_orig_at), Some(released_at)) =
            (self.released_orig_at, self.released_at)
        {
            context = context.invalidate_if(
                released_orig_at > released_at,
                Self::Invalidity::ReleasedOrigAtAfterReleasedAt,
            );
        }
        if let Some(ref publisher) = self.publisher {
            context = context.invalidate_if(
                publisher.trim().is_empty(),
                Self::Invalidity::PublisherEmpty,
            );
        }
        if let Some(ref copyright) = self.copyright {
            context = context.invalidate_if(
                copyright.trim().is_empty(),
                Self::Invalidity::CopyrightEmpty,
            );
        }
        context.into()
    }
}

impl IsCanonical for Track {
    fn is_canonical(&self) -> bool {
        true
    }
}

/// Entity-aware shell
///
/// An entity-aware wrapper around [`Track`] that contains additional
/// entity-related properties.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityBody {
    pub track: Track,

    pub updated_at: DateTime,

    /// Last synchronized track entity revision
    ///
    /// The last entity revision of this track that is considered
    /// as synchronized with the underlying media source, or `None`
    /// if unsynchronized.
    ///
    /// This property is read-only and only managed internally. Any
    /// provided value will be silently ignored when creating or
    /// updating a track entity if not mentioned otherwise.
    pub last_synchronized_rev: Option<EntityRevision>,

    /// URL as resolved from the content path of the media source
    pub content_url: Option<Url>,
}

impl Validate for EntityBody {
    type Invalidity = TrackInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        self.track.validate()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityType;

pub type EntityUid = EntityUidTyped<EntityType>;

pub type EntityHeader = EntityHeaderTyped<EntityType>;

pub type Entity = crate::entity::Entity<EntityType, EntityBody, TrackInvalidity>;

///////////////////////////////////////////////////////////////////////
// PlayCounter
///////////////////////////////////////////////////////////////////////

pub type PlayCount = u64;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayCounter {
    pub last_played_at: Option<DateTime>,
    pub times_played: Option<PlayCount>,
}
