// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Discard all modifications of unsynchronized tracks by restoring them from their files.

use std::sync::{Arc, atomic::AtomicBool};

use aoide_core::{TrackEntity, media::content::ContentPath, track::EntityHeader};
use aoide_core_api::Pagination;
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::track::update::RevertOutcome;

use crate::prelude::*;

pub use super::{Completion, Progress};

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
    report_progress_fn: ReportProgressFn,
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
    log::info!(
        "Reverting {total} unsynchronized track(s)",
        total = unsynchronized_tracks.len()
    );
    let mut outcome = Outcome::default();
    outcome.completion = super::run_batch(
        "revert of unsynchronized tracks",
        unsynchronized_tracks,
        |unsynchronized_track| {
            let entity_hdr = unsynchronized_track.raw.hdr;
            let content_path = unsynchronized_track.raw.body.content_link.path;
            let reverted = revert_one(
                db_gatekeeper,
                collection_uid.clone(),
                entity_hdr.clone(),
                import_config.clone(),
            );
            async move { (entity_hdr, content_path, reverted.await) }
        },
        |(entity_hdr, content_path, result)| match result {
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
        },
        report_progress_fn,
        &abort_flag,
    )
    .await;
    Ok(outcome)
}

async fn revert_one(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    expected_entity_hdr: EntityHeader,
    import_config: ImportTrackConfig,
) -> Result<RevertOutcome> {
    db_gatekeeper
        .spawn_blocking_write_transaction(move |connection| {
            aoide_usecases_sqlite::track::update::revert_one(
                connection,
                &collection_uid,
                &expected_entity_hdr,
                &import_config,
            )
        })
        .await
}
