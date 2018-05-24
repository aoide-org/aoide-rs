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

use domain::metadata::Score;

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

    pub name: String,

    #[serde(rename = "lang", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
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
    #[serde(rename = "prio", skip_serializing_if = "ActorPriority::is_default", default)]
    pub priority: ActorPriority,

    pub role: ActorRole,

    pub name: String,

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
    }

    pub fn summary_actor<'a>(actors: &'a [Actor], role: ActorRole) -> Option<&'a Actor> {
        debug_assert!(actors
            .iter()
            .filter(|actor| actor.priority == ActorPriority::Summary && actor.role == role)
            .count() <= 1);
        actors
            .iter()
            .filter(|actor| actor.priority == ActorPriority::Summary && actor.role == role)
            .nth(0)
    }

    // The summary actor or otherwise the singular primary actor
    pub fn main_actor<'a>(actors: &'a [Actor], role: ActorRole) -> Option<&'a Actor> {
        debug_assert!(Self::summary_actor(actors, role).is_some() || actors
            .iter()
            .filter(|actor| actor.priority == ActorPriority::Primary && actor.role == role)
            .count() <= 1);
        Self::summary_actor(actors, role).or_else(|| actors
            .iter()
            .filter(|actor| actor.priority == ActorPriority::Primary && actor.role == role)
            .nth(0))
    }
}

///////////////////////////////////////////////////////////////////////
/// Classification
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum ClassificationSubject {
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
    pub subject: ClassificationSubject,
    pub score: Score,
}

impl Classification {
    pub fn new<C: Into<Score>>(subject: ClassificationSubject, score: C) -> Self {
        Self {
            subject,
            score: score.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.score.is_valid()
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
        let summary_artist_name = "Madonna feat. M.I.A. and Nicki Minaj";
        let primary_producer_name = "Martin Solveig";
        let actors = vec![
            Actor {
                priority: ActorPriority::Summary,
                role: ActorRole::Artist,
                name: summary_artist_name.into(),
                ..Default::default()
            },
            Actor {
                priority: ActorPriority::Primary,
                role: ActorRole::Artist,
                name: "Madonna".into(),
                ..Default::default()
            },
            Actor {
                priority: ActorPriority::Secondary,
                role: ActorRole::Artist,
                name: "M.I.A.".into(),
                ..Default::default()
            },
            Actor {
                priority: ActorPriority::Primary,
                role: ActorRole::Producer,
                name: primary_producer_name.into(),
                ..Default::default()
            },
            Actor {
                priority: ActorPriority::Secondary,
                role: ActorRole::Artist,
                name: "Nicki Minaj".into(),
                ..Default::default()
            },
        ];
        assert!(Actors::is_valid(&actors));
        assert_eq!(
            summary_artist_name,
            Actors::summary_actor(&actors, ActorRole::Artist).unwrap().name
        );
        assert_eq!(
            summary_artist_name,
            Actors::main_actor(&actors, ActorRole::Artist).unwrap().name
        );
        assert_eq!(
            None,
            Actors::summary_actor(&actors, ActorRole::Producer)
        );
        assert_eq!(
            primary_producer_name,
            Actors::main_actor(&actors, ActorRole::Producer).unwrap().name
        );
        assert_eq!(None, Actors::summary_actor(&actors, ActorRole::Conductor));
        assert_eq!(None, Actors::main_actor(&actors, ActorRole::Conductor));
    }
}
