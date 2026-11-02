// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Editing of single tracks.

use std::sync::Arc;

use aoide::{
    CollectionUid, Track, TrackEntity, TrackUid,
    backend_embedded::media::predefined_faceted_tag_mapping_config,
    desktop_app::ActionEffect,
    media_file::io::{export::ExportTrackConfig, import::ImportTrackConfig},
    track::{EntityHeader, tag::FACET_ID_COMMENT},
    usecases::track::update::{ExportParams, Params},
};

// Re-exports
pub use aoide::usecases::track::update::{Conflict, Outcome};

use super::{EventEmitter, Library};

#[derive(Debug)]
pub enum Event {
    Loaded {
        collection_uid: CollectionUid,
        entity_uid: TrackUid,
        result: anyhow::Result<TrackEntity>,
    },
    Saved {
        collection_uid: CollectionUid,
        entity_uid: TrackUid,
        result: anyhow::Result<Outcome>,
    },
}

#[must_use]
fn import_track_config() -> ImportTrackConfig {
    // Must match the configuration for synchronizing the music directory.
    ImportTrackConfig {
        faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
        ..Default::default()
    }
}

#[must_use]
fn export_track_config() -> ExportTrackConfig {
    ExportTrackConfig {
        faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
        // Custom tags are encoded as gig tags into the comment tag that
        // is decoded again when importing the file.
        encode_gigtags: Some(FACET_ID_COMMENT.clone()),
        ..Default::default()
    }
}

#[must_use]
fn update_params(write_file: bool) -> Params {
    let export = write_file.then(|| ExportParams {
        export_config: export_track_config(),
        import_config: import_track_config(),
    });
    Params { export }
}

impl Library {
    /// Load the current revision of a track for editing.
    ///
    /// Emits [`Event::Loaded`] when finished.
    pub fn load_track_for_edit<E>(
        &self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
        entity_uid: TrackUid,
    ) -> ActionEffect
    where
        E: EventEmitter + Clone + 'static,
    {
        let Some(collection_uid) = self.shared_state.collection.read().entity_uid().cloned() else {
            log::info!("No collection");
            return ActionEffect::Unchanged;
        };
        rt.spawn({
            let env = Arc::clone(self.env());
            let event_emitter = event_emitter.clone();
            async move {
                let result = aoide::backend_embedded::track::load_one(
                    env.db_gatekeeper(),
                    entity_uid.clone(),
                )
                .await
                .map_err(Into::into);
                let event = Event::Loaded {
                    collection_uid,
                    entity_uid,
                    result,
                };
                drop(event_emitter.emit_event(event.into()));
            }
        });
        ActionEffect::MaybeChanged
    }

    /// Save an edited track and optionally write the file.
    ///
    /// The `expected_entity_hdr` must match the header of the entity
    /// that has been loaded for editing.
    ///
    /// Saving is rejected with a [`Conflict`] if either the track or the
    /// file have been modified in the meantime.
    ///
    /// Emits [`Event::Saved`] when finished.
    pub fn save_edited_track<E>(
        &self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
        expected_entity_hdr: EntityHeader,
        track: Track,
        write_file: bool,
    ) -> ActionEffect
    where
        E: EventEmitter + Clone + 'static,
    {
        let Some(collection_uid) = self.shared_state.collection.read().entity_uid().cloned() else {
            log::info!("No collection");
            return ActionEffect::Unchanged;
        };
        rt.spawn({
            let env = Arc::clone(self.env());
            let event_emitter = event_emitter.clone();
            async move {
                let entity_uid = expected_entity_hdr.uid.clone();
                let result = save_edited_track(
                    &env,
                    collection_uid.clone(),
                    expected_entity_hdr,
                    track,
                    write_file,
                )
                .await;
                let event = Event::Saved {
                    collection_uid,
                    entity_uid,
                    result,
                };
                drop(event_emitter.emit_event(event.into()));
            }
        });
        ActionEffect::MaybeChanged
    }
}

async fn save_edited_track(
    env: &aoide::desktop_app::Environment,
    collection_uid: CollectionUid,
    expected_entity_hdr: EntityHeader,
    track: Track,
    write_file: bool,
) -> anyhow::Result<Outcome> {
    let (validated_input, invalidities) = aoide::usecases::track::validate_input(track)?;
    if !invalidities.is_empty() {
        // Imported tracks are often invalid to some extent and editing
        // must not be prevented.
        log::warn!(
            "Saving track {uid} with invalidities: {invalidities:?}",
            uid = expected_entity_hdr.uid
        );
    }
    aoide::backend_embedded::track::update_one(
        env.db_gatekeeper(),
        collection_uid,
        expected_entity_hdr,
        validated_input,
        update_params(write_file),
    )
    .await
    .map_err(Into::into)
}
