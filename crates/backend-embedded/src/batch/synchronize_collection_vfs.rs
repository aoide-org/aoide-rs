// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use aoide_core::{
    media::content::{ContentPathConfig, VirtualFilePathConfig},
    track::Track,
    util::url::BaseUrl,
};
use aoide_core_api::{
    collection::LoadScope,
    filtering::StringPredicate,
    media::{SyncMode, tracker::DirTrackingStatus},
    track::find_unsynchronized::UnsynchronizedTrackEntity,
};
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::prelude::*;

/// Media source without a corresponding file.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UntrackedMediaSources {
    Keep,
    Purge,
}

/// Media source without a corresponding track entity.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OrphanedMediaSources {
    Keep,
    Purge,
}

/// Files without a corresponding media source.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UntrackedFiles {
    Skip,
    Find,
}

/// Current metadata revision of track differs from last synchronized revision.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnsynchronizedTracks {
    Skip,
    Find,
}

#[derive(Debug, Clone)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub max_depth: Option<usize>,
    pub sync_mode: SyncMode,
    pub import_track_config: ImportTrackConfig,
    pub untracked_media_sources: UntrackedMediaSources,
    pub orphaned_media_sources: OrphanedMediaSources,
    pub untracked_files: UntrackedFiles,
    pub unsynchronized_tracks: UnsynchronizedTracks,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Completion {
    #[default]
    Finished,
    Aborted,
}

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub completion: Completion,

    /// 1st step
    pub untrack_excluded_directories:
        Option<aoide_core_api::media::tracker::untrack_directories::Outcome>,

    /// 2nd step
    pub scan_directories: Option<aoide_core_api::media::tracker::scan_directories::Outcome>,

    /// 3rd step
    pub untrack_orphaned_directories:
        Option<aoide_core_api::media::tracker::untrack_directories::Outcome>,

    /// 4th step
    pub import_files: Option<aoide_core_api::media::tracker::import_files::Outcome>,

    /// 5th step (optional)
    ///
    /// This will also purge the corresponding track entities irreversibly!
    pub purge_untracked_media_sources:
        Option<aoide_core_api::media::source::purge_untracked::Outcome>,

    /// 6th step (optional)
    ///
    /// This will also purge the corresponding track entities irreversibly!
    pub purge_orphaned_media_sources:
        Option<aoide_core_api::media::source::purge_orphaned::Outcome>,

    /// 7th step (optional/informational)
    pub find_untracked_files: Option<aoide_core_api::media::tracker::find_untracked_files::Outcome>,

    /// 8th step (optional/informational)
    pub find_unsynchronized_tracks: Option<Vec<UnsynchronizedTrackEntity>>,
}

pub type Result = crate::Result<Outcome>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Step1UntrackExcludedDirectories,
    Step2ScanDirectories(aoide_usecases::media::tracker::scan_directories::ProgressEvent),
    Step3UntrackOrphanedDirectories,
    Step4ImportFiles(aoide_usecases::media::tracker::import_files::ProgressEvent),
    Step5PurgeUntrackedMediaSources,
    Step6PurgeOrphanedMediaSources,
    Step7FindUntrackedFiles(aoide_usecases::media::tracker::find_untracked_files::ProgressEvent),
    Step8FindUnsynchronizedTracks,
}

