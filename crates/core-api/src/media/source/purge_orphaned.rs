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

use aoide_core::{media::content::ContentPath, util::url::BaseUrl};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: Option<BaseUrl>,
    pub root_path: Option<ContentPath>,
    pub summary: Summary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub purged: usize,
}
