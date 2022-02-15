// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::prelude::*;

mod _core {
    pub use aoide_core::track::actor::{Actor, ActorKind, ActorRole};
}

///////////////////////////////////////////////////////////////////////
// ActorKind
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[repr(u8)]
pub enum ActorKind {
    Summary = _core::ActorKind::Summary as u8,
    Individual = _core::ActorKind::Individual as u8,
    Sorting = _core::ActorKind::Sorting as u8,
}

impl ActorKind {
    fn is_default(&self) -> bool {
        matches!(self, ActorKind::Summary)
    }
}

impl Default for ActorKind {
    fn default() -> ActorKind {
        _core::ActorKind::default().into()
    }
}

impl From<ActorKind> for _core::ActorKind {
    fn from(from: ActorKind) -> Self {
        use _core::ActorKind::*;
        match from {
            ActorKind::Summary => Summary,
            ActorKind::Individual => Individual,
            ActorKind::Sorting => Sorting,
        }
    }
}

impl From<_core::ActorKind> for ActorKind {
    fn from(from: _core::ActorKind) -> Self {
        use _core::ActorKind::*;
        match from {
            Summary => ActorKind::Summary,
            Individual => ActorKind::Individual,
            Sorting => ActorKind::Sorting,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// ActorRole
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[repr(u8)]
pub enum ActorRole {
    Artist = _core::ActorRole::Artist as u8,
    Arranger = _core::ActorRole::Arranger as u8,
    Composer = _core::ActorRole::Composer as u8,
    Conductor = _core::ActorRole::Conductor as u8,
    DjMixer = _core::ActorRole::DjMixer as u8,
    Engineer = _core::ActorRole::Engineer as u8,
    Lyricist = _core::ActorRole::Lyricist as u8,
    Mixer = _core::ActorRole::Mixer as u8,
    Performer = _core::ActorRole::Performer as u8,
    Producer = _core::ActorRole::Producer as u8,
    Director = _core::ActorRole::Director as u8,
    Remixer = _core::ActorRole::Remixer as u8,
    Writer = _core::ActorRole::Writer as u8,
}

impl ActorRole {
    fn is_default(&self) -> bool {
        matches!(self, Self::Artist)
    }
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
            ActorRole::Director => Director,
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
            Director => ActorRole::Director,
            Remixer => ActorRole::Remixer,
            Writer => ActorRole::Writer,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullActor {
    #[serde(skip_serializing_if = "ActorKind::is_default", default)]
    kind: ActorKind,

    name: String,

    #[serde(skip_serializing_if = "ActorRole::is_default", default)]
    role: ActorRole,

    #[serde(skip_serializing_if = "Option::is_none")]
    role_notes: Option<String>,
}

impl From<_core::Actor> for FullActor {
    fn from(from: _core::Actor) -> Self {
        let _core::Actor {
            kind,
            name,
            role,
            role_notes,
        } = from;
        Self {
            kind: kind.into(),
            name,
            role: role.into(),
            role_notes: role_notes.map(Into::into),
        }
    }
}

impl From<FullActor> for _core::Actor {
    fn from(from: FullActor) -> Self {
        let FullActor {
            kind,
            name,
            role,
            role_notes,
        } = from;
        Self {
            kind: kind.into(),
            name,
            role: role.into(),
            role_notes: role_notes.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Actor {
    Name(String),                   // name
    NameAndRole(String, ActorRole), // [name,role]
    FullActor(FullActor),           // {name,role,..}
}

impl From<_core::Actor> for Actor {
    fn from(from: _core::Actor) -> Self {
        let _core::Actor {
            kind,
            name,
            role,
            role_notes,
        } = from;
        if kind == _core::ActorKind::Summary && role_notes.is_none() {
            if role == _core::ActorRole::Artist {
                return Self::Name(name);
            } else {
                return Self::NameAndRole(name, role.into());
            }
        }
        Self::FullActor(FullActor {
            kind: kind.into(),
            name,
            role: role.into(),
            role_notes: role_notes.map(Into::into),
        })
    }
}

impl From<Actor> for _core::Actor {
    fn from(from: Actor) -> Self {
        use Actor::*;
        match from {
            Name(name) => Self {
                kind: _core::ActorKind::Summary,
                name,
                role: _core::ActorRole::Artist,
                role_notes: None,
            },
            NameAndRole(name, role) => Self {
                kind: _core::ActorKind::Summary,
                name,
                role: role.into(),
                role_notes: None,
            },
            FullActor(actor) => actor.into(),
        }
    }
}

#[cfg(test)]
mod tests;
