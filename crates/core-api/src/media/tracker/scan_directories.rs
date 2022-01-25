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

use aoide_core::{media::SourcePath, util::url::BaseUrl};

use super::{Completion, FsTraversalParams};

pub type Params = FsTraversalParams;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub root_path: SourcePath,
    pub completion: Completion,
    pub summary: Summary,
}
