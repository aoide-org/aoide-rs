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
            release: Default::default(),
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
            )
            .into()
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayCounter {
    pub last_played_at: Option<DateTime>,
    pub times_played: Option<PlayCount>,
}
