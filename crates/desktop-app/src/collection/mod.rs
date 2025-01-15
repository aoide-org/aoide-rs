// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

use anyhow::anyhow;
use discro::{Observer, Publisher};
use tokio::task::AbortHandle;
use url::Url;

use aoide_backend_embedded::{
    batch::{
        self,
        synchronize_collection_vfs::{
            OrphanedMediaSources, Outcome, Progress, UnsynchronizedTracks, UntrackedFiles,
            UntrackedMediaSources,
        },
    },
    media::predefined_faceted_tag_mapping_config,
};
use aoide_core::{
    collection::{Collection, Entity, EntityUid, MediaSourceConfig},
    media::content::{ContentPath, ContentPathConfig, VirtualFilePathConfig},
    util::{fs::DirPath, url::BaseUrl},
};
use aoide_core_api::{
    collection::{EntityWithSummary, LoadScope, Summary},
    media::SyncMode,
};
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_repo::collection::{KindFilter, MediaSourceRootUrlFilter};

use crate::{
    modify_shared_state_action_effect, modify_shared_state_action_effect_result,
    modify_shared_state_result, ActionEffect, Environment, JoinedTask,
};

pub mod tasklet;

#[must_use]
pub const fn vfs_root_url(collection: &Collection) -> Option<&BaseUrl> {
    if let ContentPathConfig::VirtualFilePath(VirtualFilePathConfig { root_url, .. }) =
        &collection.media_source_config.content_path
    {
        Some(root_url)
    } else {
        None
    }
}

#[must_use]
pub fn vfs_music_dir(collection: &Collection) -> Option<DirPath<'static>> {
    vfs_root_url(collection).and_then(|base_url| {
        base_url.to_file_path().map_or_else(
            |()| {
                log::warn!("URL is not a file path: {base_url}");
                None
            },
            |path_buf| Some(path_buf.into()),
        )
    })
}

