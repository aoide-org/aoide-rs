// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Editing of single tracks.

use std::sync::{Arc, atomic::AtomicBool};

use aoide::{
    CollectionUid, Track, TrackEntity, TrackUid,
    backend_embedded::media::{predefined_export_track_config, predefined_import_track_config},
    desktop_app::ActionEffect,
    storage_sqlite::connection::pool::gatekeeper::Gatekeeper,
    track::EntityHeader,
    usecases::track::update::{ExportParams, Params},
};

// Re-exports
pub use aoide::usecases::track::update::{Conflict, Outcome};

use super::{EventEmitter, Library, ui::MultiTrackEdit, update_tracks};

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
    /// Finished applying an edit directly in the track list.
    Applied {
        collection_uid: CollectionUid,
        entity_uid: TrackUid,
        result: anyhow::Result<update_tracks::Outcome>,
    },
}

#[must_use]
pub(crate) fn export_params() -> ExportParams {
    ExportParams {
        export_config: predefined_export_track_config(),
        import_config: predefined_import_track_config(),
    }
}

#[must_use]
fn update_params(write_file: bool) -> Params {
    let export = write_file.then(export_params);
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

    /// Modify the current revision of a single track without writing the file.
    ///
    /// The track becomes unsynchronized if it has actually been modified.
    /// In contrast to [`Self::update_tracks`] this doesn't occupy the
    /// library and multiple edits could be pending. They are serialized
    /// by the database and don't conflict, because each of them loads
    /// the current revision.
    ///
    /// Emits [`Event::Applied`] when finished.
    pub fn apply_track_edit<E>(
        &self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
        entity_uid: TrackUid,
        edit: MultiTrackEdit,
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
                let result = apply_track_edit(
                    env.db_gatekeeper(),
                    collection_uid.clone(),
                    entity_uid.clone(),
                    edit,
                )
                .await;
                let event = Event::Applied {
                    collection_uid,
                    entity_uid,
                    result,
                };
                drop(event_emitter.emit_event(event.into()));
            }
        });
        ActionEffect::MaybeChanged
    }

    /// Replace a modified track in the track search results.
    ///
    /// See also: [`aoide::desktop_app::track::repo_search::SharedState::replace_fetched_entity()`]
    pub fn replace_track_search_result(&self, entity: TrackEntity) -> ActionEffect {
        self.shared_state
            .track_search
            .replace_fetched_entity(entity)
    }
}

