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

use domain::audio::*;
use domain::audio::sample::*;
use domain::audio::signal::*;
use domain::metadata::*;
use domain::music::*;

use chrono::{DateTime, Utc};

use std::fmt;

use uuid::Uuid;

///////////////////////////////////////////////////////////////////////
/// MediaResource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaResource {
  uri: String,
  collection_uid: String,
  #[serde(skip_serializing_if = "Option::is_none")] metadata_imported: Option<DateTime<Utc>>, // most recent metadata import
  #[serde(skip_serializing_if = "Option::is_none")] metadata_exported: Option<DateTime<Utc>>, // most recent metadata export
}

impl MediaResource {
  pub fn is_valid(&self) -> bool {
    !self.uri.is_empty() && !self.collection_uid.is_empty()
  }
}

///////////////////////////////////////////////////////////////////////
/// MediaMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
  #[serde(skip_serializing_if = "Option::is_none", rename = "type")] content_type: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")]
  pub resources: Vec<MediaResource>,
}

impl MediaMetadata {
  pub fn is_empty(&self) -> bool {
    self.content_type.is_none() && self.resources.is_empty()
  }

  pub fn is_valid(&self) -> bool {
    self.content_type.is_some() && !self.resources.is_empty()
      && (self.resources.iter().filter(|loc| loc.is_valid()).count() == self.resources.len())
  }
}

///////////////////////////////////////////////////////////////////////
/// AudioMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioMetadata {
  duration: Duration,
  channels: Channels,
  samplerate: SampleRate,
  bitrate: BitRate,
  #[serde(skip_serializing_if = "Option::is_none")] encoder: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")] encoder_settings: Option<String>,
}

impl AudioMetadata {
  pub fn is_valid(&self) -> bool {
    !self.duration.is_empty() && self.channels.is_valid() && self.samplerate.is_valid()
      && self.bitrate.is_valid()
  }
}

///////////////////////////////////////////////////////////////////////
/// Titles
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Titles {
  title: String, // mandatory, i.e. not empty
  subtitle: Option<String>,
}

impl Titles {
  pub fn is_valid(&self) -> bool {
    !self.title.is_empty()
  }

  pub fn is_empty(&self) -> bool {
    self.title.is_empty() && self.subtitle.is_none()
  }
}

///////////////////////////////////////////////////////////////////////
/// TrackIdentity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackIdentity {
  #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")] acoust_id: Uuid,
  #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")] mbrainz_id: Uuid, // MusicBrainz Release Track Id
  #[serde(skip_serializing_if = "String::is_empty", default = "String::default")]
  spotify_id: String, // excl. "spotify:track:" prefix
}

impl TrackIdentity {
  pub fn is_empty(&self) -> bool {
    self.acoust_id.is_nil() && self.mbrainz_id.is_nil() && self.spotify_id.is_empty()
  }
}

///////////////////////////////////////////////////////////////////////
/// AlbumIdentity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumIdentity {
  #[serde(skip_serializing_if = "Uuid::is_nil", default = "Uuid::nil")] mbrainz_id: Uuid, // MusicBrainz Release Id
  #[serde(skip_serializing_if = "String::is_empty", default = "String::default")]
  spotify_id: String, // excl. "spotify:album:" prefix
}

impl AlbumIdentity {
  pub fn is_empty(&self) -> bool {
    self.mbrainz_id.is_nil() && self.spotify_id.is_empty()
  }
}

///////////////////////////////////////////////////////////////////////
/// AlbumMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMetadata {
  #[serde(skip_serializing_if = "Titles::is_empty", default = "Titles::default")]
  pub titles: Titles,
  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")] actors: Vec<Actor>,
  #[serde(skip_serializing_if = "Option::is_none")] grouping: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")] compilation: Option<bool>,
}

impl AlbumMetadata {
  pub fn is_empty(&self) -> bool {
    self.titles.is_empty() && self.actors.is_empty() && self.grouping.is_none()
      && self.compilation.is_none()
  }
}

///////////////////////////////////////////////////////////////////////
/// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseMetadata {
  #[serde(skip_serializing_if = "Option::is_none")] released: Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")] label: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")] copyright: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")] license: Option<String>,
}

impl ReleaseMetadata {
  pub fn is_empty(&self) -> bool {
    self.label.is_none() && self.copyright.is_none() && self.license.is_none()
      && self.released.is_none()
  }
}

///////////////////////////////////////////////////////////////////////
/// TrackNumbers
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackNumbers {
  current: u32,
  total: u32,
}

impl fmt::Display for TrackNumbers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}/{}", self.current, self.total)
  }
}

///////////////////////////////////////////////////////////////////////
/// DiscNumbers
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscNumbers {
  current: u32,
  total: u32,
}

