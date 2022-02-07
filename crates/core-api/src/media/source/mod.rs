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

use aoide_core::util::url::BaseUrl;

pub mod purge_orphaned;
pub mod purge_untracked;

#[derive(Debug, Clone)]
pub enum ResolveUrlFromPath {
    CanonicalRootUrl,
    OverrideRootUrl { root_url: BaseUrl },
}

impl Default for ResolveUrlFromPath {
    fn default() -> Self {
        Self::CanonicalRootUrl
    }
}

impl ResolveUrlFromPath {
    #[must_use]
    pub const fn override_root_url(&self) -> Option<&BaseUrl> {
        match self {
            Self::CanonicalRootUrl => None,
            Self::OverrideRootUrl { root_url } => Some(root_url),
        }
    }
}
