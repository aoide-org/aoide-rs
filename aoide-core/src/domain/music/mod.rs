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

pub mod sonic;

#[cfg(test)]
mod tests;

use domain::metadata::Score;

///////////////////////////////////////////////////////////////////////
/// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
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

impl TitleLevel {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

///////////////////////////////////////////////////////////////////////
/// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Title {
    pub name: String,

    #[serde(skip_serializing_if = "TitleLevel::is_default", default)]
    pub level: TitleLevel,

    #[serde(rename = "lang", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl Title {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

pub struct Titles;

impl Titles {
    pub fn is_valid(titles: &[Title]) -> bool {
        Self::main_title(titles).is_some() && titles.iter().all(Title::is_valid)
    }

    pub fn title<'a>(titles: &'a [Title], level: TitleLevel, language: Option<&str>) -> Option<&'a Title> {
        debug_assert!(titles
            .iter()
            .filter(|title| title.level == level && title.language.as_ref().map(|v| v.as_str()) == language)
            .count() <= 1);
        titles
            .iter()
            .filter(|title| title.level == level && title.language.as_ref().map(|v| v.as_str()) == language)
            .nth(0)
    }

    pub fn main_title<'a>(titles: &'a [Title]) -> Option<&'a Title> {
        Self::title(titles, TitleLevel::Main, None)
    }
}

///////////////////////////////////////////////////////////////////////
/// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
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

impl ActorRole {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for ActorRole {
    fn default() -> ActorRole {
        ActorRole::Artist
    }
}

///////////////////////////////////////////////////////////////////////
/// ActorPriority
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum ActorPriority {
    Summary = 0, // default
    Primary = 1,
    Secondary = 2,
}

impl ActorPriority {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for ActorPriority {
    fn default() -> ActorPriority {
        ActorPriority::Summary
    }
}

///////////////////////////////////////////////////////////////////////
/// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Actor {
    pub name: String,

    #[serde(skip_serializing_if = "ActorRole::is_default", default)]
    pub role: ActorRole,

    #[serde(rename = "prio", skip_serializing_if = "ActorPriority::is_default", default)]
    pub priority: ActorPriority,

    #[serde(rename = "refs", skip_serializing_if = "Vec::is_empty", default)]
    pub references: Vec<String>, // external URIs
}

impl Actor {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

pub struct Actors;

impl Actors {
    pub fn is_valid(actors: &[Actor]) -> bool {
        actors.iter().all(|actor| actor.is_valid())
        // TODO:
        // - at most one summary entry exists for each role
        // - at least one summary entry exists if more than one primary entry exists for disambiguation
    }

    pub fn actor<'a>(actors: &'a [Actor], role: ActorRole, priority: ActorPriority) -> Option<&'a Actor> {
        debug_assert!(actors
            .iter()
            .filter(|actor| actor.role == role && actor.priority == priority)
            .count() <= 1);
        actors
            .iter()
            .filter(|actor| actor.role == role && actor.priority == priority)
            .nth(0)
    }

    // The singular summary actor or if none exists then the singular primary actor
    pub fn main_actor<'a>(actors: &'a [Actor], role: ActorRole) -> Option<&'a Actor> {
        Self::actor(actors, role, ActorPriority::Summary).or_else(
            || Self::actor(actors, role, ActorPriority::Primary))
    }
}

///////////////////////////////////////////////////////////////////////
/// Classification
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Class {
    Acousticness,
    Danceability,
    Energy,
    Instrumentalness,
    Liveness,
    Popularity,
    Speechiness,
    Valence,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Classification {
    pub class: Class,
    pub score: Score,
}

impl Classification {
    pub fn new<C: Into<Score>>(class: Class, score: C) -> Self {
        Self {
            class,
            score: score.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.score.is_valid()
    }
}