// Always load a collection with the summary.
const ENTITY_LOAD_SCOPE: LoadScope = LoadScope::EntityWithSummary;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RestoreEntityStrategy {
    /// Only try to load an existing collection.
    #[default]
    Load,

    /// Create a new collection if no existing collection is found.
    LoadOrCreateNew,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NestedMusicDirectoriesStrategy {
    /// Prevent the creation of new collections for a music directory
    /// if collections for sub-directories already exist. Instead
    /// select an existing collection with the closest match.
    #[default]
    Deny,

    /// Allow one collection per music directory without restrictions
    /// on nesting.
    Permit,
}

async fn refresh_entity_from_db(
    env: &Environment,
    entity_uid: EntityUid,
) -> anyhow::Result<Option<EntityWithSummary>> {
    aoide_backend_embedded::collection::try_load_one(
        env.db_gatekeeper(),
        entity_uid.clone(),
        ENTITY_LOAD_SCOPE,
    )
    .await
    .inspect(|entity_with_summary| {
        if entity_with_summary.is_some() {
            log::info!("Reloaded collection {entity_uid}");
        } else {
            log::warn!("Collection {entity_uid} not found");
        }
    })
    .map_err(Into::into)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoringFromMusicDirectoryContext {
    pub kind: Option<String>,
    pub music_dir: DirPath<'static>,
    pub restore_entity: RestoreEntityStrategy,
    pub nested_music_dirs: NestedMusicDirectoriesStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadingFromDatabaseContext {
    pub entity_uid: EntityUid,
    pub loaded_before: Option<Collection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronizingVfsContext {
    pub entity: Entity,
}

fn parse_music_dir_path(path: &Path) -> anyhow::Result<(BaseUrl, PathBuf)> {
    let root_url =
        BaseUrl::try_autocomplete_from(Url::from_directory_path(path).map_err(|()| {
            anyhow::anyhow!(
                "unrecognized music directory: {path}",
                path = path.display()
            )
        })?)?;
    let root_path = root_url
        .to_file_path()
        .map_err(|()| anyhow::anyhow!("invalid music directory"))?;
    Ok((root_url, root_path))
}

impl RestoringFromMusicDirectoryContext {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::too_many_lines)] // TODO
    pub async fn restore(self, env: &Environment) -> anyhow::Result<State> {
        let Self {
            kind,
            music_dir,
            restore_entity,
            nested_music_dirs,
        } = self;
        let (root_url, music_dir) = parse_music_dir_path(&music_dir)?;
        // Search for an existing collection with a root directory
        // that contains the music directory.
        let media_source_root_url_filter = match nested_music_dirs {
            NestedMusicDirectoriesStrategy::Permit => {
                MediaSourceRootUrlFilter::Equal(root_url.clone())
            }
            NestedMusicDirectoriesStrategy::Deny => {
                MediaSourceRootUrlFilter::PrefixOf(root_url.clone())
            }
        };
        let kind_filter = kind.clone().map(|kind| KindFilter::Equal(kind.into()));
        let candidates = aoide_backend_embedded::collection::load_all(
            env.db_gatekeeper(),
            kind_filter.clone(),
            Some(media_source_root_url_filter),
            ENTITY_LOAD_SCOPE,
            None,
        )
        .await?;
        log::info!(
            "Found {num_candidates} existing collection candidate(s) for selection",
            num_candidates = candidates.len()
        );
        let mut selected_candidate: Option<EntityWithSummary> = None;
        for candidate in candidates {
            if let Some(candidate_vfs_music_dir) = vfs_music_dir(&candidate.entity.body) {
                if music_dir.starts_with(&*candidate_vfs_music_dir) {
                    if let Some(selected_candidate) = &mut selected_candidate {
                        if candidate_vfs_music_dir.starts_with(
                            &*vfs_music_dir(&selected_candidate.entity.body).expect("some"),
                        ) {
                            // Prefer the closest/longest match
                            *selected_candidate = candidate;
                            continue;
                        }
                    } else {
                        selected_candidate = Some(candidate);
                        continue;
                    }
                }
            }
            log::info!(
                "Skipping collection {uid} \"{title}\"",
                uid = candidate.entity.hdr.uid,
                title = candidate.entity.body.title,
            );
        }
        if let Some(entity_with_summary) = selected_candidate {
            log::info!(
                "Restored collection {uid} \"{title}\"",
                uid = entity_with_summary.entity.hdr.uid,
                title = entity_with_summary.entity.body.title,
            );
            let state = State::loading_from_database_succeeded(entity_with_summary);
            return Ok(state);
        }
        if !matches!(nested_music_dirs, NestedMusicDirectoriesStrategy::Permit) {
            // Search for an existing collection with a root directory
            // that is a child of the music directory.
            let candidates = aoide_backend_embedded::collection::load_all(
                env.db_gatekeeper(),
                kind_filter,
                Some(MediaSourceRootUrlFilter::Prefix(root_url.clone())),
                ENTITY_LOAD_SCOPE,
                None,
            )
            .await?;
            let context = RestoringFromMusicDirectoryContext {
                kind,
                music_dir: music_dir.into(),
                restore_entity,
                nested_music_dirs,
            };
            let state = RestoringFromMusicDirectoryState::Finished(
                RestoringFromMusicDirectoryFinishedState::NestedDirectoriesConflict { candidates },
            );
            return Ok(State::RestoringFromMusicDirectory { context, state });
        }
        // No matching entity found.
        match restore_entity {
            RestoreEntityStrategy::Load => {
                let context = RestoringFromMusicDirectoryContext {
                    kind,
                    music_dir: music_dir.into(),
                    restore_entity,
                    nested_music_dirs,
                };
                let state = RestoringFromMusicDirectoryState::Finished(
                    RestoringFromMusicDirectoryFinishedState::Failed {
                        error: RestoringFromMusicDirectoryError::EntityNotFound,
                    },
                );
                Ok(State::RestoringFromMusicDirectory { context, state })
            }
            RestoreEntityStrategy::LoadOrCreateNew => {
                // Create a new collection
                let new_collection = Collection {
                    title: music_dir.display().to_string(),
                    media_source_config: MediaSourceConfig {
                        content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                            root_url,
                            excluded_paths: vec![],
                        }),
                    },
                    kind,
                    notes: None,
                    color: None,
                };
                let entity_uid =
                    aoide_backend_embedded::collection::create(env.db_gatekeeper(), new_collection)
                        .await?
                        .raw
                        .hdr
                        .uid;
                // Reload the newly created entity with its summary
                let entity_with_summary = aoide_backend_embedded::collection::load_one(
                    env.db_gatekeeper(),
                    entity_uid,
                    ENTITY_LOAD_SCOPE,
                )
                .await?;
                log::info!(
                    "Created collection {uid} \"{title}\"",
                    uid = entity_with_summary.entity.hdr.uid,
                    title = entity_with_summary.entity.body.title,
                );
                let state = State::loading_from_database_succeeded(entity_with_summary);
                Ok(state)
            }
        }
    }
}

#[derive(Debug)]
pub enum RestoringFromMusicDirectoryError {
    EntityNotFound,
    Other(String),
}

impl RestoringFromMusicDirectoryError {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::EntityNotFound => "entity not found",
            Self::Other(error) => error,
        }
    }
}

#[derive(Debug)]
pub enum RestoringFromMusicDirectoryState {
    Pending { since: Instant, task: AbortHandle },
    Finished(RestoringFromMusicDirectoryFinishedState),
}

#[derive(Debug)]
pub enum RestoringFromMusicDirectoryFinishedState {
    Failed {
        error: RestoringFromMusicDirectoryError,
    },
    NestedDirectoriesConflict {
        candidates: Vec<EntityWithSummary>,
    },
}

#[derive(Debug)]
pub enum LoadingFromDatabaseState {
    Pending { since: Instant, task: AbortHandle },
    Finished(LoadingFromDatabaseFinishedState),
}

#[derive(Debug)]
pub enum LoadingFromDatabaseFinishedState {
    Failed { error: String },
}

#[allow(clippy::large_enum_variant)] // Only exists temporarily for a limited duration.
#[derive(Debug)]
pub enum SynchronizingVfsState {
    Pending {
        since: Instant,
        task: SynchronizingVfsTask,
    },
    Finished(SynchronizingVfsFinishedState),
}

