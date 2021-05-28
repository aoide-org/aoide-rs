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

use crate::usecases::tracks::replace::Summary as TrackReplaceSummary;

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
    pub not_imported: usize,
    pub not_created: usize,
    pub not_updated: usize,
}

impl AddAssign<&TrackReplaceSummary> for TrackSummary {
    fn add_assign(&mut self, rhs: &TrackReplaceSummary) {
        let Self {
            created,
            updated,
            unchanged,
            missing: _,
            not_imported,
            not_created,
            not_updated,
        } = self;
        let TrackReplaceSummary {
            created: rhs_created,
            updated: rhs_updated,
            unchanged: rhs_unchanged,
            not_imported: rhs_not_imported,
            not_created: rhs_not_created,
            not_updated: rhs_not_updated,
        } = rhs;
        *created += rhs_created.len();
        *updated += rhs_updated.len();
        *unchanged += rhs_unchanged.len();
        *not_imported += rhs_not_imported.len();
        *not_created += rhs_not_created.len();
        *not_updated += rhs_not_updated.len();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectorySummary {
    pub confirmed: usize,
    pub rejected: usize,
    pub skipped: usize,
}
