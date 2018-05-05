// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use audio::*;
use audio::sample::*;
use audio::signal::*;
use domain::collection::*;
use domain::entity::*;
use domain::metadata::*;
use domain::music::*;

use chrono::{DateTime, Utc};

use std::fmt;

use uuid::Uuid;

///////////////////////////////////////////////////////////////////////
/// AudioEncoder
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioEncoder {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
}

impl AudioEncoder {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioContent {
    pub duration: Duration,

    pub channels: Channels,

    pub samplerate: SampleRate,

    pub bitrate: BitRate,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoder: Option<AudioEncoder>,
}

impl AudioContent {
    pub fn is_valid(&self) -> bool {
        !self.duration.is_empty() && self.channels.is_valid() && self.samplerate.is_valid()
            && self.bitrate.is_valid()
            && self.encoder.as_ref().map_or(true, |e| e.is_valid())
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackSource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSynchronization {
    pub when: DateTime<Utc>,

    pub revision: EntityRevision,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSource {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub uri: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<TrackSynchronization>, // most recent metadata import/export

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_content: Option<AudioContent>,
}

impl TrackSource {
    pub fn is_valid(&self) -> bool {
        !self.uri.is_empty() && !self.content_type.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackResource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrackCollection {
    pub uid: CollectionUid,

    pub since: DateTime<Utc>,
}

impl TrackCollection {
    pub fn is_valid(&self) -> bool {
        self.uid.is_valid()
    }
}

pub type TrackColorCode = u32; // 0xAARRGGBB

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackColor {
    pub code: TrackColorCode,
}

impl TrackColor {
    pub const BLACK: Self = Self { code: 0x000000 };
    pub const WHITE: Self = Self { code: 0xffffff };
    pub const RED: Self = Self { code: 0xff0000 };
    pub const GREEN: Self = Self { code: 0x00ff00 };
    pub const BLUE: Self = Self { code: 0x0000ff };
    pub const YELLOW: Self = Self { code: 0xffff00 };
    pub const MAGENTA: Self = Self { code: 0xff00ff };
    pub const CYAN: Self = Self { code: 0x00ffff };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackResource {
    pub collection: TrackCollection,

    pub source: TrackSource,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TrackColor>,
}

impl TrackResource {
    pub fn is_valid(&self) -> bool {
        self.collection.is_valid() && self.source.is_valid()
    }
}

///////////////////////////////////////////////////////////////////////
/// Titles
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Titles {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
}

impl Titles {
    pub fn is_valid(&self) -> bool {
        !self.title.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackIdentity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackIdentity {
    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub acoust_id: Uuid,

    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub mbrainz_id: Uuid, // MusicBrainz Release Track Id

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub spotify_id: String, // excl. "spotify:track:" prefix
}

///////////////////////////////////////////////////////////////////////
/// AlbumIdentity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumIdentity {
    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub mbrainz_id: Uuid, // MusicBrainz Release Id

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub spotify_id: String, // excl. "spotify:album:" prefix
}

///////////////////////////////////////////////////////////////////////
/// AlbumMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<AlbumIdentity>,

    pub titles: Titles,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compilation: Option<bool>,
}

///////////////////////////////////////////////////////////////////////
/// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

///////////////////////////////////////////////////////////////////////
/// TrackNumbers
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackNumbers {
    pub this: u32,

    pub total: u32,
}

impl fmt::Display for TrackNumbers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.this, self.total)
    }
}

///////////////////////////////////////////////////////////////////////
/// DiscNumbers
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DiscNumbers {
    pub this: u32,

    pub total: u32,
}

impl fmt::Display for DiscNumbers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.this, self.total)
    }
}

///////////////////////////////////////////////////////////////////////
/// MusicMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MusicMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loudness: Option<Loudness>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<Tempo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_signature: Option<TimeSignature>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_signature: Option<KeySignature>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub classifications: Vec<Classification>, // no duplicate classifiers allowed
}

///////////////////////////////////////////////////////////////////////
/// TrackTag
///////////////////////////////////////////////////////////////////////

pub struct TrackTag;

impl TrackTag {
    // Some predefined facets that are commonly used and could serve as a starting point

    // ISO 639-2 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
    pub const FACET_LANG: &'static str = "lang";

    // "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
    pub const FACET_GENRE: &'static str = "genre";

    // "1980s", "2000s", "Evergreen", "Classic", ...
    pub const FACET_STYLE: &'static str = "style";

    // "Happy", "Sexy", "Sad", "Melancholic", "Uplifting", ...
    pub const FACET_MOOD: &'static str = "mood";

    // "Bar", "Lounge", "Beach", "Party", "Club", ...
    pub const FACET_VENUE: &'static str = "venue";

    // "Wedding", "Birthday", "Festival", ...
    pub const FACET_CROWD: &'static str = "crowd";

    // "Warmup", "Opener", "Filler", "Peak", "Closer", "Afterhours", ...
    pub const FACET_SETTIME: &'static str = "settime";
}

