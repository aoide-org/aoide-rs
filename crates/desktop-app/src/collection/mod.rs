// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
use tokio::task::{AbortHandle, JoinHandle};
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

use crate::{modify_shared_state_infallible, Environment, JoinedTask, Reaction, StateEffect};

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
            let state =
                RestoringFromMusicDirectoryState::NestedMusicDirectoriesConflict { candidates };
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
                let state = RestoringFromMusicDirectoryState::Failed {
                    error: RestoreFromMusicDirectoryError::EntityNotFound,
                };
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
pub enum RestoreFromMusicDirectoryError {
    EntityNotFound,
    Other(String),
}

impl RestoreFromMusicDirectoryError {
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
    Pending {
        since: Instant,
        abort_handle: AbortHandle,
    },
    Failed {
        error: RestoreFromMusicDirectoryError,
    },
    NestedMusicDirectoriesConflict {
        candidates: Vec<EntityWithSummary>,
    },
}

#[derive(Debug)]
pub enum LoadingFromDatabaseState {
    Pending {
        since: Instant,
        abort_handle: AbortHandle,
    },
    Failed {
        error: String,
    },
}

#[derive(Debug)]
pub enum SynchronizingVfsState {
    Pending {
        since: Instant,
        task: SynchronizingVfsTask,
    },
    Failed {
        error: String,
    },
    Succeeded {
        outcome: Outcome,
    },
    Aborted,
}

