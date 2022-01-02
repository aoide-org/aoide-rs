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

#[derive(Debug, Default)]
pub struct RemoteView {
    pub status: RemoteData<aoide_core_api::media::tracker::Status>,
    pub progress: RemoteData<aoide_core_api::media::tracker::Progress>,
    pub last_scan_directories_outcome:
        RemoteData<aoide_core_api::media::tracker::scan_directories::Outcome>,
    pub last_untrack_directories_outcome:
        RemoteData<aoide_core_api::media::tracker::untrack_directories::Outcome>,
    pub last_import_files_outcome:
        RemoteData<aoide_core_api::media::tracker::import_files::Outcome>,
    pub last_find_untracked_files_outcome:
        RemoteData<aoide_core_api::media::tracker::find_untracked_files::Outcome>,
}

impl RemoteView {
    pub fn is_pending(&self) -> bool {
        self.status.is_pending()
            || self.progress.is_pending()
            || self.last_scan_directories_outcome.is_pending()
            || self.last_untrack_directories_outcome.is_pending()
            || self.last_import_files_outcome.is_pending()
            || self.last_find_untracked_files_outcome.is_pending()
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub(super) remote_view: RemoteView,
}

impl State {
    pub fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }
}
