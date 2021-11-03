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

use crate::prelude::*;

pub mod tracker;

mod _core {
    pub use aoide_core_ext::media::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(rename_all = "kebab-case")]
pub enum ImportMode {
    Once,
    Modified,
    Always,
}

#[cfg(feature = "backend")]
impl From<ImportMode> for _core::ImportMode {
    fn from(from: ImportMode) -> Self {
        use ImportMode::*;
        match from {
            Once => Self::Once,
            Modified => Self::Modified,
            Always => Self::Always,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_core::ImportMode> for ImportMode {
    fn from(from: _core::ImportMode) -> Self {
        use _core::ImportMode::*;
        match from {
            Once => Self::Once,
            Modified => Self::Modified,
            Always => Self::Always,
        }
    }
}