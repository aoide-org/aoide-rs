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

use aoide_core::util::url::BaseUrl;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub untrack_orphaned_directories: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub summary: Summary,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    pub untracked_directories: usize,
    pub purged_media_sources: usize,
}