#[allow(clippy::large_enum_variant)] // Only exists temporarily for a limited duration.
#[derive(Debug, Clone)]
pub enum SynchronizingVfsFinishedState {
    Succeeded { outcome: Outcome },
    Failed { error: String },
    Aborted,
}

/// State of a single collection that is based on directory in the
/// local directory using a virtual file system (VFS) for content paths.
#[allow(clippy::large_enum_variant)] // That's fine for a single or very few instances.
#[derive(Debug, Default)]
pub enum State {
    #[default]
    Void,
    RestoringFromMusicDirectory {
        context: RestoringFromMusicDirectoryContext,
        state: RestoringFromMusicDirectoryState,
    },
    LoadingFromDatabase {
        context: LoadingFromDatabaseContext,
        state: LoadingFromDatabaseState,
    },
    SynchronizingVfs {
        context: SynchronizingVfsContext,
        state: SynchronizingVfsState,
    },
    Ready {
        entity: Entity,
        summary: Summary,
    },
}

#[derive(Debug)]
pub enum SpawnRestoringFromMusicDirectoryTaskReaction {
    Unchanged,
    SpawnedAndChanged(AbortHandle),
}

impl SpawnRestoringFromMusicDirectoryTaskReaction {
    pub const fn effect(&self) -> ActionEffect {
        match self {
            Self::Unchanged => ActionEffect::Unchanged,
            Self::SpawnedAndChanged(_) => ActionEffect::Changed,
        }
    }
}

impl State {
    #[must_use]
    pub const fn pending_since(&self) -> Option<Instant> {
        match self {
            Self::RestoringFromMusicDirectory {
                state: RestoringFromMusicDirectoryState::Pending { since, .. },
                ..
            }
            | Self::LoadingFromDatabase {
                state: LoadingFromDatabaseState::Pending { since, .. },
                ..
            }
            | Self::SynchronizingVfs {
                state: SynchronizingVfsState::Pending { since, .. },
                ..
            } => Some(*since),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending_since().is_some()
    }

    #[must_use]
    pub const fn is_synchronizing(&self) -> bool {
        matches!(self, Self::SynchronizingVfs { .. })
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, State::Ready { .. })
    }

