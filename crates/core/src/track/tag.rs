// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use crate::tag::FacetId;

// Some predefined facets that are commonly used and could serve as
// a starting point for complex tagging schemes
//
// <https://picard-docs.musicbrainz.org/en/variables/variables.html>
// <https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html>

// International Standard Recording Code (ISRC, ISO 3901)
// ID3v2.4: TSRC
// Vorbis:  ISRC
// MP4:     isrc
pub const FACET_ISRC: &str = "isrc";
pub const FACET_ID_ISRC: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_ISRC));

// The Grouping aka Content Group field
// ID3v2.4: GRP1 (iTunes/newer) / TIT1 (traditional/older)
// Vorbis:  GROUPING
// MP4:     ©grp
pub const FACET_GROUPING: &str = "cgrp";
pub const FACET_ID_GROUPING: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_GROUPING));

// Comment
// ID3v2.4: COMM (without `description`)
// Vorbis:  COMMENT
// MP4:     ©cmt
pub const FACET_COMMENT: &str = "comm";
pub const FACET_ID_COMMENT: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_COMMENT));

// Description
// ID3v2.4: COMM:description
// Vorbis:  DESCRIPTION
// MP4:     desc
pub const FACET_DESCRIPTION: &str = "desc";
pub const FACET_ID_DESCRIPTION: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_DESCRIPTION));

// ISO 639-3 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
// ID3v2.4: TLAN
// Vorbis:  LANGUAGE
// MP4:     ----:com.apple.iTunes:LANGUAGE
pub const FACET_LANGUAGE: &str = "lang";
pub const FACET_ID_LANGUAGE: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_LANGUAGE));

// "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
// ID3v2.4: TCON
// Vorbis:  GENRE
// MP4:     ©gen
pub const FACET_GENRE: &str = "genre";
pub const FACET_ID_GENRE: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_GENRE));

// Personal mental or emotional state, e.g. "happy", "sexy", "sad", "melancholic", "joyful", ...
// ID3v2.4: TMOO
// Vorbis:  MOOD
// MP4:     ----:com.apple.iTunes:MOOD
pub const FACET_MOOD: &str = "mood";
pub const FACET_ID_MOOD: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_MOOD));

// Custom: Decades like "1980s", "2000s", ..., or other time-based properties
pub const FACET_DECADE: &str = "decade";
pub const FACET_ID_DECADE: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_DECADE));

// Custom: Sub-genres or details like "East Coast", "West Coast", ...
pub const FACET_STYLE: &str = "style";
pub const FACET_ID_STYLE: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_STYLE));

// Custom: Atmosphere of the situation, e.g. "bouncy", "driving", "dreamy", "poppy", "punchy", "spiritual", "tropical", "uplifting" ...
pub const FACET_VIBE: &str = "vibe";
pub const FACET_ID_VIBE: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_VIBE));

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
pub const FACET_ID_ACOUSTICNESS: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_ACOUSTICNESS));

pub const FACET_AROUSAL: &str = "arousal";
pub const FACET_ID_AROUSAL: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_AROUSAL));

pub const FACET_DANCEABILITY: &str = "danceability";
pub const FACET_ID_DANCEABILITY: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_DANCEABILITY));

pub const FACET_ENERGY: &str = "energy";
pub const FACET_ID_ENERGY: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_ENERGY));

pub const FACET_INSTRUMENTALNESS: &str = "instrumentalness";
pub const FACET_ID_INSTRUMENTALNESS: FacetId<'_> =
    FacetId::new(Cow::Borrowed(FACET_INSTRUMENTALNESS));

pub const FACET_LIVENESS: &str = "liveness";
pub const FACET_ID_LIVENESS: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_LIVENESS));

pub const FACET_POPULARITY: &str = "popularity";
pub const FACET_ID_POPULARITY: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_POPULARITY));

pub const FACET_SPEECHINESS: &str = "speechiness";
pub const FACET_ID_SPEECHINESS: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_SPEECHINESS));

pub const FACET_VALENCE: &str = "valence";
pub const FACET_ID_VALENCE: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_VALENCE));

// Vendor-supplied, globally unique identifier(s) used by iTunes
// Format: prefix:scheme:identifier
// Supported schemes: upc, isrc, isan, grid, uuid, vendor_id
// Example: "SonyBMG:isrc:USRC10900295"
// See also: https://www.apple.com/au/itunes/lp-and-extras/docs/Development_Guide.pdf
pub const FACET_XID: &str = "valence";
pub const FACET_ID_XID: FacetId<'_> = FacetId::new(Cow::Borrowed(FACET_XID));
