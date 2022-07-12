// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::util::remote::RemoteData;

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
    #[must_use]
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
    #[must_use]
    pub fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }
}