    #[must_use]
    pub fn music_dir(&self) -> Option<DirPath<'_>> {
        match self {
            Self::Void
            | Self::LoadingFromDatabase {
                context:
                    LoadingFromDatabaseContext {
                        loaded_before: None,
                        ..
                    },
                ..
            } => None,
            Self::LoadingFromDatabase {
                context:
                    LoadingFromDatabaseContext {
                        loaded_before: Some(loaded_before),
                        ..
                    },
                ..
            } => vfs_music_dir(loaded_before),
            Self::SynchronizingVfs {
                context: SynchronizingVfsContext { entity },
                ..
            }
            | Self::Ready { entity, .. } => vfs_music_dir(&entity.body),
            Self::RestoringFromMusicDirectory {
                context: RestoringFromMusicDirectoryContext { music_dir, .. },
                ..
            } => Some(music_dir.borrowed()),
        }
    }

    #[must_use]
    pub fn entity_uid(&self) -> Option<&EntityUid> {
        self.entity_brief().map(|(entity_uid, _)| entity_uid)
    }

    #[must_use]
    pub fn entity_brief(&self) -> Option<(&EntityUid, Option<&Collection>)> {
        match self {
            Self::Void | Self::RestoringFromMusicDirectory { .. } => None,
            Self::LoadingFromDatabase {
                context:
                    LoadingFromDatabaseContext {
                        entity_uid,
                        loaded_before,
                    },
                ..
            } => Some((entity_uid, loaded_before.as_ref())),
            Self::SynchronizingVfs {
                context: SynchronizingVfsContext { entity },
                ..
            }
            | Self::Ready { entity, .. } => Some((&entity.hdr.uid, Some(&entity.body))),
        }
    }

    #[must_use]
    pub const fn entity_with_summary(&self) -> Option<(&Entity, &Summary)> {
        match self {
            Self::Ready { entity, summary } => Some((entity, summary)),
            _ => None,
        }
    }

    #[must_use]
    pub fn last_error(&self) -> Option<&str> {
        match self {
            Self::RestoringFromMusicDirectory {
                state:
                    RestoringFromMusicDirectoryState::Finished(
                        RestoringFromMusicDirectoryFinishedState::Failed { error },
                    ),
                ..
            } => Some(error.as_str()),
            Self::LoadingFromDatabase {
                state:
                    LoadingFromDatabaseState::Finished(LoadingFromDatabaseFinishedState::Failed {
                        error,
                    }),
                ..
            }
            | Self::SynchronizingVfs {
                state:
                    SynchronizingVfsState::Finished(SynchronizingVfsFinishedState::Failed { error }),
                ..
            } => Some(error.as_str()),
            _ => None,
        }
    }

    pub fn reset(&mut self) -> ActionEffect {
        if matches!(self, Self::Void) {
            return ActionEffect::Unchanged;
        }
        let reset = Self::Void;
        log::debug!("Resetting state: {self:?} -> {reset:?}");
        *self = reset;
        ActionEffect::Changed
    }

    /// Restore collection from a music directory.
    ///
    /// The operation is performed by a background task.
    ///
    /// If the method returns an error result the state has not been modified
    /// and is unchanged.
    #[allow(clippy::too_many_arguments)] // TODO
    pub fn spawn_restoring_from_music_directory_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        new_kind: Option<Cow<'_, str>>,
        new_music_dir: DirPath<'_>,
        restore_entity: RestoreEntityStrategy,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> anyhow::Result<SpawnRestoringFromMusicDirectoryTaskReaction> {
        match self {
            Self::Ready { entity, .. } => {
                // When set the `kind` controls the selection of collections by music directory.
                if new_kind.is_none() || new_kind.as_deref() == entity.body.kind.as_deref() {
                    let vfs_music_dir = vfs_music_dir(&entity.body);
                    if vfs_music_dir.as_ref() == Some(&new_music_dir) {
                        // Unchanged
                        log::debug!(
                            "Music directory unchanged and not updated: {music_dir}",
                            music_dir = new_music_dir.display()
                        );
                        return Ok(SpawnRestoringFromMusicDirectoryTaskReaction::Unchanged);
                    }
                }
            }
            Self::RestoringFromMusicDirectory {
                state:
                    RestoringFromMusicDirectoryState::Finished(
                        RestoringFromMusicDirectoryFinishedState::NestedDirectoriesConflict {
                            ..
                        },
                    ),
                context:
                    RestoringFromMusicDirectoryContext {
                        kind, music_dir, ..
                    },
                ..
            } => {
                // When set the `kind` controls the selection of collections by music directory.
                if (new_kind.is_none() || new_kind.as_deref() == kind.as_deref())
                    && music_dir.borrowed() == new_music_dir
                {
                    // No effect
                    log::debug!(
                        "Music directory unchanged and not updated: {music_dir}",
                        music_dir = new_music_dir.display()
                    );
                    return Ok(SpawnRestoringFromMusicDirectoryTaskReaction::Unchanged);
                }
            }
            _ => {
                // Proceed without any checks.
            }
        }

        let context = RestoringFromMusicDirectoryContext {
            kind: new_kind.map(Into::into),
            music_dir: new_music_dir.into_owned(),
            restore_entity,
            nested_music_dirs,
        };

        let pending_since = Instant::now();
        debug_assert_ne!(Some(pending_since), self.pending_since());
        let continuation = RestoringFromMusicDirectoryTaskContinuation {
            context: context.clone(),
            pending_since,
        };

        let worker_task = rt.spawn({
            let env = Arc::clone(env);
            let params = LoadStateFromDatabaseParams {
                context: Some(context.clone()),
                entity_uid: None,
            };
            async move { load_state_from_database(env, params).await }
        });
        let abort_worker_task = worker_task.abort_handle();
        let _supervisor_task = rt.spawn({
            let this = this.clone();
            async move {
                let worker_joined = JoinedTask::join(worker_task).await;
                let _ = this.continue_after_restoring_from_music_directory_task_joined(
                    worker_joined,
                    continuation,
                );
            }
        });

        *self = Self::RestoringFromMusicDirectory {
            context,
            state: RestoringFromMusicDirectoryState::Pending {
                since: pending_since,
                task: abort_worker_task.clone(),
            },
        };

        Ok(SpawnRestoringFromMusicDirectoryTaskReaction::SpawnedAndChanged(abort_worker_task))
    }

    fn spawn_loading_from_database_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (ActionEffect, Option<AbortHandle>) {
        let old_self = std::mem::replace(self, Self::Void);
        let context = match old_self {
            Self::Void => {
                // Nothing to do.
                return (ActionEffect::Unchanged, None);
            }
            Self::RestoringFromMusicDirectory {
                context,
                state:
                    RestoringFromMusicDirectoryState::Finished(
                        RestoringFromMusicDirectoryFinishedState::Failed { .. }
                        | RestoringFromMusicDirectoryFinishedState::NestedDirectoriesConflict {
                            ..
                        },
                    ),
            } => {
                let pending_since = Instant::now();
                debug_assert_ne!(Some(pending_since), self.pending_since());
                let continuation = RestoringFromMusicDirectoryTaskContinuation {
                    context: context.clone(),
                    pending_since,
                };

                let worker_task = rt.spawn({
                    let env = Arc::clone(env);
                    let params = LoadStateFromDatabaseParams {
                        entity_uid: None,
                        context: Some(context.clone()),
                    };
                    async move { load_state_from_database(env, params).await }
                });
                let abort_worker_task = worker_task.abort_handle();
                let _supervisor_task = rt.spawn({
                    let this = this.clone();
                    async move {
                        let joined = JoinedTask::join(worker_task).await;
                        let _ = this.continue_after_restoring_from_music_directory_task_joined(
                            joined,
                            continuation,
                        );
                    }
                });

                *self = Self::RestoringFromMusicDirectory {
                    context,
                    state: RestoringFromMusicDirectoryState::Pending {
                        since: pending_since,
                        task: abort_worker_task.clone(),
                    },
                };

                return (ActionEffect::Changed, Some(abort_worker_task));
            }
            Self::LoadingFromDatabase {
                state:
                    LoadingFromDatabaseState::Finished(LoadingFromDatabaseFinishedState::Failed {
                        ..
                    }),
                context,
            } => context,
            Self::Ready { entity, .. }
            | Self::SynchronizingVfs {
                state:
                    SynchronizingVfsState::Finished(
                        SynchronizingVfsFinishedState::Failed { .. }
                        | SynchronizingVfsFinishedState::Succeeded { .. }
                        | SynchronizingVfsFinishedState::Aborted { .. },
                    ),
                context: SynchronizingVfsContext { entity },
            } => LoadingFromDatabaseContext {
                entity_uid: entity.raw.hdr.uid,
                loaded_before: Some(entity.raw.body),
            },
            old_self => {
                // Restore old state and reject.
                *self = old_self;
                log::warn!("Illegal state for refreshing from database: {self:?}");
                return (ActionEffect::Unchanged, None);
            }
        };
        let abort_worker_task =
            self.spawn_loading_from_database_task_unchecked(this, rt, env, context);
        (ActionEffect::Changed, Some(abort_worker_task))
    }

    #[must_use]
    fn spawn_loading_from_database_task_unchecked(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        context: LoadingFromDatabaseContext,
    ) -> AbortHandle {
        debug_assert!(matches!(self, Self::Void));

        let params = LoadStateFromDatabaseParams {
            entity_uid: Some(context.entity_uid.clone()),
            context: None, // Omit checking the context.
        };
        let pending_since = Instant::now();
        let continuation = LoadingFromDatabaseTaskContinuation {
            context: context.clone(),
            pending_since,
        };

        let worker_task = rt.spawn({
            let env = Arc::clone(env);
            async move { load_state_from_database(env, params).await }
        });
        let abort_worker_task = worker_task.abort_handle();
        let _supervisor_task = rt.spawn({
            let this = this.clone();
            async move {
                let joined = JoinedTask::join(worker_task).await;
                let _ = this.continue_after_loading_from_database_task_joined(joined, continuation);
            }
        });

        *self = Self::LoadingFromDatabase {
            context,
            state: LoadingFromDatabaseState::Pending {
                since: pending_since,
                task: abort_worker_task.clone(),
            },
        };
        abort_worker_task
    }

    fn continue_after_restoring_from_music_directory_task_joined(
        &mut self,
        joined: JoinedTask<anyhow::Result<State>>,
        continuation: RestoringFromMusicDirectoryTaskContinuation,
    ) -> ActionEffect {
        let RestoringFromMusicDirectoryTaskContinuation {
            context: continuation_context,
            pending_since: continuation_pending_since,
        } = continuation;

        let Self::RestoringFromMusicDirectory {
            context,
            state:
                RestoringFromMusicDirectoryState::Pending {
                    since: pending_since,
                    task,
                },
        } = self
        else {
            log::warn!(
                    "State changed while restoring from music directory: current state {self:?}, continuation {continuation:?} - discarding {joined:?}",
                        continuation = RestoringFromMusicDirectoryTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                );
            return ActionEffect::Unchanged;
        };

        debug_assert!(task.is_finished());
        if *pending_since != continuation_pending_since || *context != continuation_context {
            log::warn!(
                        "State changed while restoring from music directory: current state {self:?}, continuation {continuation:?} - discarding {joined:?}",
                        continuation = RestoringFromMusicDirectoryTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                    );
            return ActionEffect::Unchanged;
        }

        match joined {
            JoinedTask::Completed(Ok(next_state)) => {
                log::debug!("Restored state from music directory: {next_state:?}");
                *self = next_state;
                ActionEffect::MaybeChanged
            }
            JoinedTask::Cancelled => {
                log::debug!("Restored state from music directory cancelled");
                *self = Self::Void;
                ActionEffect::Changed
            }
            JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                log::warn!("Failed to restore state from music directory: {err}");
                let error = RestoringFromMusicDirectoryError::Other(err.to_string());
                *self = Self::RestoringFromMusicDirectory {
                    context: continuation_context,
                    state: RestoringFromMusicDirectoryState::Finished(
                        RestoringFromMusicDirectoryFinishedState::Failed { error },
                    ),
                };
                ActionEffect::Changed
            }
        }
    }

    fn continue_after_loading_from_database_task_joined(
        &mut self,
        joined: JoinedTask<anyhow::Result<State>>,
        continuation: LoadingFromDatabaseTaskContinuation,
    ) -> ActionEffect {
        let LoadingFromDatabaseTaskContinuation {
            context: continuation_context,
            pending_since: continuation_pending_since,
        } = continuation;

        let Self::LoadingFromDatabase {
            context,
            state:
                LoadingFromDatabaseState::Pending {
                    since: pending_since,
                    task,
                },
        } = self
        else {
            log::warn!(
                    "State changed while loading from database: current {self:?}, continuation {continuation:?} - discarding {joined:?}",
                        continuation = LoadingFromDatabaseTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                );
            return ActionEffect::Unchanged;
        };
        debug_assert!(task.is_finished());
        if *pending_since != continuation_pending_since || *context != continuation_context {
            log::warn!(
                        "State changed while loading from database: current state {self:?}, continuation {continuation:?} - discarding {joined:?}",
                        continuation = LoadingFromDatabaseTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                    );
            return ActionEffect::Unchanged;
        }

        match joined {
            JoinedTask::Completed(Ok(next_state)) => {
                log::debug!("Loaded state from database: {next_state:?}");
                *self = next_state;
                ActionEffect::MaybeChanged
            }
            JoinedTask::Cancelled => {
                log::debug!("Loading state from database cancelled");
                *self = Self::Void;
                ActionEffect::Changed
            }
            JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                log::warn!("Failed to load state from database: {err}");
                let error = err.to_string();
                *self = Self::LoadingFromDatabase {
                    context: continuation_context,
                    state: LoadingFromDatabaseState::Finished(
                        LoadingFromDatabaseFinishedState::Failed { error },
                    ),
                };
                ActionEffect::Changed
            }
        }
    }

    #[must_use]
    fn loading_from_database_succeeded(entity_with_summary: EntityWithSummary) -> Self {
        let EntityWithSummary { entity, summary } = entity_with_summary;
        if let Some(summary) = summary {
            State::Ready { entity, summary }
        } else {
            // Should never happen
            let context = LoadingFromDatabaseContext {
                entity_uid: entity.raw.hdr.uid,
                loaded_before: Some(entity.raw.body),
            };
            let state =
                LoadingFromDatabaseState::Finished(LoadingFromDatabaseFinishedState::Failed {
                    error: "no summary".to_owned(),
                });
            Self::LoadingFromDatabase { context, state }
        }
    }

    fn spawn_synchronizing_vfs_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (ActionEffect, anyhow::Result<SynchronizingVfsTask>) {
        let old_self = std::mem::replace(self, Self::Void);
        let Self::Ready { entity, .. } = old_self else {
            // Restore old state and reject.
            let rejected = anyhow!("illegal state: {old_self:?}");
            *self = old_self;
            return (ActionEffect::Unchanged, Err(rejected));
        };

        let context = SynchronizingVfsContext { entity };

        let pending_since = Instant::now();
        let continuation = SynchronizingVfsTaskContinuation {
            context: context.clone(),
            pending_since,
        };

        let task = SynchronizingVfsTask::spawn(rt, Arc::clone(env), this.clone(), continuation);
        let state = SynchronizingVfsState::Pending {
            since: pending_since,
            task: task.clone(),
        };

        *self = Self::SynchronizingVfs { context, state };

        (ActionEffect::Changed, Ok(task))
    }

    fn continue_after_synchronizing_vfs_task_joined(
        &mut self,
        joined: JoinedTask<anyhow::Result<Outcome>>,
        continuation: SynchronizingVfsTaskContinuation,
    ) -> ActionEffect {
        let Self::SynchronizingVfs {
            state:
                SynchronizingVfsState::Pending {
                    since: pending_since,
                    task: _,
                },
            context,
        } = self
        else {
            log::info!("State changed while pending: current state {self:?}, continuation {continuation:?} - discarding {joined:?}");
            return ActionEffect::Unchanged;
        };
        let SynchronizingVfsTaskContinuation {
            context: continuation_context,
            pending_since: continuation_pending_since,
        } = continuation;
        if continuation_pending_since != *pending_since || continuation_context != *context {
            log::info!("State changed while pending: current state {self:?}, continuation {continuation:?} - discarding {joined:?}",
                continuation = SynchronizingVfsTaskContinuation {
                    context: continuation_context,
                    pending_since: continuation_pending_since,
                }
            );
            return ActionEffect::Unchanged;
        }

        let next_state = match joined {
            JoinedTask::Completed(Ok(outcome)) => {
                SynchronizingVfsState::Finished(SynchronizingVfsFinishedState::Succeeded {
                    outcome,
                })
            }
            JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                let error = err.to_string();
                SynchronizingVfsState::Finished(SynchronizingVfsFinishedState::Failed { error })
            }
            JoinedTask::Cancelled => {
                SynchronizingVfsState::Finished(SynchronizingVfsFinishedState::Aborted)
            }
        };
        *self = Self::SynchronizingVfs {
            state: next_state,
            context: continuation_context,
        };

        ActionEffect::Changed
    }

    /// Map an URL to the corresponding content path.
    ///
    /// Example: Map a local file path to a content path within the collection
    /// to exclude it from synchronization.
    ///
    /// Returns `None` if no collection is available or if the URL
    /// has no corresponding content path within the collection.
    pub fn resolve_content_path_from_url(
        &self,
        content_url: &Url,
    ) -> anyhow::Result<Option<ContentPath<'static>>> {
        let Some((_, Some(collection))) = self.entity_brief() else {
            return Ok(None);
        };
        let resolver = collection.media_source_config.content_path.resolver();
        resolver
            .resolve_path_from_url(content_url)
            .map_err(Into::into)
    }

    pub fn resolve_content_path_from_file_path(
        &self,
        file_path: &Path,
    ) -> anyhow::Result<Option<ContentPath<'static>>> {
        let content_url = Url::from_file_path(file_path).map_err(|()| {
            anyhow!(
                "invalid file path \"{file_path}\"",
                file_path = file_path.display()
            )
        })?;
        self.resolve_content_path_from_url(&content_url)
    }
}

