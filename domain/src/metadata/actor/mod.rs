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

use serde_repr::*;

///////////////////////////////////////////////////////////////////////
// ActorRole
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

impl Default for ActorRole {
    fn default() -> ActorRole {
        ActorRole::Artist
    }
}

///////////////////////////////////////////////////////////////////////
// ActorPrecedence
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
#[repr(u8)]
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
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    #[validate(length(min = 1))]
    #[serde(rename = "n")]
    pub name: String,

    #[serde(rename = "r", skip_serializing_if = "IsDefault::is_default", default)]
    pub role: ActorRole,

    #[serde(rename = "p", skip_serializing_if = "IsDefault::is_default", default)]
    pub precedence: ActorPrecedence,
}

#[derive(Debug, Clone, Copy)]
pub struct Actors;

impl Actors {
    // TODO: Validate that
    // - at most one summary entry exists for each role
    // - at least one summary entry exists if more than one primary entry exists for disambiguation
    pub fn validate_main_actor(actors: &[Actor]) -> Result<(), ValidationError> {
        if !actors.is_empty() && Self::main_actor(actors, ActorRole::Artist).is_none() {
            return Err(ValidationError::new("missing main actor"));
        }
        Ok(())
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

#[cfg(test)]
mod tests;
