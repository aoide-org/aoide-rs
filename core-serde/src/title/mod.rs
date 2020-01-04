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
    pub use aoide_core::title::{Title, TitleLevel};
}

///////////////////////////////////////////////////////////////////////
// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TitleLevel {
    Main = 0,
    Sub = 1,
    Work = 2,
    Movement = 3,
}

impl Default for TitleLevel {
    fn default() -> TitleLevel {
        _core::TitleLevel::default().into()
    }
}

impl From<TitleLevel> for _core::TitleLevel {
    fn from(from: TitleLevel) -> Self {
        use _core::TitleLevel::*;
        match from {
            TitleLevel::Main => Main,
            TitleLevel::Sub => Sub,
            TitleLevel::Work => Work,
            TitleLevel::Movement => Movement,
        }
    }
}

impl From<_core::TitleLevel> for TitleLevel {
    fn from(from: _core::TitleLevel) -> Self {
        use _core::TitleLevel::*;
        match from {
            Main => TitleLevel::Main,
            Sub => TitleLevel::Sub,
            Work => TitleLevel::Work,
            Movement => TitleLevel::Movement,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Title {
    Name(String),
    NameLevel(String, TitleLevel),
}

impl From<Title> for _core::Title {
    fn from(from: Title) -> Self {
        use Title::*;
        match from {
            Name(name) => Self {
                name,
                ..Default::default()
            },
            NameLevel(name, level) => Self {
                name,
                level: level.into(),
            },
        }
    }
}

impl From<_core::Title> for Title {
    fn from(from: _core::Title) -> Self {
        let _core::Title { name, level } = from;
        use Title::*;
        if level == Default::default() {
            Name(name)
        } else {
            NameLevel(name, level.into())
        }
    }
}