impl fmt::Display for DiscNumbers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}/{}", self.current, self.total)
  }
}

///////////////////////////////////////////////////////////////////////
/// MusicMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicMetadata {
  #[serde(skip_serializing_if = "Option::is_none")] loudness: Option<Loudness>,
  #[serde(skip_serializing_if = "Option::is_none")] tempo: Option<Tempo>,
  #[serde(skip_serializing_if = "Option::is_none")] time_signature: Option<TimeSignature>,
  #[serde(skip_serializing_if = "Option::is_none")] key_signature: Option<KeySignature>,
  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")]
  pub classifications: Vec<Classification>, // no duplicate classifiers allowed
}

impl MusicMetadata {
  pub fn is_empty(&self) -> bool {
    self.loudness.is_none() && self.tempo.is_none() && self.time_signature.is_none()
      && self.key_signature.is_none() && self.classifications.is_empty()
  }
}

///////////////////////////////////////////////////////////////////////
/// TrackTag
///////////////////////////////////////////////////////////////////////

pub struct TrackTag;

impl TrackTag {
  // Predefined facets
  pub const FACET_LANG: &'static str = "lang"; // ISO 639-2 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
  pub const FACET_GENRE: &'static str = "genre"; // e.g. "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
  pub const FACET_STYLE: &'static str = "style";
  pub const FACET_MOOD: &'static str = "mood";
  pub const FACET_DECADE: &'static str = "decade"; // e.g. "1980s", "2000s", ...
}

///////////////////////////////////////////////////////////////////////
/// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMetadata {
  pub media: MediaMetadata,

  pub audio: AudioMetadata,

  #[serde(skip_serializing_if = "Titles::is_empty", default = "Titles::default")]
  pub titles: Titles,

  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")] pub actors: Vec<Actor>,

  #[serde(skip_serializing_if = "TrackIdentity::is_empty", default = "TrackIdentity::default")]
  pub identity: TrackIdentity,

  #[serde(skip_serializing_if = "AlbumMetadata::is_empty", default = "AlbumMetadata::default")]
  pub album: AlbumMetadata,

  #[serde(skip_serializing_if = "ReleaseMetadata::is_empty", default = "ReleaseMetadata::default")]
  pub release: ReleaseMetadata,

  #[serde(skip_serializing_if = "Option::is_none")] pub track_numbers: Option<TrackNumbers>,

  #[serde(skip_serializing_if = "Option::is_none")] pub disc_numbers: Option<DiscNumbers>,

  #[serde(skip_serializing_if = "MusicMetadata::is_empty", default = "MusicMetadata::default")]
  pub music: MusicMetadata,

  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")] pub tags: Vec<Tag>, // no duplicate terms per facet allowed

  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")]
  pub ratings: Vec<Rating>, // no duplicate owners allowed

  #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::default")]
  pub comments: Vec<Comment>, // no duplicate owners allowed
}

impl TrackMetadata {
  pub fn is_valid(&self) -> bool {
    self.media.is_valid() && self.audio.is_valid() && self.titles.is_valid()
      && (self.album.is_empty() || self.album.titles.is_valid())
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
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use serde_json;
  use mime_guess;

  #[test]
  fn serialize_json() {
    let resource = MediaResource {
      uri: "subfolder/test.mp3".to_string(),
      collection_uid: "globallyuniquecollectionidentifier".to_string(),
      metadata_imported: Some(Utc::now()),
      metadata_exported: None,
    };
    let media = MediaMetadata {
      content_type: Some(mime_guess::guess_mime_type(&resource.uri).to_string()),
      resources: vec![resource],
    };
    let classifications = vec![
      Classification::new(Classifier::Energy, 0.1),
      Classification::new(Classifier::Popularity, 0.9),
    ];
    let music = MusicMetadata {
      classifications,
      loudness: Some(Loudness::EBUR128LUFS(LUFS { db: -2.3 })),
      ..Default::default()
    };
    let tags = vec![
      Tag::new_faceted("Decade", "1980s", 0.8),
      Tag::new_faceted("DECADE", "1990s", 0.3),
      Tag::new("Floorfiller", 1.0),
    ];
    let comments = vec![
      Comment::new_anonymous("Some anonymous notes about this track"),
    ];
    let track = TrackMetadata {
      media,
      music,
      tags,
      comments,
      ..Default::default()
    };
    let track_json = serde_json::to_string(&track).unwrap();
    assert_ne!("{}", track_json);
    println!("Track (JSON): {}", track_json);
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
          Rating::new_anonymous(Rating::rating_from_stars(stars, max_stars)).star_rating(max_stars)
        );
      }
    }
  }
}
