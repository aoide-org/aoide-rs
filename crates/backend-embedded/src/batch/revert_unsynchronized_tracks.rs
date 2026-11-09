// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discard all modifications of unsynchronized tracks by restoring them from their files.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use diesel::Connection as _;

use aoide_core::{TrackEntity, media::content::ContentPath, track::EntityHeader};
use aoide_core_api::Pagination;
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::track::update::RevertOutcome;

use crate::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    #[default]
    Finished,
    Aborted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    /// The number of unsynchronized tracks.
    pub total: usize,

    /// The number of tracks that have been visited.
    pub finished: usize,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub completion: Completion,

    /// Tracks that have been restored from their files and are synchronized.
    pub reverted: Vec<TrackEntity>,

    /// Tracks that could not be restored and are still unsynchronized, e.g.
    /// if the file is missing.
    pub failed: Vec<(EntityHeader, ContentPath<'static>, String)>,
}

/// Restore all unsynchronized tracks from their files.
///
/// Each track is restored in a separate transaction. Failures of individual
/// tracks are collected in the outcome and don't abort the batch operation.
///
/// Files are never written.
pub async fn revert_unsynchronized_tracks<ReportProgressFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    import_config: ImportTrackConfig,
    mut report_progress_fn: ReportProgressFn,
    abort_flag: Arc<AtomicBool>,
) -> Result<Outcome>
where
    ReportProgressFn: FnMut(Progress),
{
    let unsynchronized_tracks = crate::track::find_unsynchronized(
        db_gatekeeper,
        collection_uid.clone(),
        Default::default(),
        Pagination::default(),
    )
    .await?;
    let mut progress = Progress {
        total: unsynchronized_tracks.len(),
        finished: 0,
    };
    log::info!(
        "Reverting {total} unsynchronized track(s)",
        total = progress.total
    );
    report_progress_fn(progress);

    let mut outcome = Outcome::default();
    for unsynchronized_track in unsynchronized_tracks {
        if abort_flag.load(Ordering::Relaxed) {
            log::info!(
                "Aborting revert of unsynchronized tracks after {finished} of {total} track(s)",
                finished = progress.finished,
                total = progress.total
            );
            outcome.completion = Completion::Aborted;
            return Ok(outcome);
        }
        let entity_hdr = unsynchronized_track.raw.hdr;
        let content_path = unsynchronized_track.raw.body.content_link.path;
        match revert_one(
            db_gatekeeper,
            collection_uid.clone(),
            entity_hdr.clone(),
            import_config.clone(),
        )
        .await
        {
            Ok(RevertOutcome::Reverted(entity)) => {
                outcome.reverted.push(entity);
            }
            Ok(RevertOutcome::Rejected(conflict)) => {
                log::info!("Skipped revert of track {entity_hdr:?}: {conflict:?}");
                outcome.failed.push((
                    entity_hdr,
                    content_path,
                    "The track has been modified in the meantime.".to_owned(),
                ));
            }
            Err(err) => {
                log::warn!("Failed to revert track {entity_hdr:?}: {err}");
                outcome
                    .failed
                    .push((entity_hdr, content_path, err.to_string()));
            }
        }
        progress.finished += 1;
        report_progress_fn(progress);
    }
    Ok(outcome)
}

async fn revert_one(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    expected_entity_hdr: EntityHeader,
    import_config: ImportTrackConfig,
) -> Result<RevertOutcome> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::track::update::revert_one(
                    connection,
                    &collection_uid,
                    &expected_entity_hdr,
                    &import_config,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