async fn apply_track_edit(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    entity_uid: TrackUid,
    edit: MultiTrackEdit,
) -> anyhow::Result<update_tracks::Outcome> {
    let edit = Arc::new(edit);
    aoide::backend_embedded::batch::update_tracks::update_tracks(
        db_gatekeeper,
        collection_uid,
        vec![entity_uid],
        move |track: &mut Track| edit.apply(track),
        None,
        |_| {},
        Arc::new(AtomicBool::new(false)),
    )
    .await
    .map_err(Into::into)
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

#[cfg(test)]
mod tests {
    use std::num::{NonZeroU32, NonZeroU64};

    use aoide::{
        Collection, CollectionEntity, CollectionHeader, TrackBody, TrackHeader,
        collection::MediaSourceConfig,
        media::{
            self,
            content::{
                AudioContentMetadata, ContentLink, ContentPathConfig, ContentRevision,
                VirtualFilePathConfig,
            },
        },
        repo::{
            collection::EntityRepo as _, media::source::CollectionRepo as _, track::EntityRepo as _,
        },
        storage_sqlite::connection::{
            Storage,
            pool::{create_connection_pool, gatekeeper, get_pooled_connection},
        },
        util::{
            clock::{OffsetDateTimeMs, UtcDateTimeMs},
            url::BaseUrl,
        },
    };

    use crate::library::ui::{CustomTagsDelta, MultiTrackFields, TrackEditDraft};

    use super::*;

    /// The music directory of a test.
    ///
    /// Updating a track requires that the file exists and has not been modified.
    struct MusicDir(std::path::PathBuf);

    impl MusicDir {
        fn new(test_name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "aoide_track_edit_test_{pid}_{test_name}",
                pid = std::process::id()
            ));
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(path.join("track.mp3"), b"").unwrap();
            Self(path)
        }
    }

    impl Drop for MusicDir {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.0));
        }
    }

    /// A provisioned database with a single synchronized track.
    fn new_database(music_dir: &MusicDir) -> (Gatekeeper, CollectionUid, TrackUid) {
        let pool = create_connection_pool(
            &Storage::InMemory,
            NonZeroU32::MIN,
            aoide::repo_sqlite::initialize_connection,
        )
        .unwrap();
        let (collection_uid, entity_uid) = {
            let mut connection = get_pooled_connection(&pool).unwrap();
            aoide::repo_sqlite::initialize_database(&mut connection).unwrap();
            aoide::usecases_sqlite::database::migrate_schema(&mut connection).unwrap();
            let mut repo = aoide::repo_sqlite::Connection::new(&mut connection);
            let collection = Collection {
                title: "Test".into(),
                notes: None,
                kind: None,
                color: None,
                media_source_config: MediaSourceConfig {
                    content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                        root_url: BaseUrl::try_from(
                            url::Url::from_directory_path(&music_dir.0).unwrap(),
                        )
                        .unwrap(),
                        excluded_paths: Vec::new(),
                    }),
                },
            };
            let collection_entity =
                CollectionEntity::new(CollectionHeader::initial_random(), collection);
            let now = UtcDateTimeMs::now();
            let collection_id = repo
                .insert_collection_entity(now, &collection_entity)
                .unwrap();
            let media_source = media::Source {
                collected_at: OffsetDateTimeMs::from_utc(now),
                content: media::Content {
                    link: ContentLink {
                        path: "track.mp3".into(),
                        rev: ContentRevision::try_from_file(
                            &std::fs::File::open(music_dir.0.join("track.mp3")).unwrap(),
                        )
                        .unwrap(),
                    },
                    r#type: "audio/mpeg".parse().unwrap(),
                    metadata_flags: Default::default(),
                    metadata: AudioContentMetadata::default().into(),
                    digest: None,
                },
                artwork: Default::default(),
            };
            let media_source_id = repo
                .insert_media_source(collection_id, now, &media_source)
                .unwrap()
                .id;
            let mut track = Track::new_from_media_source(media_source);
            track.set_track_title("Title");
            let mut draft = TrackEditDraft::from_track(&track);
            draft.genres = "Ambient\nHouse\nTechno".to_owned();
            let track = draft.to_track(&track).unwrap();
            let hdr = TrackHeader::initial_random();
            let body = TrackBody {
                track,
                updated_at: now,
                last_synchronized_rev: Some(hdr.rev),
                content_url: None,
            };
            let entity = TrackEntity::new(hdr, body);
            repo.insert_track_entity(media_source_id, &entity).unwrap();
            (collection_entity.hdr.uid.clone(), entity.hdr.uid.clone())
        };
        let gatekeeper = Gatekeeper::new(
            pool,
            gatekeeper::Config {
                acquire_read_timeout_millis: NonZeroU64::new(10_000).unwrap(),
                acquire_write_timeout_millis: NonZeroU64::new(10_000).unwrap(),
            },
        );
        (gatekeeper, collection_uid, entity_uid)
    }

    async fn load(gatekeeper: &Gatekeeper, entity_uid: &TrackUid) -> TrackEntity {
        aoide::backend_embedded::track::load_one(gatekeeper, entity_uid.clone())
            .await
            .unwrap()
    }

    fn is_synchronized(entity: &TrackEntity) -> bool {
        entity.body.last_synchronized_rev == Some(entity.hdr.rev)
    }

    fn edit(fields: MultiTrackFields) -> MultiTrackEdit {
        MultiTrackEdit::new(fields, CustomTagsDelta::default()).unwrap()
    }

    fn genres_edit(genres: &str) -> MultiTrackEdit {
        edit(MultiTrackFields {
            genres: Some(genres.to_owned()),
            ..Default::default()
        })
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn reordering_genres_is_persisted_and_marks_the_track_as_modified() {
        let music_dir =
            MusicDir::new("reordering_genres_is_persisted_and_marks_the_track_as_modified");
        let (gatekeeper, collection_uid, entity_uid) = new_database(&music_dir);
        let initial = load(&gatekeeper, &entity_uid).await;
        assert!(is_synchronized(&initial));
        assert_eq!(
            "Ambient\nHouse\nTechno",
            TrackEditDraft::from_track(&initial.body.track).genres
        );

        let outcome = apply_track_edit(
            &gatekeeper,
            collection_uid.clone(),
            entity_uid.clone(),
            genres_edit("Techno\nAmbient\nHouse"),
        )
        .await
        .unwrap();
        assert_eq!(1, outcome.updated.len());

        let reordered = load(&gatekeeper, &entity_uid).await;
        assert_eq!(
            "Techno\nAmbient\nHouse",
            TrackEditDraft::from_track(&reordered.body.track).genres
        );
        assert!(!is_synchronized(&reordered));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn modified_tracks_are_counted_until_they_are_saved() {
        let music_dir = MusicDir::new("modified_tracks_are_counted_until_they_are_saved");
        let (gatekeeper, collection_uid, entity_uid) = new_database(&music_dir);
        let count = async || {
            crate::library::export_unsynchronized_tracks::count_unsynchronized_tracks(
                &gatekeeper,
                collection_uid.clone(),
            )
            .await
            .unwrap()
        };
        assert_eq!(0, count().await);

        apply_track_edit(
            &gatekeeper,
            collection_uid.clone(),
            entity_uid.clone(),
            genres_edit("Techno\nAmbient\nHouse"),
        )
        .await
        .unwrap();
        assert_eq!(1, count().await);

        // Applying another edit to the same track must not count it twice.
        apply_track_edit(
            &gatekeeper,
            collection_uid.clone(),
            entity_uid,
            genres_edit("House\nTechno\nAmbient"),
        )
        .await
        .unwrap();
        assert_eq!(1, count().await);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn editing_text_is_persisted_and_marks_the_track_as_modified() {
        let music_dir = MusicDir::new("editing_text_is_persisted_and_marks_the_track_as_modified");
        let (gatekeeper, collection_uid, entity_uid) = new_database(&music_dir);
        let outcome = apply_track_edit(
            &gatekeeper,
            collection_uid,
            entity_uid.clone(),
            edit(MultiTrackFields {
                title: Some("New title".to_owned()),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
        assert_eq!(1, outcome.updated.len());
        let edited = load(&gatekeeper, &entity_uid).await;
        assert_eq!(Some("New title"), edited.body.track.track_title());
        assert!(!is_synchronized(&edited));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn edit_without_changes_does_not_mark_the_track_as_modified() {
        let music_dir = MusicDir::new("edit_without_changes_does_not_mark_the_track_as_modified");
        let (gatekeeper, collection_uid, entity_uid) = new_database(&music_dir);
        let outcome = apply_track_edit(
            &gatekeeper,
            collection_uid,
            entity_uid.clone(),
            genres_edit("Ambient\nHouse\nTechno"),
        )
        .await
        .unwrap();
        assert_eq!(0, outcome.updated.len());
        assert_eq!(1, outcome.unchanged.len());
        assert!(is_synchronized(&load(&gatekeeper, &entity_uid).await));
    }
}
