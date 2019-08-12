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

use aoide_core::{
    actor::{
        Actor as CoreActor, ActorPrecedence as CoreActorPrecedence, ActorRole as CoreActorRole,
    },
    util::IsDefault,
};

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
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

impl From<ActorRole> for CoreActorRole {
    fn from(from: ActorRole) -> Self {
        use CoreActorRole::*;
        match from {
            ActorRole::Artist => Artist,
            ActorRole::Arranger => Arranger,
            ActorRole::Composer => Composer,
            ActorRole::Conductor => Conductor,
            ActorRole::DjMixer => DjMixer,
            ActorRole::Engineer => Engineer,
            ActorRole::Lyricist => Lyricist,
            ActorRole::Mixer => Mixer,
            ActorRole::Performer => Performer,
            ActorRole::Producer => Producer,
            ActorRole::Publisher => Publisher,
            ActorRole::Remixer => Remixer,
            ActorRole::Writer => Writer,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// ActorPrecedence
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
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

impl From<ActorPrecedence> for CoreActorPrecedence {
    fn from(from: ActorPrecedence) -> Self {
        use CoreActorPrecedence::*;
        match from {
            ActorPrecedence::Summary => Summary,
            ActorPrecedence::Primary => Primary,
            ActorPrecedence::Secondary => Secondary,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    #[serde(rename = "n")]
    pub name: String,

    #[serde(rename = "r", skip_serializing_if = "IsDefault::is_default", default)]
    pub role: ActorRole,

    #[serde(rename = "p", skip_serializing_if = "IsDefault::is_default", default)]
    pub precedence: ActorPrecedence,
}

impl From<Actor> for CoreActor {
    fn from(from: Actor) -> Self {
        Self {
            name: from.name,
            role: from.role.into(),
            precedence: from.precedence.into(),
        }
    }
}

#[cfg(test)]
mod tests;
