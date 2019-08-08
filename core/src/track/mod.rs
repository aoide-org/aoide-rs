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
};

use lazy_static::lazy_static;
use std::fmt;

///////////////////////////////////////////////////////////////////////
// IndexCount
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IndexCount {
    Index(u16),
    IndexAndCount(u16, u16),
}

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

impl Validate for IndexCount {
    #[allow(clippy::absurd_extreme_comparisons)]
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.index() <= 0 {
            errors.add("index", ValidationError::new("invalid value"));
        }
        if let Some(count) = self.count() {
            if count <= 0 {
                errors.add("count", ValidationError::new("invalid value"));
            } else if self.index() > count {
                errors.add("index", ValidationError::new("value exceeds count"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
// TrackTagging
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

    // "Happy", "Sexy", "Sad", "Melancholic", ...
    pub static ref FACET_MOOD: Facet = Facet::new("mood".into());

    // The set time, e.g. "Warmup", "Opening", "Filler", "Peak", "Closing", "Afterhours", ...
    pub static ref FACET_SESSION: Facet = Facet::new("session".into());

    // Sub-genres or details like "East Coast", "West Coast", ...
    pub static ref FACET_STYLE: Facet = Facet::new("style".into());

    // "Bar", "Beach", "Dinner", "Club", "Lounge", ...
    pub static ref FACET_VENUE: Facet = Facet::new("venue".into());

    // "Bouncy", "Driving", "Dreamy", "Joyful", "Poppy", "Punchy", "Spiritual", "Tropical", "Uplifting" ...
    pub static ref FACET_VIBE: Facet = Facet::new("vibe".into());

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
// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TrackLock {
    Loudness,
    Beats,
    Keys,
}

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Track {
    #[serde(rename = "col", skip_serializing_if = "Vec::is_empty", default)]
    #[validate]
    pub collections: Vec<TrackCollection>,

    #[serde(rename = "src", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(length(min = 1))]
    pub sources: Vec<TrackSource>,

    #[serde(rename = "rel", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub release: Option<ReleaseMetadata>,

    #[serde(rename = "alb", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub album: Option<AlbumMetadata>,

    #[serde(rename = "dsn", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub disc_numbers: Option<IndexCount>,

    #[serde(rename = "trn", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub track_numbers: Option<IndexCount>,

    #[serde(rename = "mvn", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub movement_numbers: Option<IndexCount>,

    #[serde(rename = "tit", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(length(min = 1), custom = "Titles::validate_main_title")]
    pub titles: Vec<Title>,

    #[serde(rename = "act", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(custom = "Actors::validate_main_actor")]
    pub actors: Vec<Actor>,

    #[serde(rename = "tag", skip_serializing_if = "IsDefault::is_default", default)]
    #[validate]
    pub tags: Tags,

    #[serde(rename = "pmk", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(custom = "validate_position_marker_cardinalities")]
    pub position_markers: Vec<PositionMarker>,

    #[serde(rename = "bmk", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(custom = "validate_beat_marker_ranges")]
    pub beat_markers: Vec<BeatMarker>,

    #[serde(rename = "kmk", skip_serializing_if = "Vec::is_empty", default)]
    #[validate(custom = "validate_key_marker_ranges")]
    pub key_markers: Vec<KeyMarker>,

    #[serde(rename = "lck", skip_serializing_if = "Vec::is_empty", default)]
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

    pub fn purge_source_by_uri(&mut self, uri: &str) -> usize {
        let len_before = self.sources.len();
        self.sources.retain(|source| source.uri != uri);
        debug_assert!(self.sources.len() <= len_before);
        len_before - self.sources.len()
    }

    pub fn purge_source_by_uri_prefix(&mut self, uri_prefix: &str) -> usize {
        let len_before = self.sources.len();
        self.sources
            .retain(|source| !source.uri.starts_with(uri_prefix));
        debug_assert!(self.sources.len() <= len_before);
        len_before - self.sources.len()
    }

    pub fn relocate_source_by_uri(&mut self, old_uri: &str, new_uri: &str) -> usize {
        let mut relocated = 0;
        for mut source in &mut self.sources {
            if source.uri == old_uri {
                source.uri = new_uri.to_owned();
                relocated += 1;
            }
        }
        relocated
    }

    pub fn relocate_source_by_uri_prefix(
        &mut self,
        old_uri_prefix: &str,
        new_uri_prefix: &str,
    ) -> usize {
        let mut relocated = 0;
        for mut source in &mut self.sources {
            if source.uri.starts_with(old_uri_prefix) {
                let mut new_uri = String::with_capacity(
                    new_uri_prefix.len() + (source.uri.len() - old_uri_prefix.len()),
                );
                new_uri.push_str(new_uri_prefix);
                new_uri.push_str(&source.uri[old_uri_prefix.len()..]);
                log::debug!("Replacing source URI: {} -> {}", source.uri, new_uri);
                source.uri = new_uri;
                relocated += 1;
            }
        }
        relocated
    }
}

///////////////////////////////////////////////////////////////////////
// TrackEntity
///////////////////////////////////////////////////////////////////////

pub type TrackEntity = Entity<Track>;

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
