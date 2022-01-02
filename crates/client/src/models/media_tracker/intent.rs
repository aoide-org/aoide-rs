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

use aoide_core::{entity::EntityUid, util::url::BaseUrl};

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    FetchProgress,
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    StartScanDirectories {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::scan_directories::Params,
    },
    StartImportFiles {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::import_files::Params,
    },
    StartFindUntrackedFiles {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::find_untracked_files::Params,
    },
    UntrackDirectories {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::untrack_directories::Params,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::FetchProgress => {
                if state.remote_view.progress.try_set_pending_now().is_none() {
                    log::warn!("Discarding intent while pending: {:?}", Self::FetchProgress);
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::maybe_changed(Action::dispatch_task(Task::FetchProgress))
            }
            Self::FetchStatus {
                collection_uid,
                root_url,
            } => {
                if state.remote_view.status.try_set_pending_now().is_none() {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::FetchStatus {
                            collection_uid,
                            root_url,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::maybe_changed(Action::dispatch_task(Task::FetchStatus {
                    collection_uid,
                    root_url,
                }))
            }
            Self::StartScanDirectories {
                collection_uid,
                params,
            } => {
                if state
                    .remote_view
                    .last_scan_directories_outcome
                    .try_set_pending_now()
                    .is_none()
                {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::StartScanDirectories {
                            collection_uid,
                            params,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                // Start batch task
                StateUpdated::maybe_changed(Action::dispatch_task(Task::StartScanDirectories {
                    collection_uid,
                    params,
                }))
            }
            Self::StartImportFiles {
                collection_uid,
                params,
            } => {
                if state
                    .remote_view
                    .last_import_files_outcome
                    .try_set_pending_now()
                    .is_none()
                {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::StartImportFiles {
                            collection_uid,
                            params,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                // Start batch task
                StateUpdated::maybe_changed(Action::dispatch_task(Task::StartImportFiles {
                    collection_uid,
                    params,
                }))
            }
            Self::StartFindUntrackedFiles {
                collection_uid,
                params,
            } => {
                if state
                    .remote_view
                    .last_find_untracked_files_outcome
                    .try_set_pending_now()
                    .is_none()
                {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::StartFindUntrackedFiles {
                            collection_uid,
                            params,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                // Start batch task
                StateUpdated::maybe_changed(Action::dispatch_task(Task::StartFindUntrackedFiles {
                    collection_uid,
                    params,
                }))
            }
            Self::UntrackDirectories {
                collection_uid,
                params,
            } => {
                if state
                    .remote_view
                    .last_untrack_directories_outcome
                    .try_set_pending_now()
                    .is_none()
                {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::UntrackDirectories {
                            collection_uid,
                            params,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::maybe_changed(Action::dispatch_task(Task::UntrackDirectories {
                    collection_uid,
                    params,
                }))
            }
        }
    }
}
