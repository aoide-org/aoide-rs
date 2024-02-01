// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Arc};

// Re-exports
pub(super) use aoide::desktop_app::collection::*;
use aoide::{
    backend_embedded::media::predefined_faceted_tag_mapping_config, desktop_app::Handle,
    media_file::io::import::ImportTrackConfig,
};

pub(super) fn synchronize_music_dir_task(
    handle: Handle,
    collection_state: Arc<ObservableState>,
    mut report_progress_fn: impl FnMut(Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>)
        + Clone
        + Send
        + 'static,
) -> impl Future<
    Output = anyhow::Result<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>,
> + Send
       + 'static {
    let mut collection_uid = None;
    collection_state.modify(|state| {
        collection_uid = state.entity_uid().map(ToOwned::to_owned);
        collection_uid.is_some() && state.reset_to_pending()
    });
    async move {
        let Some(collection_uid) = collection_uid else {
            anyhow::bail!("No collection");
        };
        log::debug!("Synchronizing collection with music directory...");
        let import_track_config = ImportTrackConfig {
            // TODO: Customize faceted tag mapping
            faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
            ..Default::default()
        };
        let res = {
            let mut report_progress_fn = report_progress_fn.clone();
            let report_progress_fn = move |progress| {
                report_progress_fn(Some(progress));
            };
            synchronize_vfs(
                &handle,
                collection_uid,
                import_track_config,
                report_progress_fn,
            )
            .await
        };
        report_progress_fn(None);
        log::debug!(
            "Synchronizing collection with music directory finished: {:?}",
            res
        );
        if let Err(err) = collection_state.refresh_from_db(&handle).await {
            log::warn!("Failed to refresh collection: {err}");
        }
        res
    }
}
