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

use std::ops::AddAssign;

use aoide_core::util::url::BaseUrl;

use crate::{media::SyncMode, track::replace::Summary as TrackReplaceSummary};

use super::Completion;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub sync_mode: Option<SyncMode>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub completion: Completion,
    pub summary: Summary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub tracks: TrackSummary,
    pub directories: DirectorySummary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TrackSummary {
    pub created: usize,
    pub updated: usize,
    pub missing: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub failed: usize,
    pub not_created: usize,
    pub not_updated: usize,
}

impl AddAssign<&TrackReplaceSummary> for TrackSummary {
    fn add_assign(&mut self, rhs: &TrackReplaceSummary) {
        let Self {
            created,
            updated,
            unchanged,
            missing: _missing,
            skipped,
            failed,
            not_created,
            not_updated,
        } = self;
        debug_assert_eq!(0, *_missing);
        let TrackReplaceSummary {
            created: rhs_created,
            updated: rhs_updated,
            unchanged: rhs_unchanged,
            skipped: rhs_skipped,
            failed: rhs_failed,
            not_created: rhs_not_created,
            not_updated: rhs_not_updated,
        } = rhs;
        *created += rhs_created.len();
        *updated += rhs_updated.len();
        *unchanged += rhs_unchanged.len();
        *skipped += rhs_skipped.len();
        *failed += rhs_failed.len();
        *not_created += rhs_not_created.len();
        *not_updated += rhs_not_updated.len();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectorySummary {
    pub confirmed: usize,
    pub skipped: usize,
    pub untracked: usize,
}
