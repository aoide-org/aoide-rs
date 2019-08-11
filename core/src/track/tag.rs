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

///////////////////////////////////////////////////////////////////////

use crate::tag::{Facet, Label};

use lazy_static::lazy_static;

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
