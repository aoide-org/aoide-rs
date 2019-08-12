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

use aoide_serde::tag::{Facet, Label};

use crate::{
    entity::*,
    metadata::{actor::*, title::*, Tags},
};

use lazy_static::lazy_static;
use std::fmt;

///////////////////////////////////////////////////////////////////////
// IndexCount
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    fn validate(&self) -> ValidationResult<()> {
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

///////////////////////////////////////////////////////////////////////
// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
