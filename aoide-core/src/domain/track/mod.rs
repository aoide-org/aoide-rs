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

use audio::sample::*;
use audio::signal::*;
use audio::*;
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
    pub const ALPHA_MASK: TrackColorCode = 0xff000000;
    pub const RED_MASK: TrackColorCode = 0x00ff0000;
    pub const GREEN_MASK: TrackColorCode = 0x0000ff00;
    pub const BLUE_MASK: TrackColorCode = 0x000000ff;

    pub const BLACK: Self = Self {
        code: Self::ALPHA_MASK,
    };
    pub const RED: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK,
    };
    pub const GREEN: Self = Self {
        code: Self::ALPHA_MASK | Self::GREEN_MASK,
    };
    pub const BLUE: Self = Self {
        code: Self::ALPHA_MASK | Self::BLUE_MASK,
    };
    pub const YELLOW: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK,
    };
    pub const MAGENTA: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK | Self::BLUE_MASK,
    };
    pub const CYAN: Self = Self {
        code: Self::ALPHA_MASK | Self::GREEN_MASK | Self::BLUE_MASK,
    };
    pub const WHITE: Self = Self {
        code: Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK,
    };

    pub fn is_valid(&self) -> bool {
        true
    }

    pub fn into_opaque(&self) -> Self {
        Self {
            code: self.code | Self::ALPHA_MASK,
        }
    }

    pub fn into_transparent(&self) -> Self {
        Self {
            code: self.code & !Self::ALPHA_MASK,
        }
    }
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
            && self.color.iter().all(TrackColor::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// Titles
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub isrc: String, // International Standard Recording Code (ISO 3901)

    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub mbrainz_id: Uuid, // MusicBrainz Release Track Id

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub spotify_id: String, // excl. "spotify:track:" prefix

    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub acoust_id: Uuid,
}

impl TrackIdentity {
    pub fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// ReleaseIdentity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseIdentity {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub ean: String,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub upc: String,

    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub mbrainz_id: Uuid, // MusicBrainz Release Id

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub asin: String,
}

