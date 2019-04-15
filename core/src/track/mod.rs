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

use super::*;

pub mod album;
pub mod collection;
pub mod marker;
pub mod release;
pub mod source;

use self::{album::*, collection::*, marker::*, release::*, source::*};

use crate::{
    entity::*,
    metadata::{actor::*, title::*, *},
    music::{key::*, time::*},
};

use lazy_static::lazy_static;
use std::fmt;

///////////////////////////////////////////////////////////////////////
/// IndexCount
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct IndexCount(/*index*/ Option<u16>, /*count*/ Option<u16>);

impl IndexCount {
    pub fn index(self) -> Option<u16> {
        self.0
    }

    pub fn count(self) -> Option<u16> {
        self.1
    }
}

impl IsValid for IndexCount {
    fn is_valid(&self) -> bool {
        match (self.index(), self.count()) {
            (None, None) => true,
            (Some(index), None) => index > 0,
            (None, Some(count)) => count > 0,
            (Some(index), Some(count)) => index > 0 && index <= count,
        }
    }
}

impl fmt::Display for IndexCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.index(), self.count()) {
            (None, None) => write!(f, ""),
            (Some(index), None) => write!(f, "{}", index),
            (None, Some(count)) => write!(f, "/{}", count),
            (Some(index), Some(count)) => write!(f, "{}/{}", index, count),
        }
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackTagging
///////////////////////////////////////////////////////////////////////

// Some predefined facets that are commonly used and could serve as
// a starting point for complex tagging schemes
lazy_static! {
    // The Content Group aka Grouping field
    pub static ref FACET_CGROUP: Facet = Facet::new("cgroup".into());

    // "Dinner", "Festival", "Party", "Soundcheck", "Top40", "Workout", ...
    pub static ref FACET_CROWD: Facet = Facet::new("crowd".into());

    // Decades like "1980s", "2000s", ..., or other time-based properties
    pub static ref FACET_EPOCH: Facet = Facet::new("epoch".into());

    // "Birthday"/"Bday", "Xmas"/"Holiday"/"Christmas", "Summer", "Vacation", "Wedding", ...
    pub static ref FACET_EVENT: Facet = Facet::new("event".into());

    // "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
    pub static ref FACET_GENRE: Facet = Facet::new("genre".into());

    // ISO 639-2 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
    pub static ref FACET_LANG: Facet = Facet::new("lang".into());

    // "Happy", "Sexy", "Sad", "Melancholic", "Uplifting", ...
    pub static ref FACET_MOOD: Facet = Facet::new("mood".into());

    // "Warmup", "Opener", "Filler", "Peak", "Closer", "Afterhours", ...
    pub static ref FACET_SESSION: Facet = Facet::new("session".into());

    // Sub-genres or details like "East Coast", "West Coast", ...
    pub static ref FACET_STYLE: Facet = Facet::new("style".into());

    // "Bar", "Beach", "Dinner", "Club", "Lounge", ...
    pub static ref FACET_VENUE: Facet = Facet::new("venue".into());

    // Select a subset of a collection, i.e. a virtual "crate".
    // Examples: "DJ", "Mobile", ...
    pub static ref FACET_CRATE: Facet = Facet::new("crate".into());

    // Facets for various musical features. These tags are only scored,
    // but should not be labeled.
    // See also: [Spotify Audio Features](https://developer.spotify.com/documentation/web-api/reference/tracks/get-audio-features/)
    pub static ref FACET_ENERGY: Label = Label::new("energy".into());
    pub static ref FACET_DANCEABILITY: Label = Label::new("danceability".into());
    pub static ref FACET_VALENCE: Label = Label::new("valence".into()); // a measure for happiness
    pub static ref FACET_ACOUSTICNESS: Label = Label::new("acousticness".into());
    pub static ref FACET_INSTRUMENTALNESS: Label = Label::new("instrumentalness".into());
    pub static ref FACET_LIVENESS: Label = Label::new("liveness".into());
    pub static ref FACET_SPEECHINESS: Label = Label::new("speechiness".into());
    pub static ref FACET_POPULARITY: Label = Label::new("popularity".into());
}

///////////////////////////////////////////////////////////////////////
/// TrackMusic
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMusic {
    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub tempo: TempoBpm,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub key: KeySignature,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub timing: TimeSignature,
}

impl IsValid for TrackMusic {
    fn is_valid(&self) -> bool {
        (self.tempo.is_valid() || self.tempo.is_default())
            && (self.key.is_valid() || self.key.is_default())
            && (self.timing.is_valid() || self.timing.is_default())
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TrackLock {
    Loudness,
    Tempo,
    Key,
    Timing,
}

impl IsValid for TrackLock {
    fn is_valid(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
struct TrackLocks;

impl TrackLocks {
    pub fn all_valid(slice: &[TrackLock]) -> bool {
        slice.iter().all(IsValid::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Track {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub collections: Vec<TrackCollection>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sources: Vec<TrackSource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<AlbumMetadata>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub track_numbers: IndexCount,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub disc_numbers: IndexCount,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub movement_numbers: IndexCount,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub music: TrackMusic,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub tags: Tags,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub position_markers: Vec<PositionMarker>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locks: Vec<TrackLock>,
}

impl Track {
    pub fn collection<'a>(&'a self, collection_uid: &EntityUid) -> Option<&'a TrackCollection> {
        self.collections
            .iter()
            .filter(|collection| &collection.uid == collection_uid)
            .nth(0)
    }

    pub fn has_collection(&self, collection_uid: &EntityUid) -> bool {
        self.collection(collection_uid).is_some()
    }

    pub fn main_title(&self) -> Option<&Title> {
        Titles::main_title(&self.titles)
    }

    pub fn main_actor(&self, role: ActorRole) -> Option<&Actor> {
        Actors::main_actor(&self.actors, role)
    }

    pub fn main_album_title(&self) -> Option<&Title> {
        self.album.as_ref().and_then(AlbumMetadata::main_title)
    }

    pub fn main_album_actor(&self, role: ActorRole) -> Option<&Actor> {
        self.album.as_ref().and_then(|album| album.main_actor(role))
    }

    /*
    pub const fn genres(&self) -> Vec<&FacetedTag> {
        self.tags.faceted
            .iter()
            .filter(|f| f == &*FACET_GENRE)
            .collect()
    }
    */
}

impl IsValid for Track {
    fn is_valid(&self) -> bool {
        !self.sources.is_empty()
            && self.sources.iter().all(IsValid::is_valid)
            && self.collections.iter().all(IsValid::is_valid)
            && self.release.iter().all(IsValid::is_valid)
            && self.album.iter().all(IsValid::is_valid)
            && (self.track_numbers.is_valid() || self.track_numbers.is_default())
            && (self.disc_numbers.is_valid() || self.disc_numbers.is_default())
            && (self.movement_numbers.is_valid() || self.movement_numbers.is_default())
            && self.music.is_valid()
            && Titles::all_valid(&self.titles)
            && Actors::all_valid(&self.actors)
            && self.tags.is_valid()
            && PositionMarker::all_valid(&self.position_markers)
            && TrackLocks::all_valid(&self.locks)
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackEntity
///////////////////////////////////////////////////////////////////////

pub type TrackEntity = Entity<Track>;

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
