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

use aoide_core_api::media::tracker::{
    find_untracked_files::Outcome as FindUntrackedFilesOutcome,
    import_files::Outcome as ImportFilesOutcome,
    scan_directories::Outcome as ScanDirectoriesOutcome,
    untrack_directories::Outcome as UntrackDirectoriesOutcome, Progress, Status,
};

use crate::prelude::remote::RemoteData;

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
    pub(super) last_scan_directories_outcome: RemoteData<ScanDirectoriesOutcome>,
    pub(super) last_untrack_directories_outcome: RemoteData<UntrackDirectoriesOutcome>,
    pub(super) last_import_files_outcome: RemoteData<ImportFilesOutcome>,
    pub(super) last_find_untracked_files_outcome: RemoteData<FindUntrackedFilesOutcome>,
    pub(super) last_purge_orphaned_and_untracked_outcome: RemoteData<()>,
}

impl RemoteView {
    pub fn status(&self) -> &RemoteData<Status> {
        &self.status
    }

    pub fn progress(&self) -> &RemoteData<Progress> {
        &self.progress
    }

    pub fn last_scan_directories_outcome(&self) -> &RemoteData<ScanDirectoriesOutcome> {
        &self.last_scan_directories_outcome
    }

    pub fn last_untrack_directories_outcome(&self) -> &RemoteData<UntrackDirectoriesOutcome> {
        &self.last_untrack_directories_outcome
    }

    pub fn last_import_files_outcome(&self) -> &RemoteData<ImportFilesOutcome> {
        &self.last_import_files_outcome
    }

    pub fn last_find_untracked_files_outcome(&self) -> &RemoteData<FindUntrackedFilesOutcome> {
        &self.last_find_untracked_files_outcome
    }

    pub fn last_purge_orphaned_and_untracked_outcome(&self) -> &RemoteData<()> {
        &self.last_purge_orphaned_and_untracked_outcome
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub(super) control_state: ControlState,
    pub(super) remote_view: RemoteView,
}

impl State {
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
