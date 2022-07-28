// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_media::io::import::ImportTrackConfig;

use aoide_core::util::url::BaseUrl;

use aoide_core_api::{
    filtering::StringPredicate,
    media::{tracker::DirTrackingStatus, SyncMode},
    track::find_unsynchronized::UnsynchronizedTrackEntity,
};

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub max_depth: Option<usize>,
    pub sync_mode: Option<SyncMode>,
    pub import_track_config: ImportTrackConfig,
    pub purge_untracked_media_sources: bool,
    pub purge_orphaned_media_sources: bool,
    pub find_untracked_files: bool,
    pub find_unsynchronized_tracks: bool,
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
    pub scan_directories: Option<aoide_core_api::media::tracker::scan_directories::Outcome>,

    /// 2nd step
    pub untrack_orphaned_directories:
        Option<aoide_core_api::media::tracker::untrack_directories::Outcome>,

    /// 3rd step
    pub import_files: Option<aoide_core_api::media::tracker::import_files::Outcome>,

    /// 4th step (optional)
    ///
    /// This will also purge the corresponding track entities irreversibly!
    pub purge_untracked_media_sources:
        Option<aoide_core_api::media::source::purge_untracked::Outcome>,

    /// 5th step (optional)
    ///
    /// This will also purge the corresponding track entities irreversibly!
    pub purge_orphaned_media_sources:
        Option<aoide_core_api::media::source::purge_orphaned::Outcome>,

    /// 6th step (optional/informational)
    pub find_untracked_files: Option<aoide_core_api::media::tracker::find_untracked_files::Outcome>,

    /// 7th step (optional/informational)
    pub find_unsynchronized_tracks: Option<Vec<UnsynchronizedTrackEntity>>,
}

pub type Result = crate::Result<Outcome>;

#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    Step1ScanDirectories(aoide_usecases::media::tracker::scan_directories::ProgressEvent),
    Step2UntrackOrphanedDirectories,
    Step3ImportFiles(aoide_usecases::media::tracker::import_files::ProgressEvent),
    Step4PurgeUntrackedMediaSources,
    Step5PurgeOrphanedMediaSources,
    Step6FindUntrackedFiles(aoide_usecases::media::tracker::find_untracked_files::ProgressEvent),
    Step7FindUnsynchronizedTracks,
}

pub async fn rescan_collection_vfs<P>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: Params,
    mut report_progress_fn: P,
) -> Result
where
    P: FnMut(Progress) + Clone + Send + 'static,
{
    let Params {
        root_url,
        max_depth,
        sync_mode,
        import_track_config,
        purge_untracked_media_sources,
        purge_orphaned_media_sources,
        find_untracked_files,
        find_unsynchronized_tracks,
    } = params;
    let mut outcome = Outcome::default();
    // 1st step: Scan directories
    let scan_directories_params = aoide_core_api::media::tracker::scan_directories::Params {
        root_url: root_url.clone(),
        max_depth,
    };
    outcome.scan_directories = Some({
        let mut report_progress_fn = report_progress_fn.clone();
        let step_outcome = crate::media::tracker::scan_directories(
            db_gatekeeper,
            collection_uid.clone(),
            scan_directories_params,
            move |event| report_progress_fn(Progress::Step1ScanDirectories(event)),
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
    // 2nd step: Untrack orphaned directories
    report_progress_fn(Progress::Step2UntrackOrphanedDirectories);
    let untrack_orphaned_directories_params =
        aoide_core_api::media::tracker::untrack_directories::Params {
            root_url: root_url.clone(),
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
    // 3rd step: Import files
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
            move |event| report_progress_fn(Progress::Step3ImportFiles(event)),
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
    // 4th step: Purge untracked media sources (optional)
    report_progress_fn(Progress::Step4PurgeUntrackedMediaSources);
    if purge_untracked_media_sources {
        let params = aoide_core_api::media::source::purge_untracked::Params {
            root_url: root_url.clone(),
        };
        outcome.purge_untracked_media_sources = Some(
            crate::media::source::purge_untracked(db_gatekeeper, collection_uid.clone(), params)
                .await?,
        );
    }
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    // 5th step: Purge orphaned media sources (optional)
    report_progress_fn(Progress::Step5PurgeOrphanedMediaSources);
    if purge_orphaned_media_sources {
        let params = aoide_core_api::media::source::purge_orphaned::Params {
            root_url: root_url.clone(),
        };
        outcome.purge_orphaned_media_sources = Some(
            crate::media::source::purge_orphaned(db_gatekeeper, collection_uid.clone(), params)
                .await?,
        );
    }
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    // 6th step: Find untracked files (optional/informational)
    if find_untracked_files {
        let params = aoide_core_api::media::tracker::find_untracked_files::Params {
            root_url: root_url.clone(),
            max_depth,
        };
        outcome.find_untracked_files = Some({
            let mut report_progress_fn = report_progress_fn.clone();
            let step_outcome = crate::media::tracker::find_untracked_files(
                db_gatekeeper,
                collection_uid.clone(),
                params,
                move |event| report_progress_fn(Progress::Step6FindUntrackedFiles(event)),
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
    if matches!(outcome.completion, Completion::Aborted) {
        return Ok(outcome);
    }
    // 7th step: Find unsynchronized tracks (optional/informational)
    report_progress_fn(Progress::Step7FindUnsynchronizedTracks);
    if find_unsynchronized_tracks {
        let content_path_predicate =
            root_url.map(|root_url| StringPredicate::StartsWith(root_url.to_string()));
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
    Ok(outcome)
}
