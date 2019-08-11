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
    title::{Title as CoreTitle, TitleLevel as CoreTitleLevel},
    util::IsDefault,
};

///////////////////////////////////////////////////////////////////////
// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TitleLevel {
    Main = 0, // default
    Sub = 1,
    Work = 2,
    Movement = 3,
}

impl Default for TitleLevel {
    fn default() -> TitleLevel {
        TitleLevel::Main
    }
}

impl From<TitleLevel> for CoreTitleLevel {
    fn from(from: TitleLevel) -> Self {
        use CoreTitleLevel::*;
        match from {
            TitleLevel::Main => Main,
            TitleLevel::Sub => Sub,
            TitleLevel::Work => Work,
            TitleLevel::Movement => Movement,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Title {
    #[serde(rename = "n")]
    pub name: String,

    #[serde(rename = "v", skip_serializing_if = "IsDefault::is_default", default)]
    pub level: TitleLevel,

    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl From<Title> for CoreTitle {
    fn from(from: Title) -> Self {
        Self {
            name: from.name,
            level: from.level.into(),
            language: from.language,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
