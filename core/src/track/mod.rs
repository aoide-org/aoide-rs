// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

pub mod album;
pub mod collection;
pub mod marker;
pub mod release;
pub mod source;
pub mod tag;

use self::{
    album::*,
    collection::*,
    marker::{beat::*, key::*, position::*},
    release::*,
    source::*,
};

use crate::{actor::*, tag::*, title::*};

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexCount {
    Index(u16),
    IndexAndCount(u16, u16),
}

const MIN_INDEX: u16 = 1;

const MIN_COUNT: u16 = 1;

impl IndexCount {
    pub fn index(self) -> u16 {
        use IndexCount::*;
        match self {
            Index(idx) => idx,
            IndexAndCount(idx, _) => idx,
        }
    }

    pub fn count(self) -> Option<u16> {
        use IndexCount::*;
        match self {
            Index(_) => None,
            IndexAndCount(_, cnt) => Some(cnt),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IndexCountValidation {
    IndexMin(u16),
    IndexMax(u16),
    CountMin(u16),
}

impl Validate for IndexCount {
    type Validation = IndexCountValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            self.index() < MIN_INDEX,
            IndexCountValidation::IndexMin(MIN_INDEX),
        );
        if let Some(count) = self.count() {
            if count < MIN_COUNT {
                context.add_violation(IndexCountValidation::CountMin(MIN_COUNT));
            } else if self.index() > count {
                context.add_violation(IndexCountValidation::IndexMax(count));
            }
        }
        context.into_result()
    }
}

impl fmt::Display for IndexCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use IndexCount::*;
        match self {
            Index(idx) => write!(f, "{}", idx),
            IndexAndCount(idx, cnt) => write!(f, "{}/{}", idx, cnt),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TrackLock {
    Loudness,
    Beats,
    Keys,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub collections: Vec<Collection>,

    pub sources: Vec<Source>,

    pub titles: Vec<Title>,
    pub actors: Vec<Actor>,

    pub album: Option<Album>,
    pub release: Option<Release>,

    pub disc_numbers: Option<IndexCount>,
    pub track_numbers: Option<IndexCount>,
    pub movement_numbers: Option<IndexCount>,

    pub position_markers: Vec<PositionMarker>,
    pub beat_markers: Vec<BeatMarker>,
    pub key_markers: Vec<KeyMarker>,

    pub tags: Vec<Tag>,

    pub locks: Vec<TrackLock>,
}

#[derive(Copy, Clone, Debug)]
pub enum TrackValidation {
    Collections(CollectionsValidation),
    Sources(SourcesValidation),
    Titles(TitlesValidation),
    Actors(ActorsValidation),
    Album(AlbumValidation),
    Release(ReleaseValidation),
    DiscNumbers(IndexCountValidation),
    TrackNumbers(IndexCountValidation),
    MovementNumbers(IndexCountValidation),
    PositionMarkers(PositionMarkersValidation),
    BeatMarkers(BeatMarkersValidation),
    KeyMarkers(KeyMarkersValidation),
    Tags(TagsValidation),
}

impl Validate for Track {
    type Validation = TrackValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(
            Collections::validate(&self.collections),
            TrackValidation::Collections,
        );
        context.map_and_merge_result(Sources::validate(&self.sources), TrackValidation::Sources);
        context.map_and_merge_result(Titles::validate(&self.titles), TrackValidation::Titles);
        context.map_and_merge_result(Actors::validate(&self.actors), TrackValidation::Actors);
        if let Some(ref album) = self.album {
            context.map_and_merge_result(album.validate(), TrackValidation::Album);
        }
        if let Some(ref release) = self.release {
            context.map_and_merge_result(release.validate(), TrackValidation::Release);
        }
        if let Some(ref disc_numbers) = self.disc_numbers {
            context.map_and_merge_result(disc_numbers.validate(), TrackValidation::DiscNumbers);
        }
        if let Some(ref track_numbers) = self.track_numbers {
            context.map_and_merge_result(track_numbers.validate(), TrackValidation::TrackNumbers);
        }
        if let Some(ref movement_numbers) = self.movement_numbers {
            context.map_and_merge_result(
                movement_numbers.validate(),
                TrackValidation::MovementNumbers,
            );
        }
        context.map_and_merge_result(
            PositionMarkers::validate(&self.position_markers),
            TrackValidation::PositionMarkers,
        );
        context.map_and_merge_result(
            BeatMarkers::validate(&self.beat_markers),
            TrackValidation::BeatMarkers,
        );
        context.map_and_merge_result(
            KeyMarkers::validate(&self.key_markers),
            TrackValidation::KeyMarkers,
        );
        context.map_and_merge_result(Tags::validate(&self.tags), TrackValidation::Tags);
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

// TODO
