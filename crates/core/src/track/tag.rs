// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use once_cell::sync::Lazy;

use crate::tag::FacetId;

// Some predefined facets that are commonly used and could serve as
// a starting point for complex tagging schemes
//
// https://picard-docs.musicbrainz.org/en/variables/variables.html
// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

// International Standard Recording Code (ISRC, ISO 3901)
// ID3v2.4: TSRC
// Vorbis:  ISRC
// MP4:     isrc
pub static FACET_ID_ISRC: Lazy<FacetId> = Lazy::new(|| FacetId::new("isrc".into()));

// The Grouping aka Content Group field
// ID3v2.4: GRP1 (iTunes/newer) / TIT1 (traditional/older)
// Vorbis:  GROUPING
// MP4:     ©grp
pub const FACET_GROUPING: &str = "cgrp";
pub static FACET_ID_GROUPING: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_GROUPING.to_owned()));

// Comment
// ID3v2.4: COMM (without `description`)
// Vorbis:  COMMENT
// MP4:     ©cmt
pub const FACET_COMMENT: &str = "comm";
pub static FACET_ID_COMMENT: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_COMMENT.to_owned()));

// Description
// ID3v2.4: COMM:description
// Vorbis:  DESCRIPTION
// MP4:     desc
pub static FACET_ID_DESCRIPTION: Lazy<FacetId> = Lazy::new(|| FacetId::new("desc".into()));

// ISO 639-3 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
// ID3v2.4: TLAN
// Vorbis:  LANGUAGE
// MP4:     ----:com.apple.iTunes:LANGUAGE
pub static FACET_ID_LANGUAGE: Lazy<FacetId> = Lazy::new(|| FacetId::new("lang".into()));

// "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
// ID3v2.4: TCON
// Vorbis:  GENRE
// MP4:     ©gen
pub const FACET_GENRE: &str = "genre";
pub static FACET_ID_GENRE: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_GENRE.to_owned()));

// Personal mental or emotional state, e.g. "happy", "sexy", "sad", "melancholic", "joyful", ...
// ID3v2.4: TMOO
// Vorbis:  MOOD
// MP4:     ----:com.apple.iTunes:MOOD
pub const FACET_MOOD: &str = "mood";
pub static FACET_ID_MOOD: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_MOOD.to_owned()));

// Custom: Decades like "1980s", "2000s", ..., or other time-based properties
pub static FACET_ID_DECADE: Lazy<FacetId> = Lazy::new(|| FacetId::new("decade".into()));

// Custom: Sub-genres or details like "East Coast", "West Coast", ...
pub static FACET_ID_STYLE: Lazy<FacetId> = Lazy::new(|| FacetId::new("style".into()));

// Custom: Atmosphere of the situation, e.g. "bouncy", "driving", "dreamy", "poppy", "punchy", "spiritual", "tropical", "uplifting" ...
pub static FACET_ID_VIBE: Lazy<FacetId> = Lazy::new(|| FacetId::new("vibe".into()));

// Predefined musical or audio feature scores (as of Spotify/EchoNest).
// A label is optional and could be used for identifying the source of
// the score.
//
// The combination of FACET_AROUSAL and FACET_VALENCE could
// be used for classifying emotion (= mood) according to Thayer's
// arousel-valence emotion plane.
//
// See also: [Spotify Audio Features](https://developer.spotify.com/documentation/web-api/reference/tracks/get-audio-features/)

pub const FACET_ACOUSTICNESS: &str = "acousticness";
pub static FACET_ID_ACOUSTICNESS: Lazy<FacetId> =
    Lazy::new(|| FacetId::new(FACET_ACOUSTICNESS.to_owned()));

pub const FACET_AROUSAL: &str = "arousal";
pub static FACET_ID_AROUSAL: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_AROUSAL.to_owned()));

pub const FACET_DANCEABILITY: &str = "danceability";
pub static FACET_ID_DANCEABILITY: Lazy<FacetId> =
    Lazy::new(|| FacetId::new(FACET_DANCEABILITY.to_owned()));

pub const FACET_ENERGY: &str = "energy";
pub static FACET_ID_ENERGY: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_ENERGY.to_owned()));

pub const FACET_INSTRUMENTALNESS: &str = "instrumentalness";
pub static FACET_ID_INSTRUMENTALNESS: Lazy<FacetId> =
    Lazy::new(|| FacetId::new(FACET_INSTRUMENTALNESS.to_owned()));

pub const FACET_LIVENESS: &str = "liveness";
pub static FACET_ID_LIVENESS: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_LIVENESS.to_owned()));

pub const FACET_POPULARITY: &str = "popularity";
pub static FACET_ID_POPULARITY: Lazy<FacetId> =
    Lazy::new(|| FacetId::new(FACET_POPULARITY.to_owned()));

pub const FACET_SPEECHINESS: &str = "speechiness";
pub static FACET_ID_SPEECHINESS: Lazy<FacetId> =
    Lazy::new(|| FacetId::new(FACET_SPEECHINESS.to_owned()));

pub const FACET_VALENCE: &str = "valence";
pub static FACET_ID_VALENCE: Lazy<FacetId> = Lazy::new(|| FacetId::new(FACET_VALENCE.to_owned()));

// Vendor-supplied, globally unique identifier(s) used by iTunes
// Format: prefix:scheme:identifier
// Supported schemes: upc, isrc, isan, grid, uuid, vendor_id
// Example: "SonyBMG:isrc:USRC10900295"
// See also: https://www.apple.com/au/itunes/lp-and-extras/docs/Development_Guide.pdf
pub static FACET_ID_XID: Lazy<FacetId> = Lazy::new(|| FacetId::new("xid".into()));