#[allow(clippy::too_many_lines)] // TODO
pub async fn synchronize_collection_vfs<InterceptImportedTrackFn, ReportProgressFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: Params,
    intercept_imported_track_fn: InterceptImportedTrackFn,
    mut report_progress_fn: ReportProgressFn,
    abort_flag: Arc<AtomicBool>,
) -> Result
where
    InterceptImportedTrackFn: Fn(Track) -> Track + Clone + Send + 'static,
    ReportProgressFn: FnMut(Progress) + Clone + Send + 'static,
{
    let Params {
        root_url,
        max_depth,
        sync_mode,
        import_track_config,
        untracked_media_sources,
        orphaned_media_sources,
        untracked_files,
        unsynchronized_tracks,
    } = params;
    let collection =
        crate::collection::load_one(db_gatekeeper, collection_uid.clone(), LoadScope::Entity)
            .await?
            .entity
            .raw
            .body;
    let excluded_content_paths = match collection.media_source_config.content_path {
        ContentPathConfig::VirtualFilePath(VirtualFilePathConfig { excluded_paths, .. }) => {
            excluded_paths
        }
        _ => vec![],
    };
    let mut outcome = Outcome::default();
    // 1st step: Untrack excluded directories
    report_progress_fn(Progress::Step1UntrackExcludedDirectories);
    if !excluded_content_paths.is_empty() {
        let untrack_excluded_directories_params =
            aoide_core_api::media::tracker::untrack_directories::Params {
                root_url: root_url.clone(),
                paths:
                    aoide_core_api::media::tracker::untrack_directories::PathsParam::SubDirectories(
                        excluded_content_paths.clone(),
                    ),
                status: None,
            };
        outcome.untrack_excluded_directories = Some(
            crate::media::tracker::untrack_directories(
                db_gatekeeper,
                collection_uid.clone(),
                untrack_excluded_directories_params,
            )
            .await?,
        );
    }
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    // 2nd step: Scan directories
    let scan_directories_params = aoide_core_api::media::tracker::scan_directories::Params {
        root_url: root_url.clone(),
        excluded_paths: excluded_content_paths.clone(),
        max_depth,
    };
    outcome.scan_directories = Some({
        let mut report_progress_fn = report_progress_fn.clone();
        let step_outcome = crate::media::tracker::scan_directories(
            db_gatekeeper,
            collection_uid.clone(),
            scan_directories_params,
            move |event| report_progress_fn(Progress::Step2ScanDirectories(event)),
            Arc::clone(&abort_flag),
        )
        .await?;
        if matches!(
            step_outcome.completion,
            aoide_core_api::media::tracker::Completion::Aborted
        ) {
            outcome.completion = Completion::Aborted;
        }
        step_outcome
    });
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    if abort_flag.load(Ordering::Relaxed) {
        outcome.completion = Completion::Aborted;
        return Ok(outcome);
    }
    #[cfg(feature = "tokio")]
    tokio::task::yield_now().await;
    // 3rd step: Untrack orphaned directories
    report_progress_fn(Progress::Step3UntrackOrphanedDirectories);
    let untrack_orphaned_directories_params =
        aoide_core_api::media::tracker::untrack_directories::Params {
            root_url: root_url.clone(),
            paths: aoide_core_api::media::tracker::untrack_directories::PathsParam::RootDirectory,
            status: Some(DirTrackingStatus::Orphaned),
        };
    outcome.untrack_orphaned_directories = Some(
        crate::media::tracker::untrack_directories(
            db_gatekeeper,
            collection_uid.clone(),
            untrack_orphaned_directories_params,
        )
        .await?,
    );
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    if abort_flag.load(Ordering::Relaxed) {
        outcome.completion = Completion::Aborted;
        return Ok(outcome);
    }
    #[cfg(feature = "tokio")]
    tokio::task::yield_now().await;
    // 4th step: Import files
    let import_files_params = aoide_core_api::media::tracker::import_files::Params {
        root_url: root_url.clone(),
        sync_mode,
    };
    outcome.import_files = Some({
        let mut report_progress_fn = report_progress_fn.clone();
        let step_outcome = crate::media::tracker::import_files(
            db_gatekeeper,
            collection_uid.clone(),
            import_files_params,
            import_track_config,
            intercept_imported_track_fn,
            move |event| report_progress_fn(Progress::Step4ImportFiles(event)),
            Arc::clone(&abort_flag),
        )
        .await?;
        if matches!(
            step_outcome.completion,
            aoide_core_api::media::tracker::Completion::Aborted
        ) {
            outcome.completion = Completion::Aborted;
        }
        step_outcome
    });
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    if abort_flag.load(Ordering::Relaxed) {
        outcome.completion = Completion::Aborted;
        return Ok(outcome);
    }
    #[cfg(feature = "tokio")]
    tokio::task::yield_now().await;
    // 5th step: Purge untracked media sources (optional)
    report_progress_fn(Progress::Step5PurgeUntrackedMediaSources);
    match untracked_media_sources {
        UntrackedMediaSources::Keep => (),
        UntrackedMediaSources::Purge => {
            let params = aoide_core_api::media::source::purge_untracked::Params {
                root_url: root_url.clone(),
            };
            outcome.purge_untracked_media_sources = Some(
                crate::media::source::purge_untracked(
                    db_gatekeeper,
                    collection_uid.clone(),
                    params,
                )
                .await?,
            );
        }
    }
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    if abort_flag.load(Ordering::Relaxed) {
        outcome.completion = Completion::Aborted;
        return Ok(outcome);
    }
    #[cfg(feature = "tokio")]
    tokio::task::yield_now().await;
    // 6th step: Purge orphaned media sources (optional)
    report_progress_fn(Progress::Step6PurgeOrphanedMediaSources);
    match orphaned_media_sources {
        OrphanedMediaSources::Keep => (),
        OrphanedMediaSources::Purge => {
            let params = aoide_core_api::media::source::purge_orphaned::Params {
                root_url: root_url.clone(),
            };
            outcome.purge_orphaned_media_sources = Some(
                crate::media::source::purge_orphaned(db_gatekeeper, collection_uid.clone(), params)
                    .await?,
            );
        }
    }
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    if abort_flag.load(Ordering::Relaxed) {
        outcome.completion = Completion::Aborted;
        return Ok(outcome);
    }
    #[cfg(feature = "tokio")]
    tokio::task::yield_now().await;
    // 7th step: Find untracked files (optional/informational)
    match untracked_files {
        UntrackedFiles::Skip => (),
        UntrackedFiles::Find => {
            let params = aoide_core_api::media::tracker::find_untracked_files::Params {
                root_url: root_url.clone(),
                excluded_paths: excluded_content_paths,
                max_depth,
            };
            outcome.find_untracked_files = Some({
                let mut report_progress_fn = report_progress_fn.clone();
                let step_outcome = crate::media::tracker::find_untracked_files(
                    db_gatekeeper,
                    collection_uid.clone(),
                    params,
                    move |event| report_progress_fn(Progress::Step7FindUntrackedFiles(event)),
                    Arc::clone(&abort_flag),
                )
                .await?;
                if matches!(
                    step_outcome.completion,
                    aoide_core_api::media::tracker::Completion::Aborted
                ) {
                    outcome.completion = Completion::Aborted;
                }
                step_outcome
            });
        }
    }
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    if abort_flag.load(Ordering::Relaxed) {
        outcome.completion = Completion::Aborted;
        return Ok(outcome);
    }
    #[cfg(feature = "tokio")]
    tokio::task::yield_now().await;
    // 8th step: Find unsynchronized tracks (optional/informational)
    report_progress_fn(Progress::Step8FindUnsynchronizedTracks);
    match unsynchronized_tracks {
        UnsynchronizedTracks::Skip => (),
        UnsynchronizedTracks::Find => {
            let content_path_predicate =
                root_url.map(|root_url| StringPredicate::StartsWith(root_url.to_string().into()));
            let params = aoide_core_api::track::find_unsynchronized::Params {
                content_path_predicate,
                resolve_url_from_content_path: None,
            };
            outcome.find_unsynchronized_tracks = Some(
                crate::track::find_unsynchronized(
                    db_gatekeeper,
                    collection_uid.clone(),
                    params,
                    Default::default(),
                )
                .await?,
            );
        }
    }
    Ok(outcome)
}