pub type SharedStateObserver = discro::Observer<State>;
pub type SharedStateSubscriber = discro::Subscriber<State>;

#[derive(Debug)]
pub struct RestoringFromMusicDirectoryTaskContinuation {
    context: RestoringFromMusicDirectoryContext,
    pending_since: Instant,
}

#[derive(Debug)]
pub struct LoadingFromDatabaseTaskContinuation {
    context: LoadingFromDatabaseContext,
    pending_since: Instant,
}

#[derive(Debug)]
pub struct SynchronizingVfsTaskContinuation {
    context: SynchronizingVfsContext,
    pending_since: Instant,
}

pub type SynchronizeVfsResult = anyhow::Result<batch::synchronize_collection_vfs::Outcome>;

pub type SynchronizingVfsTaskJoined = JoinedTask<SynchronizeVfsResult>;

/// Manages the mutable, observable state
#[derive(Debug, Default)]
pub struct SharedState(Publisher<State>);

impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl SharedState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        Self(Publisher::new(initial_state))
    }

    #[must_use]
    pub fn read(&self) -> SharedStateRef<'_> {
        self.0.read()
    }

    #[must_use]
    pub fn observe(&self) -> SharedStateObserver {
        self.0.observe()
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> SharedStateSubscriber {
        self.0.subscribe_changed()
    }

    pub fn reset(&self) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, State::reset)
    }

    pub fn spawn_restoring_from_music_directory_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        kind: Option<Cow<'static, str>>,
        new_music_dir: DirPath<'static>,
        restore_entity: RestoreEntityStrategy,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> anyhow::Result<SpawnRestoringFromMusicDirectoryTaskReaction> {
        modify_shared_state_result(
            &self.0,
            |state| {
                state.spawn_restoring_from_music_directory_task(
                    self,
                    rt,
                    env,
                    kind,
                    new_music_dir,
                    restore_entity,
                    nested_music_dirs,
                )
            },
            |result| match result {
                Ok(reaction) => reaction.effect(),
                Err(_) => ActionEffect::Unchanged,
            },
        )
    }

    pub fn spawn_loading_from_database_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (ActionEffect, Option<AbortHandle>) {
        modify_shared_state_action_effect_result(&self.0, |state| {
            state.spawn_loading_from_database_task(self, rt, env)
        })
    }

    fn continue_after_restoring_from_music_directory_task_joined(
        &self,
        joined: JoinedTask<anyhow::Result<State>>,
        continuation: RestoringFromMusicDirectoryTaskContinuation,
    ) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| {
            state.continue_after_restoring_from_music_directory_task_joined(joined, continuation)
        })
    }

    fn continue_after_loading_from_database_task_joined(
        &self,
        joined: JoinedTask<anyhow::Result<State>>,
        continuation: LoadingFromDatabaseTaskContinuation,
    ) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| {
            state.continue_after_loading_from_database_task_joined(joined, continuation)
        })
    }

    pub fn spawn_synchronizing_vfs_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (ActionEffect, anyhow::Result<SynchronizingVfsTask>) {
        modify_shared_state_action_effect_result(&self.0, |state| {
            state.spawn_synchronizing_vfs_task(self, rt, env)
        })
    }

    pub async fn finish_synchronizing_vfs_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (ActionEffect, anyhow::Result<SynchronizingVfsFinishedState>) {
        let mut subscriber = self.subscribe_changed();
        loop {
            log::debug!("Suspending finish_synchronizing_vfs_task");
            if subscriber.changed().await.is_err() {
                return (ActionEffect::Changed, Err(anyhow!("no publisher(s)")));
            }
            log::debug!("Resuming finish_synchronizing_vfs_task");

            drop(subscriber.read_ack());
            let mut finished_state_before = None;
            self.0.modify(|state| match state {
                State::SynchronizingVfs {
                    context: sync_context,
                    state: sync_state,
                } => match sync_state {
                    SynchronizingVfsState::Pending { .. } => {
                        debug_assert!(finished_state_before.is_none());
                        false
                    }
                    SynchronizingVfsState::Finished(finished_state) => {
                        let entity_uid = sync_context.entity.hdr.uid.clone();
                        finished_state_before = Some(std::mem::replace(
                            finished_state,
                            // Replace with a dummy placeholder that is overwritten immediately after.
                            SynchronizingVfsFinishedState::Aborted,
                        ));
                        *state = State::Void;
                        let context = LoadingFromDatabaseContext {
                            entity_uid,
                            loaded_before: None,
                        };
                        let _abort_handle = state
                            .spawn_loading_from_database_task_unchecked(self, rt, env, context);
                        true
                    }
                },
                _ => false,
            });
            let Some(finished_state) = finished_state_before else {
                continue;
            };
            return (ActionEffect::Changed, Ok(finished_state));
        }
    }

    fn continue_after_synchronizing_vfs_task_joined(
        &self,
        joined: JoinedTask<anyhow::Result<Outcome>>,
        continuation: SynchronizingVfsTaskContinuation,
    ) -> ActionEffect {
        modify_shared_state_action_effect(&self.0, |state| {
            state.continue_after_synchronizing_vfs_task_joined(joined, continuation)
        })
    }
}

