// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

use crate::tag::{Facet, Label};

use lazy_static::lazy_static;

// Some predefined facets that are commonly used and could serve as
// a starting point for complex tagging schemes
lazy_static! {
    // ISRC
    pub static ref FACET_ISRC: Facet = Facet::new("isrc".into());

    // The Content Group aka Grouping field
    pub static ref FACET_CGROUP: Facet = Facet::new("cgroup".into());

    // Description or comment
    pub static ref FACET_COMMENT: Facet = Facet::new("comment".into());

    // Keywords
    pub static ref FACET_KEYWORD: Facet = Facet::new("keyword".into());

    // Decades like "1980s", "2000s", ..., or other time-based properties
    pub static ref FACET_DECADE: Facet = Facet::new("decade".into());

    // "Pop", "Dance", "Electronic", "R&B/Soul", "Hip Hop/Rap", ...
    pub static ref FACET_GENRE: Facet = Facet::new("genre".into());

    // ISO 639-2 language codes: "eng", "fre"/"fra", "ita", "spa", "ger"/"deu", ...
    pub static ref FACET_LANG: Facet = Facet::new("lang".into());

    // Personal mental or emotional state, e.g. "happy", "sexy", "sad", "melancholic", "joyful", ...
    pub static ref FACET_MOOD: Facet = Facet::new("mood".into());

    // Sub-genres or details like "East Coast", "West Coast", ...
    pub static ref FACET_STYLE: Facet = Facet::new("style".into());

    // Atmosphere of the situation, e.g. "bouncy", "driving", "dreamy", "poppy", "punchy", "spiritual", "tropical", "uplifting" ...
    pub static ref FACET_VIBE: Facet = Facet::new("vibe".into());

    // Predefined musical or audio feature scores (as of Spotify/EchoNest).
    // A label is optional and could be used for identifying the source of
    // the score.
    //
    // The combination of FACET_AROUSAL and FACET_VALENCE could
    // be used for classifying emotion (= mood) according to Thayer's
    // arousel-valence emotion plane.
    //
    // See also: [Spotify Audio Features](https://developer.spotify.com/documentation/web-api/reference/tracks/get-audio-features/)
    pub static ref FACET_ACOUSTICNESS: Label = Label::new("acousticness".into());
    pub static ref FACET_AROUSAL: Label = Label::new("arousal".into());
    pub static ref FACET_DANCEABILITY: Label = Label::new("danceability".into());
    pub static ref FACET_ENERGY: Label = Label::new("energy".into());
    pub static ref FACET_INSTRUMENTALNESS: Label = Label::new("instrumentalness".into());
    pub static ref FACET_LIVENESS: Label = Label::new("liveness".into());
    pub static ref FACET_POPULARITY: Label = Label::new("popularity".into());
    pub static ref FACET_SPEECHINESS: Label = Label::new("speechiness".into());
    pub static ref FACET_VALENCE: Label = Label::new("valence".into()); // a measure for happiness
}
