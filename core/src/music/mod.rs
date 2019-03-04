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

use crate::metadata::Score;

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////
pub mod notation;

use self::notation::*;

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////
/// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TitleLevel {
    Main = 0, // default
    Sub = 1,
    // for classical music, only used for tracks not albums
    #[serde(rename = "wrk")]
    Work = 2,
    #[serde(rename = "mvn")]
    Movement = 3,
}

impl Default for TitleLevel {
    fn default() -> TitleLevel {
        TitleLevel::Main
    }
}

///////////////////////////////////////////////////////////////////////
/// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Title {
    pub name: String,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub level: TitleLevel,

    #[serde(rename = "lang", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl IsValid for Title {
    fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Titles;

impl Titles {
    pub fn is_valid(titles: &[Title]) -> bool {
        Self::main_title(titles).is_some() && titles.iter().all(Title::is_valid)
    }

    pub fn title<'a>(
        titles: &'a [Title],
        level: TitleLevel,
        language: Option<&str>,
    ) -> Option<&'a Title> {
        debug_assert!(
            titles
                .iter()
                .filter(|title| title.level == level
                    && title.language.as_ref().map(|v| v.as_str()) == language)
                .count()
                <= 1
        );
        titles
            .iter()
            .filter(|title| {
                title.level == level && title.language.as_ref().map(|v| v.as_str()) == language
            })
            .nth(0)
    }

    pub fn main_title(titles: &[Title]) -> Option<&Title> {
        Self::title(titles, TitleLevel::Main, None)
    }
}

///////////////////////////////////////////////////////////////////////
/// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ActorRole {
    Artist = 0, // default
    Arranger = 1,
    Composer = 2,
    Conductor = 3,
    DjMixer = 4,
    Engineer = 5,
    Lyricist = 6,
    Mixer = 7,
    Performer = 8,
    Producer = 9,
    Publisher = 10,
    Remixer = 11,
    Writer = 12,
}

impl Default for ActorRole {
    fn default() -> ActorRole {
        ActorRole::Artist
    }
}

///////////////////////////////////////////////////////////////////////
/// ActorPrecedence
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ActorPrecedence {
    Summary = 0, // default
    Primary = 1,
    Secondary = 2,
}

impl Default for ActorPrecedence {
    fn default() -> ActorPrecedence {
        ActorPrecedence::Summary
    }
}

///////////////////////////////////////////////////////////////////////
/// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Actor {
    pub name: String,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub role: ActorRole,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub precedence: ActorPrecedence,

    #[serde(rename = "xrefs", skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<String>,
}

impl IsValid for Actor {
    fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Actors;

impl Actors {
    pub fn is_valid(actors: &[Actor]) -> bool {
        actors.iter().all(|actor| actor.is_valid())
        // TODO:
        // - at most one summary entry exists for each role
        // - at least one summary entry exists if more than one primary entry exists for disambiguation
    }

    pub fn actor(actors: &[Actor], role: ActorRole, precedence: ActorPrecedence) -> Option<&Actor> {
        debug_assert!(
            actors
                .iter()
                .filter(|actor| actor.role == role && actor.precedence == precedence)
                .count()
                <= 1
        );
        actors
            .iter()
            .filter(|actor| actor.role == role && actor.precedence == precedence)
            .nth(0)
    }

    // The singular summary actor or if none exists then the singular primary actor
    pub fn main_actor(actors: &[Actor], role: ActorRole) -> Option<&Actor> {
        Self::actor(actors, role, ActorPrecedence::Summary)
            .or_else(|| Self::actor(actors, role, ActorPrecedence::Primary))
    }
}

///////////////////////////////////////////////////////////////////////
/// Lyrics
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Lyrics {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub text: String,

    #[serde(rename = "lang", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit: Option<bool>,
}

impl IsValid for Lyrics {
    fn is_valid(&self) -> bool {
        true
    }
}

///////////////////////////////////////////////////////////////////////
/// Song Features & Classification
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum SongFeature {
    Acousticness,
    Danceability,
    Energy,
    Instrumentalness,
    Liveness,
    Popularity,
    Speechiness,
    Valence,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoredSongFeature(Score, SongFeature);

impl ScoredSongFeature {
    pub fn new<S: Into<Score>>(score: S, feature: SongFeature) -> Self {
        ScoredSongFeature(score.into(), feature)
    }

    pub fn score(&self) -> Score {
        self.0
    }

    pub fn feature(&self) -> SongFeature {
        self.1
    }
}

impl IsValid for ScoredSongFeature {
    fn is_valid(&self) -> bool {
        self.score().is_valid()
    }
}

///////////////////////////////////////////////////////////////////////
/// SongProfile
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SongProfile {
    #[serde(
        rename = "tempoBpm",
        skip_serializing_if = "IsDefault::is_default",
        default
    )]
    pub tempo: TempoBpm,

    #[serde(
        rename = "timeSig",
        skip_serializing_if = "IsDefault::is_default",
        default
    )]
    pub time_sig: TimeSignature,

    #[serde(
        rename = "keySig",
        skip_serializing_if = "IsDefault::is_default",
        default
    )]
    pub key_sig: KeySignature,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub features: Vec<ScoredSongFeature>, // no duplicate features allowed
}

impl SongProfile {
    pub fn has_feature(&self, feature: SongFeature) -> bool {
        self.features
            .iter()
            .any(|feature_score| feature_score.feature() == feature)
    }

    fn is_feature_unique(&self, feature: SongFeature) -> bool {
        self.features
            .iter()
            .filter(|feature_score| feature_score.feature() == feature)
            .count()
            <= 1
    }

    pub fn feature(&self, feature: SongFeature) -> Option<&ScoredSongFeature> {
        debug_assert!(self.is_feature_unique(feature));
        self.features
            .iter()
            .filter(|feature_score| feature_score.feature() == feature)
            .nth(0)
    }
}

impl IsValid for SongProfile {
    fn is_valid(&self) -> bool {
        (self.tempo.is_default() || self.tempo.is_valid())
            && (self.time_sig.is_valid() || self.time_sig.is_default())
            && (self.key_sig.is_valid() || self.key_sig.is_default())
            && self.features.iter().all(ScoredSongFeature::is_valid)
            && self.features.iter().all(|feature_score| {
                feature_score.is_valid() && self.is_feature_unique(feature_score.feature())
            })
    }
}