pub type SharedStateRef<'a> = discro::Ref<'a, State>;

#[derive(Debug, Clone)]
struct LoadStateFromDatabaseParams {
    entity_uid: Option<EntityUid>,
    context: Option<RestoringFromMusicDirectoryContext>,
}

async fn load_state_from_database<E>(
    env: E,
    params: LoadStateFromDatabaseParams,
) -> anyhow::Result<State>
where
    E: AsRef<Environment> + Send + 'static,
{
    let LoadStateFromDatabaseParams {
        entity_uid,
        context,
    } = params;
    let entity_with_summary = if let Some(entity_uid) = entity_uid.as_ref() {
        refresh_entity_from_db(env.as_ref(), entity_uid.clone()).await?
    } else {
        None
    };
    let Some(context) = context else {
        return Ok(entity_with_summary.map_or_else(
            || {
                if let Some(entity_uid) = entity_uid {
                    let context = LoadingFromDatabaseContext {
                        entity_uid,
                        loaded_before: None,
                    };
                    let state = LoadingFromDatabaseState::Finished(
                        LoadingFromDatabaseFinishedState::Failed {
                            error: "not found".to_owned(),
                        },
                    );
                    State::LoadingFromDatabase { context, state }
                } else {
                    State::Void
                }
            },
            State::loading_from_database_succeeded,
        ));
    };
    if let Some(entity_with_summary) = entity_with_summary {
        let RestoringFromMusicDirectoryContext {
            kind, music_dir, ..
        } = &context;
        if kind.is_none() || kind == &entity_with_summary.entity.body.kind {
            let entity_music_dir = vfs_music_dir(&entity_with_summary.entity.body);
            if entity_music_dir.as_ref() == Some(music_dir) {
                return Ok(State::loading_from_database_succeeded(entity_with_summary));
            }
        }
        log::debug!(
            "Discarding collection {uid}",
            uid = entity_with_summary.entity.hdr.uid
        );
    }
    context.restore(env.as_ref()).await
}