///////////////////////////////////////////////////////////////////////
/// TrackBody
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackBody {
    pub titles: Titles,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<AlbumMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_numbers: Option<TrackNumbers>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc_numbers: Option<DiscNumbers>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<TrackIdentity>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub resources: Vec<TrackResource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub music: Option<MusicMetadata>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<Tag>, // no duplicate terms per facet allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ratings: Vec<Rating>, // no duplicate owners allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub comments: Vec<Comment>, // no duplicate owners allowed
}

impl TrackBody {
    pub fn is_valid(&self) -> bool {
        self.titles.is_valid() && !self.resources.is_empty()
            && (self.resources
                .iter()
                .filter(|source| source.is_valid())
                .count() == self.resources.len())
    }

    pub fn actors_to_string(&self, role_opt: Option<ActorRole>) -> String {
        Actor::actors_to_string(&self.actors, role_opt)
    }

    pub fn artists_to_string(&self) -> String {
        self.actors_to_string(Some(ActorRole::Artist))
    }

    pub fn album_actors_to_string(&self, role_opt: Option<ActorRole>) -> String {
        Actor::actors_to_string(&self.actors, role_opt)
    }

    pub fn album_artists_to_string(&self) -> String {
        self.album_actors_to_string(Some(ActorRole::Artist))
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackEntity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackEntity {
    header: EntityHeader,

    body: TrackBody,
}

impl TrackEntity {
    pub fn new(header: EntityHeader, body: TrackBody) -> Self {
        Self { header, body }
    }

    pub fn with_body(body: TrackBody) -> Self {
        let uid = EntityUidGenerator::generate_uid();
        let header = EntityHeader::with_uid(uid);
        Self { header, body }
    }

    pub fn is_valid(&self) -> bool {
        self.header.is_valid() && self.body.is_valid()
    }

    pub fn header<'a>(&'a self) -> &'a EntityHeader {
        &self.header
    }

    pub fn body<'a>(&'a self) -> &'a TrackBody {
        &self.body
    }

    pub fn body_mut<'a>(&'a mut self) -> &'a mut TrackBody {
        &mut self.body
    }

    pub fn update_revision(&mut self, next_revision: EntityRevision) {
        self.header.update_revision(next_revision);
    }

    pub fn revert_revision(&mut self, prev_revision: EntityRevision) {
        self.header.revert_revision(prev_revision);
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use mime_guess;

    #[test]
    fn serialize_json() {
        let classifications = vec![
            Classification::new(Classifier::Energy, 0.1),
            Classification::new(Classifier::Popularity, 0.9),
        ];
        let music = MusicMetadata {
            classifications,
            loudness: Some(Loudness::EBUR128LUFS(LUFS { db: -2.3 })),
            ..Default::default()
        };
        let comments = vec![
            Comment::new_anonymous("Some anonymous notes about this track"),
        ];
        let uri = "subfolder/test.mp3";
        let source = TrackSource {
            uri: uri.to_string(),
            synchronization: Some(TrackSynchronization {
                when: Utc::now(),
                revision: EntityRevision::initial(),
            }),
            content_type: mime_guess::guess_mime_type(uri).to_string(),
            audio_content: None,
        };
        let resources = vec![TrackResource {
            collection: TrackCollection {
                uid: EntityUidGenerator::generate_uid(),
                since: Utc::now(),
            },
            source,
            color: Some(TrackColor::RED),
        }];
        let tags = vec![
            Tag::new_faceted(TrackTag::FACET_STYLE, "1980s", 0.8),
            Tag::new_faceted("STYLE", "1990s", 0.3),
            Tag::new_faceted(TrackTag::FACET_SETTIME, "Filler", 0.6),
            Tag::new("non-faceted tag", 1.0),
        ];
        let body = TrackBody {
            resources,
            music: Some(music),
            tags,
            comments,
            ..Default::default()
        };
        let uid = EntityUidGenerator::generate_uid();
        let header = EntityHeader::with_uid(uid);
        let entity = TrackEntity { header, body };
        let entity_json = serde_json::to_string(&entity).unwrap();
        assert_ne!("{}", entity_json);
        println!("Track Entity (JSON): {}", entity_json);
    }

    #[test]
    fn star_rating() {
        assert_eq!(0, Rating::new_anonymous(0.0).star_rating(5));
        assert_eq!(1, Rating::new_anonymous(0.01).star_rating(5));
        assert_eq!(1, Rating::new_anonymous(0.2).star_rating(5));
        assert_eq!(2, Rating::new_anonymous(0.21).star_rating(5));
        assert_eq!(2, Rating::new_anonymous(0.4).star_rating(5));
        assert_eq!(3, Rating::new_anonymous(0.41).star_rating(5));
        assert_eq!(3, Rating::new_anonymous(0.6).star_rating(5));
        assert_eq!(4, Rating::new_anonymous(0.61).star_rating(5));
        assert_eq!(4, Rating::new_anonymous(0.8).star_rating(5));
        assert_eq!(5, Rating::new_anonymous(0.81).star_rating(5));
        assert_eq!(5, Rating::new_anonymous(0.99).star_rating(5));
        assert_eq!(5, Rating::new_anonymous(1.0).star_rating(5));
        for max_stars in 4..10 {
            for stars in 0..max_stars {
                assert_eq!(
                    stars,
                    Rating::new_anonymous(Rating::rating_from_stars(stars, max_stars))
                        .star_rating(max_stars)
                );
            }
        }
    }
}
