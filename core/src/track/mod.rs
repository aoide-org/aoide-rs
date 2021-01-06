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

///////////////////////////////////////////////////////////////////////

pub mod actor;
pub mod album;
pub mod cue;
pub mod index;
pub mod metric;
pub mod release;
pub mod tag;
pub mod title;

use self::{actor::*, album::*, cue::*, index::*, metric::*, release::*, title::*};

use crate::{media::*, prelude::*, tag::*};

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub media_source: Source,

    pub release: Release,

    pub album: Album,

    pub indexes: Indexes,

    pub titles: Vec<Title>,

    pub actors: Vec<Actor>,

    pub tags: Tags,

    pub color: Option<Color>,

    pub metrics: Metrics,

    pub cues: Vec<Cue>,

    pub play_counter: PlayCounter,
}

impl Track {
    pub fn track_title(&self) -> Option<&str> {
        Titles::main_title(&self.titles).map(|title| title.name.as_str())
    }

    pub fn set_track_title(&mut self, track_title: impl Into<String>) -> bool {
        Titles::set_main_title(&mut self.titles, track_title)
    }

    pub fn track_artist(&self) -> Option<&str> {
        Actors::main_actor(self.actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    }

    pub fn set_track_artist(&mut self, track_artist: impl Into<String>) -> bool {
        Actors::set_main_actor(&mut self.actors, ActorRole::Artist, track_artist)
    }

    pub fn track_composer(&self) -> Option<&str> {
        Actors::main_actor(self.actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    }

    pub fn set_track_composer(&mut self, track_composer: impl Into<String>) -> bool {
        Actors::set_main_actor(&mut self.actors, ActorRole::Composer, track_composer)
    }

    pub fn album_title(&self) -> Option<&str> {
        Titles::main_title(&self.album.titles).map(|title| title.name.as_str())
    }

    pub fn set_album_title(&mut self, album_title: impl Into<String>) -> bool {
        Titles::set_main_title(&mut self.album.titles, album_title)
    }

    pub fn album_artist(&self) -> Option<&str> {
        Actors::main_actor(self.album.actors.iter(), ActorRole::Artist)
            .map(|actor| actor.name.as_str())
    }

    pub fn set_album_artist(&mut self, album_artist: impl Into<String>) -> bool {
        Actors::set_main_actor(&mut self.album.actors, ActorRole::Artist, album_artist)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackInvalidity {
    MediaSource(SourceInvalidity),
    Release(ReleaseInvalidity),
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
        ValidationContext::new()
            .validate_with(&self.media_source, Self::Invalidity::MediaSource)
            .validate_with(&self.release, Self::Invalidity::Release)
            .validate_with(&self.album, Self::Invalidity::Album)
            .merge_result_with(
                Titles::validate(self.titles.iter()),
                Self::Invalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(self.actors.iter()),
                Self::Invalidity::Actors,
            )
            .validate_with(&self.indexes, Self::Invalidity::Indexes)
            .validate_with(&self.tags, Self::Invalidity::Tags)
            .validate_with(&self.color, Self::Invalidity::Color)
            .validate_with(&self.metrics, Self::Invalidity::Metrics)
            .merge_result(
                self.cues
                    .iter()
                    .fold(ValidationContext::new(), |context, next| {
                        context.validate_with(next, Self::Invalidity::Cue)
                    })
                    .into(),
            )
            .into()
    }
}

pub type Entity = crate::entity::Entity<TrackInvalidity, Track>;

///////////////////////////////////////////////////////////////////////
// PlayCounter
///////////////////////////////////////////////////////////////////////

pub type PlayCount = u64;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayCounter {
    pub last_played_at: Option<DateTime>,
    pub times_played: Option<PlayCount>,
}