async fn synchronize_vfs<E, ReportProgressFn>(
    env: E,
    entity_uid: EntityUid,
    import_track_config: ImportTrackConfig,
    report_progress_fn: ReportProgressFn,
    abort_flag: Arc<AtomicBool>,
) -> SynchronizeVfsResult
where
    E: AsRef<Environment> + Send + 'static,
    ReportProgressFn: FnMut(batch::synchronize_collection_vfs::Progress) + Clone + Send + 'static,
{
    let params = batch::synchronize_collection_vfs::Params {
        root_url: None,
        max_depth: None,
        sync_mode: SyncMode::Modified,
        import_track_config,
        untracked_media_sources: UntrackedMediaSources::Purge,
        orphaned_media_sources: OrphanedMediaSources::Purge,
        untracked_files: UntrackedFiles::Find,
        unsynchronized_tracks: UnsynchronizedTracks::Find,
    };
    batch::synchronize_collection_vfs::synchronize_collection_vfs(
        env.as_ref().db_gatekeeper(),
        entity_uid,
        params,
        std::convert::identity,
        report_progress_fn,
        abort_flag,
    )
    .await
    .map_err(Into::into)
}

#[derive(Debug, Clone)]
pub struct SynchronizingVfsTask {
    progress: Observer<Option<Progress>>,
    abort_flag: Arc<AtomicBool>,
    task: AbortHandle,
}