impl ReleaseIdentity {
    pub fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<ReleaseIdentity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl ReleaseMetadata {
    pub fn is_valid(&self) -> bool {
        self.identity.iter().all(ReleaseIdentity::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// AlbumIdentity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumIdentity {
    #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")]
    pub mbrainz_id: Uuid, // MusicBrainz Release Group Id

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub spotify_id: String, // excl. "spotify:album:" prefix
}

impl AlbumIdentity {
    pub fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// AlbumMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<AlbumIdentity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseMetadata>,

    pub titles: Titles,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compilation: Option<bool>,
}

impl AlbumMetadata {
    pub fn is_valid(&self) -> bool {
        self.identity.iter().all(AlbumIdentity::is_valid)
            && self.release.iter().all(ReleaseMetadata::is_valid) && self.titles.is_valid()
            && self.actors.iter().all(Actor::is_valid)
    }
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

impl TrackNumbers {
    pub fn is_valid(&self) -> bool {
        self.this <= self.total
    }
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

impl DiscNumbers {
    pub fn is_valid(&self) -> bool {
        self.this <= self.total
    }
}

impl fmt::Display for DiscNumbers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.this, self.total)
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackMarker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TrackMark {
    LoadCue,
    HotCue,
    FadeIn,
    FadeOut,
    Loop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMarker {
    pub mark: TrackMark,

    pub position: Duration,

    #[serde(skip_serializing_if = "Duration::is_empty", default)]
    pub duration: Duration,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TrackColor>,
}

impl TrackMarker {
    pub fn is_singular(mark: TrackMark) -> bool {
        match mark {
            TrackMark::LoadCue | TrackMark::FadeIn | TrackMark::FadeOut => true,
            TrackMark::HotCue | TrackMark::Loop => false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.position.is_valid() && self.duration.is_valid() && match self.mark {
            TrackMark::LoadCue | TrackMark::HotCue => self.duration.is_empty(), // not available
            TrackMark::FadeIn | TrackMark::FadeOut => true,                     // optional
            TrackMark::Loop => !self.duration.is_empty(),                       // mandatory
        }
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

impl MusicMetadata {
    pub fn is_valid(&self) -> bool {
        self.loudness.iter().all(Loudness::is_valid) && self.tempo.iter().all(Tempo::is_valid)
            && self.time_signature.iter().all(TimeSignature::is_valid)
            && self.key_signature.iter().all(KeySignature::is_valid)
            && self.classifications.iter().all(Classification::is_valid)
            && self.classifications.iter().all(|classification| {
                classification.is_valid() && self.is_classifier_unique(classification.classifier)
            })
    }

    pub fn has_classifier(&self, classifier: Classifier) -> bool {
        self.classifications
            .iter()
            .any(|classification| classification.classifier == classifier)
    }

    fn is_classifier_unique(&self, classifier: Classifier) -> bool {
        self.classifications
            .iter()
            .filter(|classification| classification.classifier == classifier)
            .count() <= 1
    }

    pub fn classification(&self, classifier: Classifier) -> Option<&Classification> {
        assert!(self.is_classifier_unique(classifier));
        self.classifications
            .iter()
            .filter(|classification| classification.classifier == classifier)
            .nth(0)
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackLyrics
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackLyrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit: Option<bool>,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub text: String,
}

impl TrackLyrics {
    pub fn is_empty(&self) -> bool {
        self.explicit.is_none() && self.text.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        true
    }
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
/// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TrackLock {
    Loudness,
    Tempo,
    TimeSig,
    KeySig,
}

impl TrackLock {
    pub fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackBody
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackBody {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub resources: Vec<TrackResource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<TrackIdentity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<AlbumMetadata>,

    pub titles: Titles,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_numbers: Option<TrackNumbers>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc_numbers: Option<DiscNumbers>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub music: Option<MusicMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<TrackLyrics>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub markers: Vec<TrackMarker>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locks: Vec<TrackLock>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<Tag>, // no duplicate terms per facet allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ratings: Vec<Rating>, // no duplicate owners allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub comments: Vec<Comment>, // no duplicate owners allowed
}

impl TrackBody {
    pub fn is_valid(&self) -> bool {
        !self.resources.is_empty() && self.resources.iter().all(TrackResource::is_valid)
            && self.identity.iter().all(TrackIdentity::is_valid)
            && self.album.iter().all(AlbumMetadata::is_valid) && self.titles.is_valid()
            && self.actors.iter().all(Actor::is_valid)
            && self.track_numbers.iter().all(TrackNumbers::is_valid)
            && self.disc_numbers.iter().all(DiscNumbers::is_valid)
            && self.music.iter().all(MusicMetadata::is_valid)
            && self.lyrics.iter().all(TrackLyrics::is_valid)
            && self.markers.iter().all(|marker| {
                marker.is_valid()
                    && (!TrackMarker::is_singular(marker.mark)
                        || self.markers
                            .iter()
                            .filter(|marker2| marker.mark == marker2.mark)
                            .count() <= 1)
            }) && self.locks.iter().all(TrackLock::is_valid)
            && self.tags.iter().all(Tag::is_valid)
            && self.ratings.iter().all(Rating::is_valid)
            && self.comments.iter().all(Comment::is_valid)
    }

    pub fn resource<'a>(&'a self, collection_uid: &CollectionUid) -> Option<&'a TrackResource> {
        assert!(
            self.resources
                .iter()
                .filter(|resource| &resource.collection.uid == collection_uid)
                .count() <= 1
        );
        self.resources
            .iter()
            .filter(|resource| &resource.collection.uid == collection_uid)
            .nth(0)
    }

    pub fn has_collection(&self, collection_uid: &CollectionUid) -> bool {
        self.resource(collection_uid).is_some()
    }

    pub fn actors_to_string(&self, role_opt: Option<ActorRole>) -> Option<String> {
        Actor::actors_to_string(&self.actors, role_opt)
    }

    pub fn artists_to_string(&self) -> Option<String> {
        self.actors_to_string(Some(ActorRole::Artist))
    }

    pub fn album_actors_to_string(&self, role_opt: Option<ActorRole>) -> Option<String> {
        Actor::actors_to_string(&self.actors, role_opt)
    }

    pub fn album_artists_to_string(&self) -> Option<String> {
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

    pub fn replace_body(&mut self, body: TrackBody) {
        self.body = body;
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use mime_guess;
    use serde_json;

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
        let resources = vec![
            TrackResource {
                collection: TrackCollection {
                    uid: EntityUidGenerator::generate_uid(),
                    since: Utc::now(),
                },
                source,
                color: Some(TrackColor::RED),
            },
        ];
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
