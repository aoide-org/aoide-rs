// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    pub use aoide_core::track::release::{DateOrDateTime, Release};
}

///////////////////////////////////////////////////////////////////////
// DateOrDateTime
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DateOrDateTime {
    Date(DateYYYYMMDD),
    DateTime(DateTime),
}

impl From<_core::DateOrDateTime> for DateOrDateTime {
    fn from(from: _core::DateOrDateTime) -> Self {
        use _core::DateOrDateTime::*;
        match from {
            Date(from) => Self::Date(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
        }
    }
}

impl From<DateOrDateTime> for _core::DateOrDateTime {
    fn from(from: DateOrDateTime) -> Self {
        use DateOrDateTime::*;
        match from {
            Date(from) => Self::Date(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Release
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Release {
    #[serde(skip_serializing_if = "Option::is_none")]
    released_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    released_at: Option<DateOrDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,
}

impl From<_core::Release> for Release {
    fn from(from: _core::Release) -> Self {
        let _core::Release {
            released_at,
            released_by,
            copyright,
        } = from;
        Self {
            released_at: released_at.map(Into::into),
            released_by,
            copyright,
        }
    }
}

impl From<Release> for _core::Release {
    fn from(from: Release) -> Self {
        let Release {
            released_at,
            released_by,
            copyright,
        } = from;
        Self {
            released_at: released_at.map(Into::into),
            released_by,
            copyright,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
