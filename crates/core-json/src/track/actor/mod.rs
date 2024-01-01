// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::track::actor::{Actor, Kind, Role};
}

///////////////////////////////////////////////////////////////////////
// Kind
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum Kind {
    Summary = _core::Kind::Summary as u8,
    Individual = _core::Kind::Individual as u8,
    Sorting = _core::Kind::Sorting as u8,
}

impl Kind {
    const fn is_default(&self) -> bool {
        matches!(self, Kind::Summary)
    }
}

impl Default for Kind {
    fn default() -> Kind {
        _core::Kind::default().into()
    }
}

impl From<Kind> for _core::Kind {
    fn from(from: Kind) -> Self {
        use Kind as From;
        match from {
            From::Summary => Self::Summary,
            From::Individual => Self::Individual,
            From::Sorting => Self::Sorting,
        }
    }
}

impl From<_core::Kind> for Kind {
    fn from(from: _core::Kind) -> Self {
        use _core::Kind as From;
        match from {
            From::Summary => Self::Summary,
            From::Individual => Self::Individual,
            From::Sorting => Self::Sorting,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Role
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum Role {
    Artist = _core::Role::Artist as u8,
    Arranger = _core::Role::Arranger as u8,
    Composer = _core::Role::Composer as u8,
    Conductor = _core::Role::Conductor as u8,
    MixDj = _core::Role::MixDj as u8,
    Engineer = _core::Role::Engineer as u8,
    Lyricist = _core::Role::Lyricist as u8,
    MixEngineer = _core::Role::MixEngineer as u8,
    Performer = _core::Role::Performer as u8,
    Producer = _core::Role::Producer as u8,
    Director = _core::Role::Director as u8,
    Remixer = _core::Role::Remixer as u8,
    Writer = _core::Role::Writer as u8,
}

impl Role {
    const fn is_default(&self) -> bool {
        matches!(self, Self::Artist)
    }
}

impl Default for Role {
    fn default() -> Role {
        _core::Role::default().into()
    }
}

impl From<Role> for _core::Role {
    fn from(from: Role) -> Self {
        use Role as From;
        match from {
            From::Artist => Self::Artist,
            From::Arranger => Self::Arranger,
            From::Composer => Self::Composer,
            From::Conductor => Self::Conductor,
            From::MixDj => Self::MixDj,
            From::Engineer => Self::Engineer,
            From::Lyricist => Self::Lyricist,
            From::MixEngineer => Self::MixEngineer,
            From::Performer => Self::Performer,
            From::Producer => Self::Producer,
            From::Director => Self::Director,
            From::Remixer => Self::Remixer,
            From::Writer => Self::Writer,
        }
    }
}

impl From<_core::Role> for Role {
    fn from(from: _core::Role) -> Self {
        use _core::Role as From;
        match from {
            From::Artist => Self::Artist,
            From::Arranger => Self::Arranger,
            From::Composer => Self::Composer,
            From::Conductor => Self::Conductor,
            From::MixDj => Self::MixDj,
            From::Engineer => Self::Engineer,
            From::Lyricist => Self::Lyricist,
            From::MixEngineer => Self::MixEngineer,
            From::Performer => Self::Performer,
            From::Producer => Self::Producer,
            From::Director => Self::Director,
            From::Remixer => Self::Remixer,
            From::Writer => Self::Writer,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Actor
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullActor {
    #[serde(skip_serializing_if = "Kind::is_default", default)]
    kind: Kind,

    name: String,

    #[serde(skip_serializing_if = "Role::is_default", default)]
    role: Role,

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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Actor {
    Name(String),              // name
    NameAndRole(String, Role), // [name,role]
    FullActor(FullActor),      // {name,role,..}
}

impl From<_core::Actor> for Actor {
    fn from(from: _core::Actor) -> Self {
        let _core::Actor {
            kind,
            name,
            role,
            role_notes,
        } = from;
        if kind == _core::Kind::Summary && role_notes.is_none() {
            if role == _core::Role::Artist {
                Self::Name(name)
            } else {
                Self::NameAndRole(name, role.into())
            }
        } else {
            Self::FullActor(FullActor {
                kind: kind.into(),
                name,
                role: role.into(),
                role_notes: role_notes.map(Into::into),
            })
        }
    }
}

impl From<Actor> for _core::Actor {
    fn from(from: Actor) -> Self {
        use Actor as From;
        match from {
            From::Name(name) => Self {
                kind: _core::Kind::Summary,
                name,
                role: _core::Role::Artist,
                role_notes: None,
            },
            From::NameAndRole(name, role) => Self {
                kind: _core::Kind::Summary,
                name,
                role: role.into(),
                role_notes: None,
            },
            From::FullActor(actor) => actor.into(),
        }
    }
}

#[cfg(test)]
mod tests;
