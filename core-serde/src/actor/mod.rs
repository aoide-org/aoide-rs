// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod _core {
    pub use aoide_core::actor::{Actor, ActorPrecedence, ActorRole};
}

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ActorRole {
    Artist = 0,
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
        _core::ActorRole::default().into()
    }
}

impl From<ActorRole> for _core::ActorRole {
    fn from(from: ActorRole) -> Self {
        use _core::ActorRole::*;
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

impl From<_core::ActorRole> for ActorRole {
    fn from(from: _core::ActorRole) -> Self {
        use _core::ActorRole::*;
        match from {
            Artist => ActorRole::Artist,
            Arranger => ActorRole::Arranger,
            Composer => ActorRole::Composer,
            Conductor => ActorRole::Conductor,
            DjMixer => ActorRole::DjMixer,
            Engineer => ActorRole::Engineer,
            Lyricist => ActorRole::Lyricist,
            Mixer => ActorRole::Mixer,
            Performer => ActorRole::Performer,
            Producer => ActorRole::Producer,
            Publisher => ActorRole::Publisher,
            Remixer => ActorRole::Remixer,
            Writer => ActorRole::Writer,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// ActorPrecedence
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ActorPrecedence {
    Summary = 0,
    Primary = 1,
    Secondary = 2,
}

impl Default for ActorPrecedence {
    fn default() -> ActorPrecedence {
        _core::ActorPrecedence::default().into()
    }
}

impl From<ActorPrecedence> for _core::ActorPrecedence {
    fn from(from: ActorPrecedence) -> Self {
        use _core::ActorPrecedence::*;
        match from {
            ActorPrecedence::Summary => Summary,
            ActorPrecedence::Primary => Primary,
            ActorPrecedence::Secondary => Secondary,
        }
    }
}

impl From<_core::ActorPrecedence> for ActorPrecedence {
    fn from(from: _core::ActorPrecedence) -> Self {
        use _core::ActorPrecedence::*;
        match from {
            Summary => ActorPrecedence::Summary,
            Primary => ActorPrecedence::Primary,
            Secondary => ActorPrecedence::Secondary,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Actor {
    Name(String),
    NameRole(String, ActorRole),
    NameRolePrecedence(String, ActorRole, ActorPrecedence),
}

impl From<Actor> for _core::Actor {
    fn from(from: Actor) -> Self {
        use Actor::*;
        match from {
            Name(name) => Self {
                name,
                ..Default::default()
            },
            NameRole(name, role) => Self {
                name,
                role: role.into(),
                ..Default::default()
            },
            NameRolePrecedence(name, role, precedence) => Self {
                name,
                role: role.into(),
                precedence: precedence.into(),
            },
        }
    }
}

impl From<_core::Actor> for Actor {
    fn from(from: _core::Actor) -> Self {
        let _core::Actor {
            name,
            role,
            precedence,
        } = from;
        use Actor::*;
        if precedence == Default::default() {
            if role == Default::default() {
                Name(name)
            } else {
                NameRole(name, role.into())
            }
        } else {
            NameRolePrecedence(name, role.into(), precedence.into())
        }
    }
}
