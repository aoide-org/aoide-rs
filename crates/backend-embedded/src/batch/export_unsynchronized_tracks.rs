// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Write the metadata of all unsynchronized tracks into their files.

use std::sync::{Arc, atomic::AtomicBool};

use diesel::Connection as _;

use aoide_core::{TrackEntity, media::content::ContentPath, track::EntityHeader};
use aoide_core_api::Pagination;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::track::update::{Conflict, ExportParams, Outcome as UpdateOutcome};

use crate::prelude::*;

pub use super::{Completion, Progress};

#[derive(Debug, Default)]
pub struct Outcome {
    pub completion: Completion,

    /// Tracks that have been written into their files and are synchronized.
    pub exported: Vec<TrackEntity>,

    /// Tracks that have not been written due to a conflict, i.e. if the
    /// file has been modified on disk since it has been imported.
    pub skipped: Vec<(EntityHeader, ContentPath<'static>, Conflict)>,

    /// Tracks that could not be written.
    pub failed: Vec<(EntityHeader, ContentPath<'static>, String)>,
}

/// Write the metadata of all unsynchronized tracks into their files.
///
/// Each track is exported and re-imported in a separate transaction.
/// Tracks with conflicts are skipped, i.e. files that have been modified
/// on disk are never overwritten.
///
/// Failures of individual tracks are collected in the outcome and don't
/// abort the batch operation.
pub async fn export_unsynchronized_tracks<ReportProgressFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    export: ExportParams,
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
        "Exporting {total} unsynchronized track(s)",
        total = unsynchronized_tracks.len()
    );
    let mut outcome = Outcome::default();
    outcome.completion = super::run_batch(
        "export of unsynchronized tracks",
        unsynchronized_tracks,
        |unsynchronized_track| {
            let entity_hdr = unsynchronized_track.raw.hdr;
            let content_path = unsynchronized_track.raw.body.content_link.path;
            let exported = export_one(
                db_gatekeeper,
                collection_uid.clone(),
                entity_hdr.clone(),
                export.clone(),
            );
            async move { (entity_hdr, content_path, exported.await) }
        },
        |(entity_hdr, content_path, result)| match result {
            Ok(UpdateOutcome::Updated { entity, exported }) => {
                debug_assert!(exported);
                outcome.exported.push(entity);
            }
            Ok(UpdateOutcome::Rejected(conflict)) => {
                log::info!("Skipped export of track {entity_hdr:?}: {conflict:?}");
                outcome.skipped.push((entity_hdr, content_path, conflict));
            }
            Err(err) => {
                log::warn!("Failed to export track {entity_hdr:?}: {err}");
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

async fn export_one(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    expected_entity_hdr: EntityHeader,
    export: ExportParams,
) -> Result<UpdateOutcome> {
    db_gatekeeper
        .spawn_blocking_write_task_flattened(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::track::update::export_one(
                    connection,
                    &collection_uid,
                    &expected_entity_hdr,
                    export,
                )
            })
        })
        .await
}
