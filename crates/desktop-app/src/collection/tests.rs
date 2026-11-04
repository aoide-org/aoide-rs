// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Duration;

use aoide_core_api::collection::{MediaSourceSummary, PlaylistSummary, TrackSummary};

use aoide_core::{
    collection::MediaSourceConfig,
    media::content::{ContentPathConfig, VirtualFilePathConfig},
    util::url::BaseUrl,
};
use aoide_storage_sqlite::connection::{self, Storage, pool};

use super::*;

const MUSIC_DIR: &str = "/music/";

fn new_entity() -> Entity {
    let collection = Collection {
        title: "Test".into(),
        notes: None,
        kind: None,
        color: None,
        media_source_config: MediaSourceConfig {
            content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                root_url: BaseUrl::try_from(Url::from_directory_path(MUSIC_DIR).unwrap()).unwrap(),
                excluded_paths: Vec::new(),
            }),
        },
    };
    Entity::new(
        aoide_core::collection::EntityHeader::initial_random(),
        collection,
    )
}

fn new_summary() -> Summary {
    Summary {
        media_sources: MediaSourceSummary { total_count: 0 },
        playlists: PlaylistSummary { total_count: 0 },
        tracks: TrackSummary { total_count: 0 },
    }
}

fn commission_environment() -> Arc<Environment> {
    let config = aoide_backend_embedded::storage::DatabaseConfig {
        connection: connection::Config {
            storage: Storage::InMemory,
            pool: pool::Config {
                max_size: 1.try_into().unwrap(),
                gatekeeper: pool::gatekeeper::Config {
                    acquire_read_timeout_millis: 10_000.try_into().unwrap(),
                    acquire_write_timeout_millis: 10_000.try_into().unwrap(),
                },
            },
        },
        migrate_schema: None,
    };
    Arc::new(Environment::commission(&config).unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn restoring_the_same_music_dir_must_not_replace_a_running_synchronization() {
    let rt = tokio::runtime::Handle::current();
    let env = commission_environment();
    let shared_state = SharedState::new(State::Ready {
        entity: new_entity(),
        summary: new_summary(),
    });
    let (_, task) = shared_state.spawn_synchronizing_vfs_task(&rt, &env);
    task.unwrap();

    // The collection feeds its music directory back into the settings.
    let reaction = shared_state
        .spawn_restoring_from_music_directory_task(
            &rt,
            &env,
            None,
            DirPath::from_owned(MUSIC_DIR.into()),
            RestoreEntityStrategy::Load,
            NestedMusicDirectoriesStrategy::Deny,
        )
        .unwrap();

    assert!(matches!(
        reaction,
        SpawnRestoringFromMusicDirectoryTaskReaction::Unchanged
    ));
    assert!(matches!(
        *shared_state.read(),
        State::SynchronizingVfs { .. }
    ));
}

#[tokio::test]
async fn outdated_result_of_restoring_should_be_ignored_while_another_task_is_pending() {
    let running_task = tokio::spawn(std::future::pending::<()>());
    let context = RestoringFromMusicDirectoryContext {
        kind: None,
        music_dir: DirPath::from_owned(MUSIC_DIR.into()),
        restore_entity: RestoreEntityStrategy::Load,
        nested_music_dirs: NestedMusicDirectoriesStrategy::Deny,
    };
    let mut state = State::RestoringFromMusicDirectory {
        context: context.clone(),
        state: RestoringFromMusicDirectoryState::Pending {
            since: Instant::now(),
            task: running_task.abort_handle(),
        },
    };
    let outdated = RestoringFromMusicDirectoryTaskContinuation {
        context,
        pending_since: Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
    };

    let effect = state
        .continue_after_restoring_from_music_directory_task_joined(JoinedTask::Cancelled, outdated);

    assert!(matches!(effect, ActionEffect::Unchanged));
    assert!(matches!(
        state,
        State::RestoringFromMusicDirectory {
            state: RestoringFromMusicDirectoryState::Pending { .. },
            ..
        }
    ));
    running_task.abort();
}

#[tokio::test]
async fn outdated_result_of_loading_should_be_ignored_while_another_task_is_pending() {
    let running_task = tokio::spawn(std::future::pending::<()>());
    let context = LoadingFromDatabaseContext {
        entity_uid: new_entity().hdr.uid.clone(),
        loaded_before: None,
    };
    let mut state = State::LoadingFromDatabase {
        context: context.clone(),
        state: LoadingFromDatabaseState::Pending {
            since: Instant::now(),
            task: running_task.abort_handle(),
        },
    };
    let outdated = LoadingFromDatabaseTaskContinuation {
        context,
        pending_since: Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
    };

    let effect =
        state.continue_after_loading_from_database_task_joined(JoinedTask::Cancelled, outdated);

    assert!(matches!(effect, ActionEffect::Unchanged));
    assert!(matches!(
        state,
        State::LoadingFromDatabase {
            state: LoadingFromDatabaseState::Pending { .. },
            ..
        }
    ));
    running_task.abort();
}
