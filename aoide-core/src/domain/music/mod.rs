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

use domain::metadata::Confidence;

///////////////////////////////////////////////////////////////////////
/// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TitleLevel {
    Main,
    Sub,
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
    #[serde(skip_serializing_if = "TitleLevel::is_default", default)]
    pub level: TitleLevel,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    pub name: String,
}

impl Title {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }

    pub fn is_main_level(&self) -> bool {
        self.level == TitleLevel::Main
    }

    pub fn is_language(&self, language: &str) -> bool {
        self.language.as_ref().map(|language| language.as_str()) == Some(language)
    }
}

pub struct Titles;

impl Titles {
    pub fn is_valid(titles: &[Title]) -> bool {
        Self::has_main_title_without_language(titles) && titles.iter().all(Title::is_valid)
    }

    pub fn main_title_with_language<'a>(titles: &'a [Title], language: &str) -> Option<&'a Title> {
        debug_assert!(titles
            .iter()
            .filter(|title| title.is_main_level() && title.is_language(language))
            .count() <= 1);
        titles
            .iter()
            .filter(|title| title.is_main_level() && title.is_language(language))
            .nth(0)
    }

    pub fn main_title_without_language<'a>(titles: &'a [Title]) -> Option<&'a Title> {
        debug_assert!(titles
            .iter()
            .filter(|title| title.is_main_level() && title.language == None)
            .count() <= 1);
        titles
            .iter()
            .filter(|title| title.is_main_level() && title.language == None)
            .nth(0)
    }

    pub fn has_main_title_without_language<'a>(titles: &'a [Title]) -> bool {
        if let Some(_) = Self::main_title_without_language(titles) {
            true
        } else {
            false
        }
    }

    pub fn main_title<'a>(titles: &'a [Title], language: &str) -> Option<&'a Title> {
        Self::main_title_with_language(titles, language)
            .or_else(|| Self::main_title_without_language(titles))
    }
}

///////////////////////////////////////////////////////////////////////
/// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum ActorRole {
    Artist,
    Arranger,
    Composer,
    Conductor,
    DjMixer,
    Engineer,
    Lyricist,
    Mixer,
    Performer,
    Producer,
    Publisher,
    Remixer,
    Writer,
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
    Summary = 0,
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
    pub role: ActorRole,

    #[serde(skip_serializing_if = "ActorPriority::is_default", default)]
    pub prio: ActorPriority,

    pub name: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub refs: Vec<String>, // external URIs
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
        // - at most one primary entry exists for each role
        // - at least one summary or primary entry exists for each role
    }

    pub fn summary_actor<'a>(actors: &'a [Actor], role: ActorRole) -> Option<&'a Actor> {
        debug_assert!(actors
            .iter()
            .filter(|actor| actor.role == role && actor.prio == ActorPriority::Summary)
            .count() <= 1);
        actors
            .iter()
            .filter(|actor| actor.role == role && actor.prio == ActorPriority::Summary)
            .nth(0)
    }

    pub fn primary_actor<'a>(actors: &'a [Actor], role: ActorRole) -> Option<&'a Actor> {
        debug_assert!(actors
            .iter()
            .filter(|actor| actor.role == role && actor.prio == ActorPriority::Primary)
            .count() <= 1);
        actors
            .iter()
            .filter(|actor| actor.role == role && actor.prio == ActorPriority::Primary)
            .nth(0)
    }

    pub fn main_actor<'a>(actors: &'a [Actor], role: ActorRole) -> Option<&'a Actor> {
        Self::summary_actor(actors, role).or_else(|| Self::primary_actor(actors, role))
    }
}

///////////////////////////////////////////////////////////////////////
/// Classification
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Classifier {
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
    pub classifier: Classifier,
    pub confidence: Confidence,
}

impl Classification {
    pub fn new<C: Into<Confidence>>(classifier: Classifier, confidence: C) -> Self {
        Self {
            classifier,
            confidence: confidence.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.confidence.is_valid()
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actors() {
        let main_artist_name = "Madonna feat. M.I.A. and Nicki Minaj";
        let default_producer_name = "Martin Solveig";
        let actors = vec![
            Actor {
                role: ActorRole::Artist,
                prio: ActorPriority::Summary,
                name: main_artist_name.into(),
                ..Default::default()
            },
            Actor {
                role: ActorRole::Artist,
                prio: ActorPriority::Primary,
                name: "Madonna".into(),
                ..Default::default()
            },
            Actor {
                role: ActorRole::Artist,
                prio: ActorPriority::Secondary,
                name: "M.I.A.".into(),
                ..Default::default()
            },
            Actor {
                role: ActorRole::Producer,
                prio: ActorPriority::Primary,
                name: default_producer_name.into(),
                ..Default::default()
            },
            Actor {
                role: ActorRole::Artist,
                prio: ActorPriority::Secondary,
                name: "Nicki Minaj".into(),
                ..Default::default()
            },
        ];
        assert!(Actors::is_valid(&actors));
        assert_eq!(
            main_artist_name,
            Actors::main_actor(&actors, ActorRole::Artist).unwrap().name
        );
        assert_eq!(
            default_producer_name,
            Actors::main_actor(&actors, ActorRole::Producer).unwrap().name
        );
        assert_eq!(None, Actors::main_actor(&actors, ActorRole::Conductor));
    }
}