impl SynchronizingVfsTask {
    #[allow(clippy::missing_panics_doc)]
    fn spawn(
        rt: &tokio::runtime::Handle,
        env: Arc<Environment>,
        state: SharedState,
        continuation: SynchronizingVfsTaskContinuation,
    ) -> Self {
        let progress_pub = Publisher::new(None);
        let progress = progress_pub.observe();
        let abort_flag = Arc::new(AtomicBool::new(false));

        let worker_task = rt.spawn({
            let report_progress_fn = {
                // TODO: How to avoid wrapping the publisher?
                let progress_pub = Arc::new(Mutex::new(progress_pub));
                move |progress: Progress| {
                    progress_pub.lock().unwrap().write(Some(progress));
                }
            };
            let import_track_config = ImportTrackConfig {
                // TODO: Customize faceted tag mapping
                faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
                ..Default::default()
            };
            let abort_flag = Arc::clone(&abort_flag);
            let entity_uid = continuation.context.entity.hdr.uid.clone();
            async move {
                log::debug!("Synchronizing collection with local file system...");
                synchronize_vfs(
                    env,
                    entity_uid,
                    import_track_config,
                    report_progress_fn,
                    abort_flag,
                )
                .await
            }
        });
        let abort_worker_task = worker_task.abort_handle();
        let _supervisor_task = rt.spawn({
            async move {
                let joined = JoinedTask::join(worker_task).await;
                let _effect =
                    state.continue_after_synchronizing_vfs_task_joined(joined, continuation);
            }
        });

        Self {
            progress,
            abort_flag,
            task: abort_worker_task,
        }
    }

    #[must_use]
    pub const fn progress(&self) -> &Observer<Option<Progress>> {
        &self.progress
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::Relaxed);
        self.task.abort();
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.task.is_finished()
    }
}
