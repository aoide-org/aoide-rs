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

use chrono::{DateTime, Utc};

///////////////////////////////////////////////////////////////////////
// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReleaseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1))]
    pub released_by: Option<String>, // record label

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1))]
    pub copyright: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    // TODO: Validate that each license is not empty
    pub licenses: Vec<String>,
}
