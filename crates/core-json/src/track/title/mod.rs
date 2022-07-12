// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::track::title::{Title, TitleKind};
}

///////////////////////////////////////////////////////////////////////
// TitleKind
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[repr(u8)]
pub enum TitleKind {
    Main = _core::TitleKind::Main as u8,
    Sub = _core::TitleKind::Sub as u8,
    Sorting = _core::TitleKind::Sorting as u8,
    Work = _core::TitleKind::Work as u8,
    Movement = _core::TitleKind::Movement as u8,
}

impl TitleKind {
    fn is_default(&self) -> bool {
        matches!(self, TitleKind::Main)
    }
}

impl Default for TitleKind {
    fn default() -> TitleKind {
        _core::TitleKind::default().into()
    }
}

impl From<TitleKind> for _core::TitleKind {
    fn from(from: TitleKind) -> Self {
        use _core::TitleKind::*;
        match from {
            TitleKind::Main => Main,
            TitleKind::Sub => Sub,
            TitleKind::Sorting => Sorting,
            TitleKind::Work => Work,
            TitleKind::Movement => Movement,
        }
    }
}

impl From<_core::TitleKind> for TitleKind {
    fn from(from: _core::TitleKind) -> Self {
        use _core::TitleKind::*;
        match from {
            Main => TitleKind::Main,
            Sub => TitleKind::Sub,
            Sorting => TitleKind::Sorting,
            Work => TitleKind::Work,
            Movement => TitleKind::Movement,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullTitle {
    name: String,

    #[serde(skip_serializing_if = "TitleKind::is_default", default)]
    kind: TitleKind,
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
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Title {
    Name(String),                   // name
    NameAndKind(String, TitleKind), // [name,kind]
    FullTitle(FullTitle),           // {name,kind,..}
}

impl From<_core::Title> for Title {
    fn from(from: _core::Title) -> Self {
        let _core::Title { name, kind } = from;
        if kind == _core::TitleKind::Main {
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
                kind: _core::TitleKind::Main,
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
