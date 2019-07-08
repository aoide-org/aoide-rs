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

///////////////////////////////////////////////////////////////////////
// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum TitleLevel {
    Main = 0, // default
    Sub = 1,
    // for classical music, only used for tracks not albums
    #[serde(rename = "wrk")]
    Work = 2,
    #[serde(rename = "mvn")]
    Movement = 3,
}

impl Default for TitleLevel {
    fn default() -> TitleLevel {
        TitleLevel::Main
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Title {
    #[validate(length(min = 1))]
    pub name: String,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub level: TitleLevel,

    #[serde(rename = "lang", skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1))]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Titles;

impl Titles {
    pub fn validate_main_title(titles: &[Title]) -> Result<(), ValidationError> {
        if !titles.is_empty() && Self::main_title(titles).is_none() {
            return Err(ValidationError::new("missing main title"));
        }
        Ok(())
    }

    pub fn title<'a>(
        titles: &'a [Title],
        level: TitleLevel,
        language: Option<&str>,
    ) -> Option<&'a Title> {
        debug_assert!(
            titles
                .iter()
                .filter(|title| title.level == level
                    && title.language.as_ref().map(String::as_str) == language)
                .count()
                <= 1
        );
        titles
            .iter()
            .filter(|title| {
                title.level == level && title.language.as_ref().map(String::as_str) == language
            })
            .nth(0)
    }

    pub fn main_title(titles: &[Title]) -> Option<&Title> {
        Self::title(titles, TitleLevel::Main, None)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
