// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::prelude::*;

use aoide_core::music::key::KeyCodeValue;

mod _core {
    pub use aoide_core::music::key::KeyCode;
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct KeyCode(KeyCodeValue);

impl From<_core::KeyCode> for KeyCode {
    fn from(from: _core::KeyCode) -> Self {
        Self(from.to_value())
    }
}

impl From<KeyCode> for _core::KeyCode {
    fn from(from: KeyCode) -> Self {
        let KeyCode(val) = from;
        Self::from_value(val)
    }
}
