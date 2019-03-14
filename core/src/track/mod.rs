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

use crate::{
    metadata::*,
    music::{notation::*, *},
    prelude::*,
};

use chrono::{DateTime, Utc};

use failure;

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;

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

impl IsValid for AudioEncoder {
    fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioContent {
    pub channels: Channels,

    #[serde(rename = "durationMs")]
    pub duration: DurationMs,

    #[serde(rename = "sampleRateHz")]
    pub sample_rate: SampleRateHz,

    #[serde(rename = "bitRateBps")]
    pub bit_rate: BitRateBps,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub loudness: Vec<Loudness>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoder: Option<AudioEncoder>,
}

impl IsValid for AudioContent {
    fn is_valid(&self) -> bool {
        self.channels.is_valid()
            && !self.duration.is_empty()
            && self.sample_rate.is_valid()
            && self.bit_rate.is_valid()
            && self.loudness.iter().all(Loudness::is_valid)
            && self.encoder.as_ref().map_or(true, |e| e.is_valid())
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackSource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSynchronization {
    pub revision: EntityRevision,

    pub when: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSource {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub uri: String,

    // The content_type uniquely identifies a TrackSource of
    // a Track, i.e. no duplicate content types are allowed
    // among the track sources of each track.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_content: Option<AudioContent>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_sync: Option<TrackSynchronization>, // most recent metadata import/export
}

impl TrackSource {
    pub fn filter_slice_by_content_type<'a>(
        sources: &'a [TrackSource],
        content_type: &str,
    ) -> Option<&'a TrackSource> {
        debug_assert!(
            sources
                .iter()
                .filter(|source| source.content_type == content_type)
                .count()
                <= 1
        );
        sources
            .iter()
            .filter(|source| source.content_type == content_type)
            .nth(0)
    }
}

impl IsValid for TrackSource {
    fn is_valid(&self) -> bool {
        // TODO: Validate the URI
        // Currently (2018-05-28) there is no crate that is able to do this.
        // Crate http/hyper: Fail to recognize absolute file paths with the
        // scheme "file" and without an authority, e.g. parsing fails for
        // "file:///path/to/local/file.txt"
        // Crate url: Doesn't care about reserved characters, e.g. parses
        // "file:///path to local/file.txt" successfully
        !self.uri.is_empty() && !self.content_type.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////
/// ColorArgb
///////////////////////////////////////////////////////////////////////

pub type ColorCode = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorArgb(ColorCode); // 0xAARRGGBB

impl ColorArgb {
    const STRING_PREFIX: &'static str = "#";
    const STRING_LEN: usize = 9;

    pub const ALPHA_MASK: ColorCode = 0xff_00_00_00;
    pub const RED_MASK: ColorCode = 0x00_ff_00_00;
    pub const GREEN_MASK: ColorCode = 0x00_00_ff_00;
    pub const BLUE_MASK: ColorCode = 0x00_00_00_ff;

    pub const BLACK: Self = ColorArgb(Self::ALPHA_MASK);
    pub const RED: Self = ColorArgb(Self::ALPHA_MASK | Self::RED_MASK);
    pub const GREEN: Self = ColorArgb(Self::ALPHA_MASK | Self::GREEN_MASK);
    pub const BLUE: Self = ColorArgb(Self::ALPHA_MASK | Self::BLUE_MASK);
    pub const YELLOW: Self = ColorArgb(Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK);
    pub const MAGENTA: Self = ColorArgb(Self::ALPHA_MASK | Self::RED_MASK | Self::BLUE_MASK);
    pub const CYAN: Self = ColorArgb(Self::ALPHA_MASK | Self::GREEN_MASK | Self::BLUE_MASK);
    pub const WHITE: Self =
        ColorArgb(Self::ALPHA_MASK | Self::RED_MASK | Self::GREEN_MASK | Self::BLUE_MASK);

    pub fn code(self) -> ColorCode {
        self.0
    }

    pub fn to_opaque(self) -> Self {
        ColorArgb(self.code() | Self::ALPHA_MASK)
    }

    pub fn to_transparent(self) -> Self {
        ColorArgb(self.code() & !Self::ALPHA_MASK)
    }
}

impl IsValid for ColorArgb {
    fn is_valid(&self) -> bool {
        true
    }
}

impl fmt::Display for ColorArgb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{:08X}", Self::STRING_PREFIX, self.code())
    }
}

impl FromStr for ColorArgb {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == Self::STRING_LEN {
            let (prefix, hex_code) = s.split_at(1);
            if prefix == Self::STRING_PREFIX {
                return u32::from_str_radix(&hex_code, 16)
                    .map(ColorArgb)
                    .map_err(Into::into);
            }
        }
        Err(failure::format_err!("Invalid color code '{}'", s))
    }
}

impl Serialize for ColorArgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
struct ColorDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for ColorDeserializeVisitor {
    type Value = ColorArgb;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a color code string '#AARRGGBB'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ColorArgb::from_str(value).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for ColorArgb {
    fn deserialize<D>(deserializer: D) -> Result<ColorArgb, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackCollection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrackCollection {
    pub uid: EntityUid,

    pub since: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorArgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<usize>,
}

impl TrackCollection {
    pub fn filter_slice_by_uid<'a>(
        collections: &'a [TrackCollection],
        collection_uid: &EntityUid,
    ) -> Option<&'a TrackCollection> {
        debug_assert!(
            collections
                .iter()
                .filter(|collection| &collection.uid == collection_uid)
                .count()
                <= 1
        );
        collections
            .iter()
            .filter(|collection| &collection.uid == collection_uid)
            .nth(0)
    }
}

impl IsValid for TrackCollection {
    fn is_valid(&self) -> bool {
        self.uid.is_valid() && self.color.iter().all(ColorArgb::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_by: Option<String>, // record label

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub licenses: Vec<String>,

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
}

impl IsValid for ReleaseMetadata {
    fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// AlbumMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlbumMetadata {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compilation: Option<bool>,

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
}

impl IsValid for AlbumMetadata {
    fn is_valid(&self) -> bool {
        Titles::is_valid(&self.titles) && Actors::is_valid(&self.actors)
    }
}

///////////////////////////////////////////////////////////////////////
/// IndexCount
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct IndexCount(/*index*/ Option<u32>, /*count*/ Option<u32>);

impl IndexCount {
    pub fn index(&self) -> Option<u32> {
        self.0
    }

    pub fn count(&self) -> Option<u32> {
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
/// TrackMarker
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TrackMark {
    // Cueing: Points without a length
    LoadCue, // default start point when loading a track, only one per track
    HotCue,
    // Fading: Short sections for automatic playback transitions
    FadeIn,  // only one per track
    FadeOut, // only one per track
    // Mixing: Long sections for manual transitions with beat matching
    MixIn,
    MixOut,
    // Sampling
    Sample,
    // Looping
    Loop,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMarkerOffset {
    #[serde(rename = "ms", skip_serializing_if = "DurationMs::is_empty", default)]
    pub duration: DurationMs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples: Option<SamplePosition>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub beats: Option<Beats>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMarkerLength {
    #[serde(rename = "ms")]
    pub duration: DurationMs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples: Option<SampleLength>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub beats: Option<Beats>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TrackMarkerModifier {
    Reverse,
}

impl IsValid for TrackMarkerLength {
    fn is_valid(&self) -> bool {
        self.duration.is_valid()
            && !self.duration.is_empty()
            && self
                .samples
                .map(|samples| samples > SampleLength(0.0))
                .unwrap_or(true)
            && self.beats.map(|beats| beats > 0.0).unwrap_or(true)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackMarker {
    pub mark: TrackMark,

    pub offset: TrackMarkerOffset,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<TrackMarkerLength>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<TrackMarkerModifier>,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorArgb>,
}

impl TrackMarker {
    pub fn is_singular(mark: TrackMark) -> bool {
        match mark {
            TrackMark::LoadCue | TrackMark::FadeIn | TrackMark::FadeOut => true,
            _ => false,
        }
    }
}

impl IsValid for TrackMarker {
    fn is_valid(&self) -> bool {
        self.offset.duration.is_valid()
            && self.length.iter().all(|length| length.duration.is_valid())
            && self.color.iter().all(ColorArgb::is_valid)
            && match self.mark {
                TrackMark::LoadCue | TrackMark::HotCue => self.length.is_none(), // not available
                TrackMark::Sample | TrackMark::Loop => {
                    // mandatory
                    self.length.is_some() && self.length.iter().all(TrackMarkerLength::is_valid)
                }
                _ => self.length.iter().all(TrackMarkerLength::is_valid), // optional
            }
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackTagging
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct TrackTagging;

impl TrackTagging {
    // Some predefined facets that are commonly used and could serve as a starting point

    // ISO 639-2 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
    pub const FACET_LANG: &'static str = "lang";

    // The Content Group aka Grouping field
    pub const FACET_CGROUP: &'static str = "cgroup";

    // "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
    pub const FACET_GENRE: &'static str = "genre";

    // Sub-genres or details like "East Coast", "West Coast", ...
    pub const FACET_STYLE: &'static str = "style";

    // "Happy", "Sexy", "Sad", "Melancholic", "Uplifting", ...
    pub const FACET_MOOD: &'static str = "mood";

    // Decades like "1980s", "2000s", ..., or other time-related feature
    pub const FACET_EPOCH: &'static str = "epoch";

    // "Birthday"/"Bday", "Xmas"/"Holiday"/"Christmas", "Summer", "Vacation", "Wedding", "Workout"...
    pub const FACET_EVENT: &'static str = "event";

    // "Bar", "Beach", "Dinner", "Club", "Lounge", ...
    pub const FACET_VENUE: &'static str = "venue";

    // "Dinner", "Festival", "Party", "Soundcheck", "Top40", "Workout", ...
    pub const FACET_CROWD: &'static str = "crowd";

    // "Warmup", "Opener", "Filler", "Peak", "Closer", "Afterhours", ...
    pub const FACET_SESSION: &'static str = "session";

    // Equivalence tags for marking duplicates or similar/alternative versions within a collection
    pub const FACET_EQUIV: &'static str = "equiv";
}

///////////////////////////////////////////////////////////////////////
/// RefOrigin
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum RefOrigin {
    Track = 1,
    TrackActor = 2,
    Album = 3,
    AlbumActor = 4,
    Release = 5,
}

///////////////////////////////////////////////////////////////////////
/// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TrackLock {
    Loudness,
    Tempo,
    KeySig,
    TimeSig,
}

impl IsValid for TrackLock {
    fn is_valid(&self) -> bool {
        true
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

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<Lyrics>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<SongProfile>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub markers: Vec<TrackMarker>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locks: Vec<TrackLock>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<ScoredTag>, // no duplicate terms per facet allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub comments: Vec<Comment>, // no duplicate owners allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ratings: Vec<Rating>, // no duplicate owners allowed

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
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

    pub fn main_actor(&self, role: ActorRole) -> Option<&Actor> {
        Actors::main_actor(&self.actors, role)
    }

    pub fn album_main_actor(&self, role: ActorRole) -> Option<&Actor> {
        self.album
            .as_ref()
            .and_then(|album| Actors::main_actor(&album.actors, role))
    }
}

impl IsValid for Track {
    fn is_valid(&self) -> bool {
        !self.sources.is_empty()
            && self.sources.iter().all(TrackSource::is_valid)
            && self.collections.iter().all(TrackCollection::is_valid)
            && self.release.iter().all(ReleaseMetadata::is_valid)
            && self.album.iter().all(AlbumMetadata::is_valid)
            && (self.track_numbers.is_valid() || self.track_numbers.is_default())
            && (self.disc_numbers.is_valid() || self.disc_numbers.is_default())
            && (self.movement_numbers.is_valid() || self.movement_numbers.is_default())
            && Titles::is_valid(&self.titles)
            && Actors::is_valid(&self.actors)
            && self.lyrics.iter().all(Lyrics::is_valid)
            && self.profile.iter().all(SongProfile::is_valid)
            && self.markers.iter().all(|marker| {
                marker.is_valid()
                    && (!TrackMarker::is_singular(marker.mark)
                        || self
                            .markers
                            .iter()
                            .filter(|marker2| marker.mark == marker2.mark)
                            .count()
                            <= 1)
            })
            && self.locks.iter().all(TrackLock::is_valid)
            && self.tags.iter().all(ScoredTag::is_valid)
            && self.ratings.iter().all(Rating::is_valid)
            && self.comments.iter().all(Comment::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// TrackEntity
///////////////////////////////////////////////////////////////////////

pub type TrackEntity = Entity<Track>;
