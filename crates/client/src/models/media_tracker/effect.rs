// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    EffectApplied, FetchStatus, Model, PendingTask, StartFindUntrackedFiles, StartImportFiles,
    StartScanDirectories, Task, UntrackDirectories,
};
use crate::util::roundtrip::PendingToken;

#[derive(Debug)]
pub enum Effect {
    FetchProgressAccepted,
    FetchProgressFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::Progress>,
    },
    FetchStatusAccepted(FetchStatus),
    FetchStatusFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::Status>,
    },
    StartScanDirectoriesAccepted(StartScanDirectories),
    ScanDirectoriesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::scan_directories::Outcome>,
    },
    UntrackDirectoriesAccepted(UntrackDirectories),
    UntrackDirectoriesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::untrack_directories::Outcome>,
    },
    StartImportFilesAccepted(StartImportFiles),
    ImportFilesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::import_files::Outcome>,
    },
    StartFindUntrackedFilesAccepted(StartFindUntrackedFiles),
    FindUntrackedFilesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::find_untracked_files::Outcome>,
    },
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    #[allow(clippy::too_many_lines)] // TODO
    pub fn apply_on(self, model: &mut Model) -> EffectApplied {
        log::trace!("Applying effect {self:?} on {model:?}");
        match self {
            Self::FetchProgressAccepted => {
                debug_assert!(!model.remote_view().progress.is_pending());
                model.last_error = None;
                let token = model.remote_view.progress.start_pending_now();
                let task = PendingTask::FetchProgress;
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed(task)
            }
            Self::FetchProgressFinished { token, result } => match result {
                Ok(progress) => {
                    if let Err(outcome) = model
                        .remote_view
                        .progress
                        .finish_pending_with_value_now(token, progress)
                    {
                        let effect_reconstructed = Self::FetchProgressFinished {
                            token,
                            result: Ok(outcome),
                        };
                        // Doesn't matter when fetching data
                        log::debug!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged_done();
                    }
                    EffectApplied::maybe_changed_done()
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model.remote_view.progress.finish_pending(token);
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::FetchStatusAccepted(fetch_status) => {
                debug_assert!(!model.remote_view().status.is_pending());
                model.last_error = None;
                let token = model.remote_view.status.start_pending_now();
                let task = PendingTask::FetchStatus(fetch_status);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed(task)
            }
            Self::FetchStatusFinished { token, result } => match result {
                Ok(status) => {
                    if let Err(outcome) = model
                        .remote_view
                        .status
                        .finish_pending_with_value_now(token, status)
                    {
                        let effect_reconstructed = Self::FetchStatusFinished {
                            token,
                            result: Ok(outcome),
                        };
                        // Doesn't matter when fetching data
                        log::debug!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged_done();
                    }
                    EffectApplied::maybe_changed_done()
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model.remote_view.status.finish_pending(token);
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::StartScanDirectoriesAccepted(start_scan_directories) => {
                debug_assert!(!model
                    .remote_view()
                    .last_scan_directories_outcome
                    .is_pending());
                model.last_error = None;
                let token = model
                    .remote_view
                    .last_scan_directories_outcome
                    .start_pending_now();
                let task = PendingTask::StartScanDirectories(start_scan_directories);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed(task)
            }
            Self::ScanDirectoriesFinished { token, result } => match result {
                Ok(outcome) => {
                    if let Err(outcome) = model
                        .remote_view
                        .last_scan_directories_outcome
                        .finish_pending_with_value_now(token, outcome)
                    {
                        let effect_reconstructed = Self::ScanDirectoriesFinished {
                            token,
                            result: Ok(outcome),
                        };
                        log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged_done();
                    }
                    fetch_progress_effect(model)
                        .map_or_else(EffectApplied::unchanged_done, |effect| {
                            effect.apply_on(model)
                        })
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model
                        .remote_view
                        .last_scan_directories_outcome
                        .finish_pending(token);
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::UntrackDirectoriesAccepted(untrack_directories) => {
                debug_assert!(!model
                    .remote_view()
                    .last_untrack_directories_outcome
                    .is_pending());
                model.last_error = None;
                let token = model
                    .remote_view
                    .last_untrack_directories_outcome
                    .start_pending_now();
                let task = PendingTask::UntrackDirectories(untrack_directories);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed(task)
            }
            Self::UntrackDirectoriesFinished { token, result } => match result {
                Ok(outcome) => {
                    if let Err(outcome) = model
                        .remote_view
                        .last_untrack_directories_outcome
                        .finish_pending_with_value_now(token, outcome)
                    {
                        let effect_reconstructed = Self::UntrackDirectoriesFinished {
                            token,
                            result: Ok(outcome),
                        };
                        log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged_done();
                    }
                    fetch_progress_effect(model)
                        .map_or_else(EffectApplied::unchanged_done, |effect| {
                            effect.apply_on(model)
                        })
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model.remote_view.last_untrack_directories_outcome.reset();
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::StartImportFilesAccepted(start_import_files) => {
                debug_assert!(!model.remote_view().last_import_files_outcome.is_pending());
                model.last_error = None;
                let token = model
                    .remote_view
                    .last_import_files_outcome
                    .start_pending_now();
                let task = PendingTask::StartImportFiles(start_import_files);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed(task)
            }
            Self::ImportFilesFinished { token, result } => match result {
                Ok(outcome) => {
                    if let Err(outcome) = model
                        .remote_view
                        .last_import_files_outcome
                        .finish_pending_with_value_now(token, outcome)
                    {
                        let effect_reconstructed = Self::ImportFilesFinished {
                            token,
                            result: Ok(outcome),
                        };
                        log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged_done();
                    }
                    fetch_progress_effect(model)
                        .map_or_else(EffectApplied::unchanged_done, |effect| {
                            effect.apply_on(model)
                        })
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model.remote_view.last_import_files_outcome.reset();
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::StartFindUntrackedFilesAccepted(start_find_untracked_files) => {
                debug_assert!(!model
                    .remote_view()
                    .last_find_untracked_files_outcome
                    .is_pending());
                model.last_error = None;
                let token = model
                    .remote_view
                    .last_find_untracked_files_outcome
                    .start_pending_now();
                let task = PendingTask::StartFindUntrackedFiles(start_find_untracked_files);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed(task)
            }
            Self::FindUntrackedFilesFinished { token, result } => match result {
                Ok(outcome) => {
                    if let Err(outcome) = model
                        .remote_view
                        .last_find_untracked_files_outcome
                        .finish_pending_with_value_now(token, outcome)
                    {
                        let effect_reconstructed = Self::FindUntrackedFilesFinished {
                            token,
                            result: Ok(outcome),
                        };
                        log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged_done();
                    }
                    fetch_progress_effect(model)
                        .map_or_else(EffectApplied::unchanged_done, |effect| {
                            effect.apply_on(model)
                        })
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model.remote_view.last_find_untracked_files_outcome.reset();
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::ErrorOccurred(err) => {
                model.last_error = Some(err);
                EffectApplied::maybe_changed_done()
            }
        }
    }
}

fn fetch_progress_effect(model: &Model) -> Option<Effect> {
    if model.remote_view().progress.is_pending() {
        log::warn!("Cannot fetch progress while pending");
        return None;
    }
    let effect = Effect::FetchProgressAccepted;
    Some(effect)
}