/// State of a single collection that is based on directory in the
/// local directory using a virtual file system (VFS) for content paths.
#[derive(Debug, Default)]
#[allow(clippy::large_enum_variant)]
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
                state: RestoringFromMusicDirectoryState::Failed { error },
                ..
            } => Some(error.as_str()),
            Self::LoadingFromDatabase {
                state: LoadingFromDatabaseState::Failed { error },
                ..
            } => Some(error.as_str()),
            Self::SynchronizingVfs {
                state: SynchronizingVfsState::Failed { error },
                ..
            } => Some(error.as_str()),
            _ => None,
        }
    }

    pub fn reset(&mut self) -> StateEffect {
        if matches!(self, Self::Void) {
            return StateEffect::Unchanged;
        }
        let reset = Self::Void;
        log::debug!("Resetting state: {self:?} -> {reset:?}");
        *self = reset;
        StateEffect::Changed
    }

    pub fn spawn_restoring_from_music_directory_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        new_kind: Option<Cow<'_, str>>,
        new_music_dir: DirPath<'_>,
        restore_entity: RestoreEntityStrategy,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> (Reaction, StateEffect) {
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
                        return (Reaction::Accepted, StateEffect::Unchanged);
                    }
                }
            }
            Self::RestoringFromMusicDirectory {
                state: RestoringFromMusicDirectoryState::NestedMusicDirectoriesConflict { .. },
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
                    return (Reaction::Accepted, StateEffect::Unchanged);
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
        let abort_handle = worker_task.abort_handle();
        rt.spawn({
            let this = this.clone();
            async move {
                let worker_joined = JoinedTask::join(worker_task).await;
                let _ =
                    this.restoring_from_music_directory_task_joined(worker_joined, continuation);
            }
        });

        *self = Self::RestoringFromMusicDirectory {
            context,
            state: RestoringFromMusicDirectoryState::Pending {
                since: pending_since,
                abort_handle,
            },
        };
        (Reaction::Accepted, StateEffect::Changed)
    }

    fn spawn_loading_from_database_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (Reaction, StateEffect) {
        let old_self = std::mem::replace(self, Self::Void);
        let context = match old_self {
            Self::Void => {
                // Nothing to do.
                return (Reaction::Accepted, StateEffect::Unchanged);
            }
            Self::RestoringFromMusicDirectory {
                context,
                state:
                    RestoringFromMusicDirectoryState::Failed { .. }
                    | RestoringFromMusicDirectoryState::NestedMusicDirectoriesConflict { .. },
            } => {
                let pending_since = Instant::now();
                debug_assert_ne!(Some(pending_since), self.pending_since());
                let continuation = RestoringFromMusicDirectoryTaskContinuation {
                    context: context.clone(),
                    pending_since,
                };

                let task = rt.spawn({
                    let this = this.clone();
                    let env = env.clone();
                    let params = LoadStateFromDatabaseParams {
                        entity_uid: None,
                        context: Some(context.clone()),
                    };
                    async move {
                        let result = load_state_from_database(env, params).await;
                        let _ = this
                            .restoring_from_music_directory_task_completed(result, continuation);
                    }
                });

                *self = Self::RestoringFromMusicDirectory {
                    context,
                    state: RestoringFromMusicDirectoryState::Pending {
                        since: pending_since,
                        task,
                    },
                };

                return (Reaction::Accepted, StateEffect::Changed);
            }
            Self::LoadingFromDatabase {
                state: LoadingFromDatabaseState::Failed { .. },
                context,
            } => context,
            Self::Ready { entity, .. }
            | Self::SynchronizingVfs {
                state:
                    SynchronizingVfsState::Failed { .. }
                    | SynchronizingVfsState::Succeeded { .. }
                    | SynchronizingVfsState::Aborted { .. },
                context: SynchronizingVfsContext { entity, .. },
            } => LoadingFromDatabaseContext {
                entity_uid: entity.raw.hdr.uid,
                loaded_before: Some(entity.raw.body),
            },
            old_self => {
                // Restore old state and reject.
                *self = old_self;
                log::warn!("Illegal state for refreshing from database: {self:?}");
                return (Reaction::Rejected, StateEffect::Unchanged);
            }
        };
        self.spawn_loading_from_database_task_unchecked(this, rt, env, context);
        (Reaction::Accepted, StateEffect::Changed)
    }

    #[must_use]
    fn spawn_loading_from_database_task_unchecked(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        context: LoadingFromDatabaseContext,
    ) {
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

        let task = rt.spawn({
            let this = this.clone();
            let env = env.clone();
            async move {
                let result = load_state_from_database(env, params).await;
                let _ = this.loading_from_database_task_completed(result, continuation);
            }
        });

        *self = Self::LoadingFromDatabase {
            context,
            state: LoadingFromDatabaseState::Pending {
                since: pending_since,
                task,
            },
        };
    }

    fn continue_after_restoring_from_music_directory_task_joined(
        &mut self,
        joined: JoinedTask<anyhow::Result<State>>,
        continuation: RestoringFromMusicDirectoryTaskContinuation,
    ) -> (Reaction, StateEffect) {
        let RestoringFromMusicDirectoryTaskContinuation {
            context: continuation_context,
            pending_since: continuation_pending_since,
        } = continuation;
        match self {
            Self::RestoringFromMusicDirectory {
                context,
                state:
                    RestoringFromMusicDirectoryState::Pending {
                        since: pending_since,
                        abort_handle,
                    },
            } => {
                debug_assert!(abort_handle.is_finished());
                if *pending_since != continuation_pending_since || *context != continuation_context
                {
                    log::warn!(
                        "State changed while restoring from music directory: current state {self:?}, continuation {continuation:?} - discarding {joined:?}",
                        continuation = RestoringFromMusicDirectoryTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                    );
                    return (Reaction::Rejected, StateEffect::Unchanged);
                }
                match joined {
                    JoinedTask::Completed(Ok(next_state)) => {
                        log::debug!("Restored state from music directory: {next_state:?}");
                        *self = next_state;
                    }
                    JoinedTask::Cancelled => {
                        log::debug!("Restored state from music directory cancelled");
                        *self = Self::Void;
                    }
                    JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                        log::warn!("Failed to restore state from music directory: {err}");
                        let error = RestoreFromMusicDirectoryError::Other(err.to_string());
                        *self = Self::RestoringFromMusicDirectory {
                            context: continuation_context,
                            state: RestoringFromMusicDirectoryState::Failed { error },
                        };
                    }
                }
            }
            _ => {
                log::warn!(
                    "State changed while restoring from music directory: current state {self:?}, continuation {continuation:?} - discarding {joined:?}",
                        continuation = RestoringFromMusicDirectoryTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                );
                return (Reaction::Rejected, StateEffect::Unchanged);
            }
        }
        (Reaction::Accepted, StateEffect::MaybeChanged)
    }

    fn continue_after_loading_from_database_task_joined(
        &mut self,
        joined: JoinedTask<anyhow::Result<State>>,
        continuation: LoadingFromDatabaseTaskContinuation,
    ) -> (Reaction, StateEffect) {
        let LoadingFromDatabaseTaskContinuation {
            context: continuation_context,
            pending_since: continuation_pending_since,
        } = continuation;
        match self {
            Self::LoadingFromDatabase {
                context: self_context,
                state:
                    LoadingFromDatabaseState::Pending {
                        since: self_pending_since,
                        abort_handle,
                    },
            } => {
                debug_assert!(abort_handle.is_finished());
                if *pending_since != continuation_pending_since || *context != continuation_context {
                    log::warn!(
                        "State changed while loading from database: current state {self:?}, continuation {self_context:?} - discarding {joined:?}",
                        continuation = LoadingFromDatabaseTaskContinuation {
                            context: continuation_context,
                            pending_since: continuation_pending_since,
                        }
                    );
                    return (Reaction::Rejected, StateEffect::Unchanged);
                }
                match joined {
                    JoinedTask::Completed(Ok(next_state)) => {
                        log::debug!("Loaded state from database: {next_state:?}");
                        *self = next_state;
                    }
                    JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                        log::warn!("Failed to load state from database: {err}");
                        let error = err.to_string();
                        *self = Self::LoadingFromDatabase {
                            context,
                            state: LoadingFromDatabaseState::Failed { error },
                        };
                    }
                }
            }
            _ => {
                log::warn!(
                    "State changed while loading from database: current {self:?}, continuation {continuation:?} - discarding {result:?}",
                        continuation = LoadingFromDatabaseTaskContinuation {
                            context,
                            pending_since,
                        }
                );
                return (Reaction::Rejected, StateEffect::Unchanged);
            }
        }
        (Reaction::Accepted, StateEffect::MaybeChanged)
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
            let state = LoadingFromDatabaseState::Failed {
                error: "no summary".to_owned(),
            };
            Self::LoadingFromDatabase { context, state }
        }
    }

    fn spawn_synchronizing_vfs_task(
        &mut self,
        this: &SharedState,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (Reaction, StateEffect) {
        let old_self = std::mem::replace(self, Self::Void);
        let Self::Ready { entity, .. } = old_self else {
            // Restore old state and reject.
            log::warn!("Illegal state for synchronizing with local file system: {old_self:?}");
            *self = old_self;
            return (Reaction::Rejected, StateEffect::Unchanged);
        };

        let context = SynchronizingVfsContext { entity };

        let pending_since = Instant::now();
        let continuation = SynchronizingVfsTaskContinuation {
            context: context.clone(),
            pending_since,
        };

        let task = SynchronizingVfsTask::spawn(rt, env.clone(), this.clone(), continuation);
        let state = SynchronizingVfsState::Pending {
            since: pending_since,
            task,
        };

        *self = Self::SynchronizingVfs { context, state };
        (Reaction::Accepted, StateEffect::Changed)
    }

    fn continue_after_synchronizing_vfs_task_completed(
        &mut self,
        result: anyhow::Result<Outcome>,
        continuation: SynchronizingVfsTaskContinuation,
    ) -> (Reaction, StateEffect) {
        let Self::SynchronizingVfs {
            state:
                SynchronizingVfsState::Pending {
                    since: pending_since,
                    task: _,
                },
            context,
        } = self
        else {
            log::warn!(
                "State changed while synchronizing: current state {self:?}, continuation {continuation:?} - discarding {result:?}",
            );
            return (Reaction::Rejected, StateEffect::Unchanged);
        };
        let SynchronizingVfsTaskContinuation {
            context: continuation_context,
            pending_since: continuation_pending_since,
        } = continuation;
        if continuation_pending_since != *pending_since || continuation_context != *context {
            log::warn!(
                "State changed while synchronizing: current state {self:?}, continuation {continuation:?} - discarding {result:?}",
                continuation = SynchronizingVfsTaskContinuation {
                    context: continuation_context,
                    pending_since: continuation_pending_since,
                }
            );
            return (Reaction::Rejected, StateEffect::Unchanged);
        }

        let next_state = match result {
            Ok(outcome) => SynchronizingVfsState::Succeeded { outcome },
            Err(err) => {
                let error = err.to_string();
                SynchronizingVfsState::Failed { error }
            }
            JoinedTask::Cancelled => SynchronizingVfsState::Aborted,
        };
        *self = Self::SynchronizingVfs {
            state: next_state,
            context: continuation_context,
        };

        (Reaction::Accepted, StateEffect::Changed)
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
    pub fn subscribe_changed(&self) -> SharedStateSubscriber {
        self.0.subscribe_changed()
    }

    pub fn set_modified(&self) {
        self.0.set_modified();
    }

    pub fn reset(&self) -> StateEffect {
        modify_shared_state_infallible(&self.0, |state| (Reaction::Accepted, state.reset())).1
    }

    #[must_use]
    pub fn spawn_restoring_from_music_directory_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
        kind: Option<Cow<'static, str>>,
        new_music_dir: Option<DirPath<'static>>,
        restore_entity: RestoreEntityStrategy,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> (Reaction, StateEffect) {
        let Some(new_music_dir) = new_music_dir else {
            return (Reaction::Accepted, self.reset());
        };
        modify_shared_state_infallible(&self.0, |state| {
            state.spawn_restoring_from_music_directory_task(
                self,
                rt,
                env,
                kind,
                new_music_dir,
                restore_entity,
                nested_music_dirs,
            )
        })
    }

    #[must_use]
    pub fn spawn_loading_from_database_task(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (Reaction, StateEffect) {
        modify_shared_state_infallible(&self.0, |state| {
            state.spawn_loading_from_database_task(self, rt, env)
        })
    }

    #[must_use]
    fn continue_after_restoring_from_music_directory_task_completed(
        &self,
        result: anyhow::Result<State>,
        continuation: RestoringFromMusicDirectoryTaskContinuation,
    ) -> (Reaction, StateEffect) {
        modify_shared_state_infallible(&self.0, |state| {
            state.continue_after_restoring_from_music_directory_task_completed(result, continuation)
        })
    }

    #[must_use]
    fn continue_after_loading_from_database_task_completed(
        &self,
        result: anyhow::Result<State>,
        continuation: LoadingFromDatabaseTaskContinuation,
    ) -> (Reaction, StateEffect) {
        modify_shared_state_infallible(&self.0, |state| {
            state.continue_after_loading_from_database_task_completed(result, continuation)
        })
    }

    fn spawn_synchronizing_vfs_task<ReportProgressFn>(
        &self,
        rt: &tokio::runtime::Handle,
        env: &Arc<Environment>,
    ) -> (Reaction, StateEffect) {
        modify_shared_state_infallible(&self.0, |state| {
            state.spawn_synchronizing_vfs_task(self, rt, env)
        })
    }

    fn continue_after_synchronizing_vfs_task_completed(
        &self,
        result: anyhow::Result<Outcome>,
        continuation: SynchronizingVfsTaskContinuation,
    ) -> (Reaction, StateEffect) {
        modify_shared_state_infallible(&self.0, |state| {
            state.continue_after_synchronizing_vfs_task_completed(result, continuation)
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
                    let state = LoadingFromDatabaseState::Failed {
                        error: "not found".to_owned(),
                    };
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

#[derive(Debug)]
pub struct SynchronizingVfsTask {
    progress: Observer<Option<Progress>>,
    abort_flag: Arc<AtomicBool>,
    task: JoinHandle<()>,
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

        let task = rt.spawn({
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
                let result = synchronize_vfs(
                    env,
                    entity_uid,
                    import_track_config,
                    report_progress_fn,
                    abort_flag,
                )
                .await;
                let _ = state.continue_after_synchronizing_vfs_task_completed(result, continuation);
            }
        });

        Self {
            progress,
            abort_flag,
            task,
        }
    }

    #[must_use]
    pub const fn progress(&self) -> &Observer<Option<Progress>> {
        &self.progress
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::Relaxed);
        // Both tasks should not be cancelled! The inner task will finish
        // ASAP after the abort flag has been set.
    }
}
