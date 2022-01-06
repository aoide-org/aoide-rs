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

pub mod actor;
pub mod album;
pub mod cue;
pub mod index;
pub mod metric;
pub mod tag;
pub mod title;

use self::{actor::*, album::*, cue::*, index::*, metric::*, title::*};

use crate::{
    media::*,
    prelude::*,
    tag::*,
    util::canonical::{Canonical, IsCanonical},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub media_source: Source,

    /// The recording date
    ///
    /// The distinction of the recording and the release date is
    /// blurry and sometimes even unknown depending on the format
    /// were the metadata is originating from. Often only the
    /// release date instead of the actual recording date is
    /// available.
    pub recorded_at: Option<DateOrDateTime>,

    /// The release date
    ///
    /// The release date may differ for tracks from the same album!
    /// On compilation albums the release date often denotes the
    /// original release or recording date. Even tracks of the same
    /// regular album may have different release dates from a different
    /// calendar year.
    ///
    /// The release date is supposed to be not earlier than the
    /// recording date.
    pub released_at: Option<DateOrDateTime>,

    /// The publisher, e.g. a record label
    pub released_by: Option<String>,

    pub copyright: Option<String>,

    pub album: Canonical<Album>,

    pub indexes: Indexes,

    pub titles: Canonical<Vec<Title>>,

    pub actors: Canonical<Vec<Actor>>,

    pub tags: Canonical<Tags>,

    pub color: Option<Color>,

    pub metrics: Metrics,

    pub cues: Canonical<Vec<Cue>>,

    pub play_counter: PlayCounter,
}

impl Track {
    pub fn new_from_media_source(media_source: Source) -> Self {
        Self {
            media_source,
            recorded_at: Default::default(),
            released_at: Default::default(),
            released_by: Default::default(),
            copyright: Default::default(),
            album: Default::default(),
            indexes: Default::default(),
            titles: Default::default(),
            actors: Default::default(),
            tags: Default::default(),
            color: Default::default(),
            metrics: Default::default(),
            cues: Default::default(),
            play_counter: Default::default(),
        }
    }

    pub fn track_title(&self) -> Option<&str> {
        Titles::main_title(self.titles.as_ref()).map(|title| title.name.as_str())
    }

    pub fn set_track_title(&mut self, track_title: impl Into<String>) -> bool {
        let mut titles = std::mem::take(&mut self.titles).untie();
        let res = Titles::set_main_title(&mut titles, track_title);
        drop(std::mem::replace(&mut self.titles, Canonical::tie(titles)));
        res
    }

    pub fn track_artist(&self) -> Option<&str> {
        Actors::main_actor(self.actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    }

    pub fn set_track_artist(&mut self, track_artist: impl Into<String>) -> bool {
        let mut actors = std::mem::take(&mut self.actors).untie();
        let res = Actors::set_main_actor(&mut actors, ActorRole::Artist, track_artist);
        drop(std::mem::replace(&mut self.actors, Canonical::tie(actors)));
        res
    }

    pub fn track_composer(&self) -> Option<&str> {
        Actors::main_actor(self.actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    }

    pub fn set_track_composer(&mut self, track_composer: impl Into<String>) -> bool {
        let mut actors = std::mem::take(&mut self.actors).untie();
        let res = Actors::set_main_actor(&mut actors, ActorRole::Composer, track_composer);
        drop(std::mem::replace(&mut self.actors, Canonical::tie(actors)));
        res
    }

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

    pub fn album_artist(&self) -> Option<&str> {
        Actors::main_actor(self.album.actors.iter(), ActorRole::Artist)
            .map(|actor| actor.name.as_str())
    }

    pub fn set_album_artist(&mut self, album_artist: impl Into<String>) -> bool {
        let mut album = std::mem::take(&mut self.album).untie();
        let mut actors = album.actors.untie();
        let res = Actors::set_main_actor(&mut actors, ActorRole::Artist, album_artist);
        album.actors = Canonical::tie(actors);
        drop(std::mem::replace(&mut self.album, Canonical::tie(album)));
        res
    }

    pub fn merge_newer_from_synchronized_media_source(&mut self, newer: Track) {
        let Self {
            actors,
            album,
            color,
            copyright,
            cues,
            indexes,
            media_source,
            metrics,
            play_counter,
            recorded_at,
            released_at,
            released_by,
            tags,
            titles,
        } = self;
        let Self {
            actors: newer_actors,
            album: newer_album,
            color: newer_color,
            copyright: newer_copyright,
            cues: newer_cues,
            indexes: newer_indexes,
            media_source: mut newer_media_source,
            metrics: newer_metrics,
            play_counter: newer_play_counter,
            recorded_at: newer_recorded_at,
            released_at: newer_released_at,
            released_by: newer_released_by,
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
        if newer_play_counter != Default::default() {
            *play_counter = newer_play_counter;
        }
        *recorded_at = recorded_at.or(newer_recorded_at);
        *released_at = released_at.or(newer_released_at);
        if newer_released_by.is_some() {
            *released_by = newer_released_by;
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackInvalidity {
    MediaSource(SourceInvalidity),
    RecordedAt(DateOrDateTimeInvalidity),
    ReleasedAt(DateOrDateTimeInvalidity),
    ReleasedAtBeforeRecordedAt,
    ReleasedByEmpty,
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
            .validate_with(self.album.as_ref(), Self::Invalidity::Album)
            .merge_result_with(
                Titles::validate(self.titles.iter()),
                Self::Invalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(self.actors.iter()),
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
        if let (Some(recorded_at), Some(released_at)) = (self.recorded_at, self.recorded_at) {
            context = context.invalidate_if(
                released_at < recorded_at,
                Self::Invalidity::ReleasedAtBeforeRecordedAt,
            )
        }
        if let Some(ref released_by) = self.released_by {
            context = context.invalidate_if(
                released_by.trim().is_empty(),
                Self::Invalidity::ReleasedByEmpty,
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

pub type Entity = crate::entity::Entity<TrackInvalidity, Track>;

///////////////////////////////////////////////////////////////////////
// PlayCounter
///////////////////////////////////////////////////////////////////////

pub type PlayCount = u64;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayCounter {
    pub last_played_at: Option<DateTime>,
    pub times_played: Option<PlayCount>,
}
