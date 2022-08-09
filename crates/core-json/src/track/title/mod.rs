// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    fn is_default(&self) -> bool {
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
        use _core::Kind::*;
        match from {
            Kind::Main => Main,
            Kind::Sub => Sub,
            Kind::Sorting => Sorting,
            Kind::Work => Work,
            Kind::Movement => Movement,
        }
    }
}

impl From<_core::Kind> for Kind {
    fn from(from: _core::Kind) -> Self {
        use _core::Kind::*;
        match from {
            Main => Kind::Main,
            Sub => Kind::Sub,
            Sorting => Kind::Sorting,
            Work => Kind::Work,
            Movement => Kind::Movement,
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
        use Title::*;
        match from {
            Name(name) => Self {
                name,
                kind: _core::Kind::Main,
            },
            NameAndKind(name, kind) => Self {
                name,
                kind: kind.into(),
            },
            FullTitle(actor) => actor.into(),
        }
    }
}

#[cfg(test)]
mod tests;
