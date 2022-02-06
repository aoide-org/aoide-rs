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
pub static FACET_ISRC: Lazy<FacetId> = Lazy::new(|| FacetId::new("isrc".into()));

// The Grouping aka Content Group field
pub static FACET_GROUPING: Lazy<FacetId> = Lazy::new(|| FacetId::new("cgrp".into()));

// Comment
pub static FACET_COMMENT: Lazy<FacetId> = Lazy::new(|| FacetId::new("comm".into()));

// Description
pub static FACET_DESCRIPTION: Lazy<FacetId> = Lazy::new(|| FacetId::new("desc".into()));

// ISO 639-3 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
pub static FACET_LANGUAGE: Lazy<FacetId> = Lazy::new(|| FacetId::new("lang".into()));

// "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
pub static FACET_GENRE: Lazy<FacetId> = Lazy::new(|| FacetId::new("genre".into()));

// Personal mental or emotional state, e.g. "happy", "sexy", "sad", "melancholic", "joyful", ...
pub static FACET_MOOD: Lazy<FacetId> = Lazy::new(|| FacetId::new("mood".into()));

// Custom: Decades like "1980s", "2000s", ..., or other time-based properties
pub static FACET_DECADE: Lazy<FacetId> = Lazy::new(|| FacetId::new("decade".into()));

// Custom: Sub-genres or details like "East Coast", "West Coast", ...
pub static FACET_STYLE: Lazy<FacetId> = Lazy::new(|| FacetId::new("style".into()));

// Custom: Atmosphere of the situation, e.g. "bouncy", "driving", "dreamy", "poppy", "punchy", "spiritual", "tropical", "uplifting" ...
pub static FACET_VIBE: Lazy<FacetId> = Lazy::new(|| FacetId::new("vibe".into()));

// Predefined musical or audio feature scores (as of Spotify/EchoNest).
// A label is optional and could be used for identifying the source of
// the score.
//
// The combination of FACET_AROUSAL and FACET_VALENCE could
// be used for classifying emotion (= mood) according to Thayer's
// arousel-valence emotion plane.
//
// See also: [Spotify Audio Features](https://developer.spotify.com/documentation/web-api/reference/tracks/get-audio-features/)
pub static FACET_ACOUSTICNESS: Lazy<FacetId> = Lazy::new(|| FacetId::new("acousticness".into()));
pub static FACET_AROUSAL: Lazy<FacetId> = Lazy::new(|| FacetId::new("arousal".into()));
pub static FACET_DANCEABILITY: Lazy<FacetId> = Lazy::new(|| FacetId::new("danceability".into()));
pub static FACET_ENERGY: Lazy<FacetId> = Lazy::new(|| FacetId::new("energy".into()));
pub static FACET_INSTRUMENTALNESS: Lazy<FacetId> =
    Lazy::new(|| FacetId::new("instrumentalness".into()));
pub static FACET_LIVENESS: Lazy<FacetId> = Lazy::new(|| FacetId::new("liveness".into()));
pub static FACET_POPULARITY: Lazy<FacetId> = Lazy::new(|| FacetId::new("popularity".into()));
pub static FACET_SPEECHINESS: Lazy<FacetId> = Lazy::new(|| FacetId::new("speechiness".into()));
pub static FACET_VALENCE: Lazy<FacetId> = Lazy::new(|| FacetId::new("valence".into())); // a measure for happiness

// Vendor-supplied, globally unique identifier(s) used by iTunes
// Format: prefix:scheme:identifier
// Supported schemes: upc, isrc, isan, grid, uuid, vendor_id
// Example: "SonyBMG:isrc:USRC10900295"
// See also: https://www.apple.com/au/itunes/lp-and-extras/docs/Development_Guide.pdf
pub static FACET_XID: Lazy<FacetId> = Lazy::new(|| FacetId::new("xid".into()));
