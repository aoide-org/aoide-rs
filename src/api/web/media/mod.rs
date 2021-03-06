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

use aoide_core::media::resolver::VirtualFilePathResolver;

use super::*;

///////////////////////////////////////////////////////////////////////

pub mod import_track;
pub mod relocate_collected_sources;
pub mod tracker;

mod uc {
    pub use crate::usecases::media::ImportMode;
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportMode {
    Once,
    Modified,
    Always,
}

impl From<ImportMode> for uc::ImportMode {
    fn from(from: ImportMode) -> Self {
        match from {
            ImportMode::Once => Self::Once,
            ImportMode::Modified => Self::Modified,
            ImportMode::Always => Self::Always,
        }
    }
}
