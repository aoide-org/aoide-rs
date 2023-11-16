// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::track::title::{Kind, Title};
}

///////////////////////////////////////////////////////////////////////
// Kind
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum Kind {
    Main = _core::Kind::Main as u8,
    Sub = _core::Kind::Sub as u8,
    Sorting = _core::Kind::Sorting as u8,
    Work = _core::Kind::Work as u8,
    Movement = _core::Kind::Movement as u8,
}

impl Kind {
    const fn is_default(&self) -> bool {
        matches!(self, Kind::Main)
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
            From::Main => Self::Main,
            From::Sub => Self::Sub,
            From::Sorting => Self::Sorting,
            From::Work => Self::Work,
            From::Movement => Self::Movement,
        }
    }
}

impl From<_core::Kind> for Kind {
    fn from(from: _core::Kind) -> Self {
        use _core::Kind as From;
        match from {
            From::Main => Self::Main,
            From::Sub => Self::Sub,
            From::Sorting => Self::Sorting,
            From::Work => Self::Work,
            From::Movement => Self::Movement,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullTitle {
    name: String,

    #[serde(skip_serializing_if = "Kind::is_default", default)]
    kind: Kind,
}

impl From<_core::Title> for FullTitle {
    fn from(from: _core::Title) -> Self {
        let _core::Title { name, kind } = from;
        Self {
            name,
            kind: kind.into(),
        }
    }
}

impl From<FullTitle> for _core::Title {
    fn from(from: FullTitle) -> Self {
        let FullTitle { name, kind } = from;
        Self {
            name,
            kind: kind.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Title {
    Name(String),              // name
    NameAndKind(String, Kind), // [name,kind]
    FullTitle(FullTitle),      // {name,kind,..}
}

impl From<_core::Title> for Title {
    fn from(from: _core::Title) -> Self {
        let _core::Title { name, kind } = from;
        if kind == _core::Kind::Main {
            Self::Name(name)
        } else {
            Self::NameAndKind(name, kind.into())
        }
    }
}

impl From<Title> for _core::Title {
    fn from(from: Title) -> Self {
        use Title as From;
        match from {
            From::Name(name) => Self {
                name,
                kind: _core::Kind::Main,
            },
            From::NameAndKind(name, kind) => Self {
                name,
                kind: kind.into(),
            },
            From::FullTitle(actor) => actor.into(),
        }
    }
}

#[cfg(test)]
mod tests;
