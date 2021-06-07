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

use crate::prelude::remote::RemoteData;

use aoide_core::usecases::media::tracker::{
    import::Outcome as ImportOutcome, scan::Outcome as ScanOutcome,
    untrack::Outcome as UntrackOutcome, Progress, Status,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    Idle,
    Busy,
}

impl Default for ControlState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Default)]
pub struct RemoteView {
    pub(super) status: RemoteData<Status>,
    pub(super) progress: RemoteData<Progress>,
    pub(super) last_scan_outcome: RemoteData<ScanOutcome>,
    pub(super) last_import_outcome: RemoteData<ImportOutcome>,
    pub(super) last_untrack_outcome: RemoteData<UntrackOutcome>,
}

impl RemoteView {
    pub fn status(&self) -> &RemoteData<Status> {
        &self.status
    }

    pub fn progress(&self) -> &RemoteData<Progress> {
        &self.progress
    }

    pub fn last_scan_outcome(&self) -> &RemoteData<ScanOutcome> {
        &self.last_scan_outcome
    }

    pub fn last_import_outcome(&self) -> &RemoteData<ImportOutcome> {
        &self.last_import_outcome
    }

    pub fn last_untrack_outcome(&self) -> &RemoteData<UntrackOutcome> {
        &self.last_untrack_outcome
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub(super) control_state: ControlState,
    pub(super) remote_view: RemoteView,
}

impl Model {
    pub fn control_state(&self) -> ControlState {
        self.control_state
    }

    pub fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }

    pub fn is_idle(&self) -> bool {
        self.control_state == ControlState::Idle
            && (self.remote_view.progress.get().map(|x| &x.value) == Some(&Progress::Idle)
                || self.remote_view.progress.is_unknown())
    }
}
