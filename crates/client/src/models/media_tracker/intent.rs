// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    Action, Effect, FetchStatus, IntentHandled, Model, StartFindUntrackedFiles, StartImportFiles,
    StartScanDirectories, UntrackDirectories,
};

#[derive(Debug)]
pub enum Intent {
    FetchProgress,
    FetchStatus(FetchStatus),
    StartScanDirectories(StartScanDirectories),
    StartImportFiles(StartImportFiles),
    StartFindUntrackedFiles(StartFindUntrackedFiles),
    UntrackDirectories(UntrackDirectories),
}

impl Intent {
    #[allow(clippy::too_many_lines)] // TODO
    #[must_use]
    pub fn apply_on(self, model: &Model) -> IntentHandled {
        log::trace!("Applying intent {self:?} on {model:?}");
        let next_action = match self {
            Self::FetchProgress => {
                if model.remote_view.progress.is_pending() {
                    let self_reconstructed = Self::FetchProgress;
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::FetchProgressAccepted;
                Action::apply_effect(effect)
            }
            Self::FetchStatus(fetch_status) => {
                if model.remote_view.status.is_pending() {
                    let self_reconstructed = Self::FetchStatus(fetch_status);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::FetchStatusAccepted(fetch_status);
                Action::apply_effect(effect)
            }
            Self::StartScanDirectories(start_scan_directories) => {
                if model.remote_view.last_scan_directories_outcome.is_pending() {
                    let self_reconstructed = Self::StartScanDirectories(start_scan_directories);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::StartScanDirectoriesAccepted(start_scan_directories);
                Action::apply_effect(effect)
            }
            Self::StartImportFiles(start_import_files) => {
                if model.remote_view.last_import_files_outcome.is_pending() {
                    let self_reconstructed = Self::StartImportFiles(start_import_files);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::StartImportFilesAccepted(start_import_files);
                Action::apply_effect(effect)
            }
            Self::StartFindUntrackedFiles(start_find_untracked_files) => {
                if model
                    .remote_view
                    .last_find_untracked_files_outcome
                    .is_pending()
                {
                    let self_reconstructed =
                        Self::StartFindUntrackedFiles(start_find_untracked_files);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::StartFindUntrackedFilesAccepted(start_find_untracked_files);
                Action::apply_effect(effect)
            }
            Self::UntrackDirectories(untrack_directories) => {
                if model
                    .remote_view
                    .last_untrack_directories_outcome
                    .is_pending()
                {
                    let self_reconstructed = Self::UntrackDirectories(untrack_directories);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::UntrackDirectoriesAccepted(untrack_directories);
                Action::apply_effect(effect)
            }
        };
        IntentHandled::Accepted(Some(next_action))
    }
}
