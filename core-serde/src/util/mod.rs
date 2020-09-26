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

pub mod color;

mod _core {
    pub use aoide_core::util::LockableValue;
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LockableValue<T> {
    #[serde(rename = "val")]
    pub value: T,

    #[serde(rename = "lck")]
    pub locked: bool,
}

impl<T, U> From<_core::LockableValue<T>> for LockableValue<U>
where
    T: Into<U>,
{
    fn from(from: _core::LockableValue<T>) -> Self {
        let _core::LockableValue { value, locked } = from;
        Self {
            value: value.into(),
            locked,
        }
    }
}

impl<T, U> From<LockableValue<T>> for _core::LockableValue<U>
where
    T: Into<U>,
{
    fn from(from: LockableValue<T>) -> Self {
        let LockableValue { value, locked } = from;
        Self {
            value: value.into(),
            locked,
        }
    }
}
