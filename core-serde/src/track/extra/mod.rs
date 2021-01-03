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
    pub use aoide_core::track::extra::*;
}

use crate::util::color::Color;

///////////////////////////////////////////////////////////////////////
// Extra
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extra {
    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

impl From<_core::Extra> for Extra {
    fn from(from: _core::Extra) -> Self {
        let _core::Extra { color } = from;
        Self {
            color: color.map(Into::into),
        }
    }
}

impl From<Extra> for _core::Extra {
    fn from(from: Extra) -> Self {
        let Extra { color } = from;
        Self {
            color: color.map(Into::into),
        }
    }
}
