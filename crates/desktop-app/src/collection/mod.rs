// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    future::Future,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};

use anyhow::anyhow;
use discro::{Observer, Publisher};
use tokio::task::JoinHandle;
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
    modify_observable_state, Environment, Handle, JoinedTask, Observable, ObservableReader,
    ObservableRef, StateUnchanged,
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
pub struct LoadingContext {
    pub entity_uid: EntityUid,
    pub loaded_before: Option<Collection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronizingContext {
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
            let state = State::loading_succeeded(entity_with_summary);
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
                let state = State::loading_succeeded(entity_with_summary);
                Ok(state)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoringFromMusicDirectoryState {
    Pending {
        pending_since: Instant,
    },
    Failed {
        error: RestoreFromMusicDirectoryError,
    },
    NestedMusicDirectoriesConflict {
        candidates: Vec<EntityWithSummary>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadingState {
    Pending { pending_since: Instant },
    Failed { error: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizingState {
    Pending { pending_since: Instant },
    Failed { error: String },
    Succeeded,
    Aborted,
}

/// State of a single collection that is based on directory in the
/// local directory using a virtual file system (VFS) for content paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum State {
    #[default]
    Void,
    RestoringFromMusicDirectory {
        context: RestoringFromMusicDirectoryContext,
        state: RestoringFromMusicDirectoryState,
    },
    Loading {
        context: LoadingContext,
        state: LoadingState,
    },
    Synchronizing {
        context: SynchronizingContext,
        state: SynchronizingState,
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
                state: RestoringFromMusicDirectoryState::Pending { pending_since, .. },
                ..
            }
            | Self::Loading {
                state: LoadingState::Pending { pending_since, .. },
                ..
            }
            | Self::Synchronizing {
                state: SynchronizingState::Pending { pending_since, .. },
                ..
            } => Some(*pending_since),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending_since().is_some()
    }

    #[must_use]
    pub const fn is_synchronizing(&self) -> bool {
        matches!(self, Self::Synchronizing { .. })
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, State::Ready { .. })
    }

    #[must_use]
    pub fn music_dir(&self) -> Option<DirPath<'_>> {
        match self {
            Self::Void
            | Self::Loading {
                context:
                    LoadingContext {
                        loaded_before: None,
                        ..
                    },
                ..
            } => None,
            Self::Loading {
                context:
                    LoadingContext {
                        loaded_before: Some(loaded_before),
                        ..
                    },
                ..
            } => vfs_music_dir(loaded_before),
            Self::Synchronizing {
                context: SynchronizingContext { entity },
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
            Self::Loading {
                context:
                    LoadingContext {
                        entity_uid,
                        loaded_before,
                    },
                ..
            } => Some((entity_uid, loaded_before.as_ref())),
            Self::Synchronizing {
                context: SynchronizingContext { entity },
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
            Self::Loading {
                state: LoadingState::Failed { error },
                ..
            } => Some(error.as_str()),
            _ => None,
        }
    }

    pub fn reset(&mut self) -> Result<(), StateUnchanged> {
        if matches!(self, Self::Void) {
            return Err(StateUnchanged);
        }
        let reset = Self::Void;
        log::debug!("Resetting state: {self:?} -> {reset:?}");
        *self = reset;
        Ok(())
    }

    pub fn update_music_dir(
        &mut self,
        new_kind: Option<Cow<'_, str>>,
        new_music_dir: DirPath<'_>,
        restore_entity: RestoreEntityStrategy,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> Result<(), StateUnchanged> {
        if self.is_pending() {
            log::warn!("Illegal state for updating directory: {self:?}");
            return Err(StateUnchanged);
        }
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
                        return Err(StateUnchanged);
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
                    return Err(StateUnchanged);
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
        let state = RestoringFromMusicDirectoryState::Pending {
            pending_since: Instant::now(),
        };
        *self = Self::RestoringFromMusicDirectory { context, state };
        Ok(())
    }

    fn refresh_from_db(&mut self) -> Result<RefreshStateFromDbParams, StateUnchanged> {
        let old_self = std::mem::replace(self, Self::Void);
        let context = match old_self {
            Self::Void => {
                return Err(StateUnchanged);
            }
            Self::RestoringFromMusicDirectory { context, .. } => {
                let params = RefreshStateFromDbParams {
                    entity_uid: None,
                    context: Some(context.clone()),
                };
                let state = RestoringFromMusicDirectoryState::Pending {
                    pending_since: Instant::now(),
                };
                *self = Self::RestoringFromMusicDirectory { context, state };
                return Ok(params);
            }
            Self::Loading {
                state: LoadingState::Failed { .. },
                context,
            } => context,
            Self::Ready { entity, .. }
            | Self::Synchronizing {
                state:
                    SynchronizingState::Failed { .. }
                    | SynchronizingState::Succeeded { .. }
                    | SynchronizingState::Aborted { .. },
                context: SynchronizingContext { entity, .. },
            } => LoadingContext {
                entity_uid: entity.raw.hdr.uid,
                loaded_before: Some(entity.raw.body),
            },
            _ => {
                log::warn!("Illegal state for refreshing from database: {old_self:?}");
                *self = old_self;
                return Err(StateUnchanged);
            }
        };
        let params = self.refresh_from_db_unchecked(context);
        Ok(params)
    }

    #[must_use]
    fn refresh_from_db_unchecked(&mut self, context: LoadingContext) -> RefreshStateFromDbParams {
        debug_assert!(matches!(self, Self::Void));
        let params = RefreshStateFromDbParams {
            entity_uid: Some(context.entity_uid.clone()),
            context: None, // Omit checking the context.
        };
        let state = LoadingState::Pending {
            pending_since: Instant::now(),
        };
        *self = Self::Loading { context, state };
        params
    }

    fn synchronize_vfs(&mut self) -> Result<EntityUid, StateUnchanged> {
        let old_self = std::mem::replace(self, Self::Void);
        let Self::Ready { entity, .. } = old_self else {
            log::warn!("Illegal state for synchronizing: {old_self:?}");
            *self = old_self;
            return Err(StateUnchanged);
        };
        let entity_uid = entity.hdr.uid.clone();
        let context = SynchronizingContext { entity };
        let state = SynchronizingState::Pending {
            pending_since: Instant::now(),
        };
        *self = Self::Synchronizing { context, state };
        Ok(entity_uid)
    }

    #[must_use]
    fn loading_succeeded(entity_with_summary: EntityWithSummary) -> Self {
        let EntityWithSummary { entity, summary } = entity_with_summary;
        if let Some(summary) = summary {
            State::Ready { entity, summary }
        } else {
            // Should never happen
            let context = LoadingContext {
                entity_uid: entity.raw.hdr.uid,
                loaded_before: Some(entity.raw.body),
            };
            let state = LoadingState::Failed {
                error: "no summary".to_owned(),
            };
            Self::Loading { context, state }
        }
    }

    fn refresh_from_db_task_completed(
        &mut self,
        result: anyhow::Result<State>,
        continuation: RefreshFromDbTaskContinuation,
    ) -> Result<(), StateUnchanged> {
        let RefreshFromDbTaskContinuation { pending_state } = continuation;
        if pending_state != *self {
            log::warn!(
                "State changed while refreshing from database: expected {pending_state:?}, actual {self:?} - discarding {result:?}",
            );
            return Err(StateUnchanged);
        }
        match result {
            Ok(next_state) => {
                if *self == next_state {
                    return Err(StateUnchanged);
                }
                log::debug!("Refreshed state from database: {next_state:?}");
                *self = next_state;
            }
            Err(err) => {
                let error = err.to_string();
                match self {
                    State::RestoringFromMusicDirectory { state, .. } => {
                        log::warn!("Restoring from music directory failed: {error}");
                        let error = RestoreFromMusicDirectoryError::Other(error);
                        let next_state = RestoringFromMusicDirectoryState::Failed { error };
                        debug_assert_ne!(*state, next_state);
                        *state = next_state;
                    }
                    State::Loading { state, .. } => {
                        log::warn!("Loading failed: {error}");
                        let next_state = LoadingState::Failed { error };
                        debug_assert_ne!(*state, next_state);
                        *state = next_state;
                    }
                    _ => unreachable!(),
                }
            }
        };
        Ok(())
    }

    fn synchronize_vfs_task_joined(
        &mut self,
        joined_task: SynchronizeVfsTaskJoined,
        continuation: SynchronizeVfsTaskContinuation,
    ) -> Result<Option<Outcome>, StateUnchanged> {
        let SynchronizeVfsTaskContinuation { pending_state } = continuation;
        debug_assert!(matches!(
            pending_state,
            State::Synchronizing {
                state: SynchronizingState::Pending { .. },
                ..
            }
        ));
        if pending_state != *self {
            log::warn!(
                "State changed while synchronizing: expected {pending_state:?}, actual {self:?}",
            );
            return Err(StateUnchanged);
        }
        let Self::Synchronizing { state, .. } = self else {
            unreachable!("illegal state");
        };
        let mut outcome = None;
        let next_state = match joined_task {
            JoinedTask::Cancelled => SynchronizingState::Aborted,
            JoinedTask::Completed(Ok(ok)) => {
                outcome = Some(ok);
                SynchronizingState::Succeeded
            }
            JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                let error = err.to_string();
                SynchronizingState::Failed { error }
            }
        };
        debug_assert_ne!(*state, next_state);
        *state = next_state;
        Ok(outcome)
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

pub type StateSubscriber = discro::Subscriber<State>;

#[derive(Debug)]
pub struct RefreshFromDbTaskContinuation {
    pending_state: State,
}

/// Context for applying the corresponding [`JoinedTask`].
#[derive(Debug)]
pub struct SynchronizeVfsTaskContinuation {
    pending_state: State,
}

pub type SynchronizeVfsResult = anyhow::Result<batch::synchronize_collection_vfs::Outcome>;

pub type SynchronizeVfsTaskJoined = JoinedTask<SynchronizeVfsResult>;

/// Manages the mutable, observable state
#[derive(Debug)]
pub struct ObservableState(Observable<State>);

impl ObservableState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        Self(Observable::new(initial_state))
    }

    #[must_use]
    pub fn read(&self) -> ObservableStateRef<'_> {
        self.0.read()
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> StateSubscriber {
        self.0.subscribe_changed()
    }

    pub fn set_modified(&self) {
        self.0.set_modified();
    }

    pub fn reset(&self) -> Result<(), StateUnchanged> {
        modify_observable_state(&self.0, State::reset)
    }

    fn update_music_dir(
        &self,
        kind: Option<Cow<'static, str>>,
        new_music_dir: Option<DirPath<'static>>,
        restore_entity: RestoreEntityStrategy,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> Result<(), StateUnchanged> {
        let Some(new_music_dir) = new_music_dir else {
            log::debug!("Resetting music directory");
            return self.reset();
        };
        log::debug!(
            "Updating music directory: {new_music_dir}",
            new_music_dir = new_music_dir.display()
        );
        modify_observable_state(&self.0, |state| {
            state.update_music_dir(kind, new_music_dir, restore_entity, nested_music_dirs)
        })
    }

    pub fn refresh_from_db_task(
        &self,
        handle: &Handle,
    ) -> Result<
        (
            impl Future<Output = anyhow::Result<State>> + Send + 'static,
            RefreshFromDbTaskContinuation,
        ),
        StateUnchanged,
    > {
        let (pending_state, params) = modify_observable_state(&self.0, |state: &mut State| {
            let params = state.refresh_from_db()?;
            debug_assert!(state.is_pending());
            let pending_state = state.clone();
            Ok((pending_state, params))
        })?;
        let handle = handle.clone();
        let task = async move { refresh_state_from_db(handle, params).await };
        let continuation = RefreshFromDbTaskContinuation { pending_state };
        Ok((task, continuation))
    }

    pub fn refresh_from_db_task_completed(
        &self,
        result: anyhow::Result<State>,
        continuation: RefreshFromDbTaskContinuation,
    ) -> Result<(), StateUnchanged> {
        modify_observable_state(&self.0, |state| {
            state.refresh_from_db_task_completed(result, continuation)
        })
    }

    fn synchronize_vfs_task<ReportProgressFn>(
        &self,
        handle: &Handle,
        import_track_config: ImportTrackConfig,
        report_progress_fn: ReportProgressFn,
        abort_flag: Arc<AtomicBool>,
    ) -> Result<
        (
            impl Future<Output = SynchronizeVfsResult> + Send + 'static,
            SynchronizeVfsTaskContinuation,
        ),
        StateUnchanged,
    >
    where
        ReportProgressFn:
            FnMut(batch::synchronize_collection_vfs::Progress) + Clone + Send + 'static,
    {
        let (pending_state, entity_uid) = modify_observable_state(&self.0, |state| {
            let entity_uid = state.synchronize_vfs()?;
            debug_assert!(state.is_pending());
            let pending_state = state.clone();
            Ok((pending_state, entity_uid))
        })?;
        debug_assert!(matches!(pending_state, State::Synchronizing { .. }));
        let handle = handle.clone();
        let task = async move {
            synchronize_vfs(
                handle,
                entity_uid,
                import_track_config,
                report_progress_fn,
                abort_flag,
            )
            .await
        };
        let continuation = SynchronizeVfsTaskContinuation { pending_state };
        Ok((task, continuation))
    }

    fn synchronize_vfs_task_joined(
        &self,
        joined_task: SynchronizeVfsTaskJoined,
        continuation: SynchronizeVfsTaskContinuation,
    ) -> Result<Option<Outcome>, StateUnchanged> {
        modify_observable_state(&self.0, |state| {
            state.synchronize_vfs_task_joined(joined_task, continuation)
        })
    }
}

impl Default for ObservableState {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

pub type ObservableStateRef<'a> = ObservableRef<'a, State>;

impl ObservableReader<State> for ObservableState {
    fn read_lock(&self) -> ObservableStateRef<'_> {
        self.0.read_lock()
    }
}

#[derive(Debug, Clone)]
struct RefreshStateFromDbParams {
    entity_uid: Option<EntityUid>,
    context: Option<RestoringFromMusicDirectoryContext>,
}

async fn refresh_state_from_db<E>(env: E, params: RefreshStateFromDbParams) -> anyhow::Result<State>
where
    E: AsRef<Environment> + Send + 'static,
{
    let RefreshStateFromDbParams {
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
                    let context = LoadingContext {
                        entity_uid,
                        loaded_before: None,
                    };
                    let state = LoadingState::Failed {
                        error: "not found".to_owned(),
                    };
                    State::Loading { context, state }
                } else {
                    State::Void
                }
            },
            State::loading_succeeded,
        ));
    };
    if let Some(entity_with_summary) = entity_with_summary {
        let RestoringFromMusicDirectoryContext {
            kind, music_dir, ..
        } = &context;
        if kind.is_none() || kind == &entity_with_summary.entity.body.kind {
            let entity_music_dir = vfs_music_dir(&entity_with_summary.entity.body);
            if entity_music_dir.as_ref() == Some(music_dir) {
                return Ok(State::loading_succeeded(entity_with_summary));
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

/// Background task.
///
/// Both progress and outcome are observable.
#[derive(Debug)]
pub struct SynchronizeVfsTask {
    started_at: Instant,
    progress: Observer<Option<Progress>>,
    outcome: Observer<Option<Outcome>>,
    abort_flag: Arc<AtomicBool>,
    supervisor_handle: JoinHandle<()>,
}

impl SynchronizeVfsTask {
    #[allow(clippy::missing_panics_doc)]
    pub fn spawn(
        rt: &tokio::runtime::Handle,
        handle: &Handle,
        state: &Arc<ObservableState>,
    ) -> Result<Self, StateUnchanged> {
        let started_at = Instant::now();
        let progress_pub = Publisher::new(None);
        let progress = progress_pub.observe();
        let outcome_pub = Publisher::new(None);
        let outcome = outcome_pub.observe();
        let report_progress_fn = {
            // TODO: How to avoid wrapping the publisher?
            let progress_pub = Arc::new(Mutex::new(progress_pub));
            move |progress: Option<Progress>| {
                progress_pub.lock().unwrap().write(progress);
            }
        };
        let abort_flag = Arc::new(AtomicBool::new(false));
        let (task, continuation) =
            synchronize_vfs_task(state, handle, report_progress_fn, Arc::clone(&abort_flag))?;
        let join_handle = rt.spawn(task);
        let state = Arc::clone(state);
        // The supervisor task is responsible for updating the state eventually.
        // It finishes after the main task finished.
        let supervisor_task = async move {
            let joined_task = JoinedTask::join(join_handle).await;
            log::debug!("Synchronize music directory task joined: {joined_task:?}");
            let result = state.synchronize_vfs_task_joined(joined_task, continuation);
            if let Ok(outcome) = result {
                outcome_pub.write(outcome);
            }
        };
        let supervisor_handle = rt.spawn(supervisor_task);
        Ok(Self {
            started_at,
            progress,
            outcome,
            abort_flag,
            supervisor_handle,
        })
    }

    #[must_use]
    pub const fn started_at(&self) -> Instant {
        self.started_at
    }

    #[must_use]
    pub const fn progress(&self) -> &Observer<Option<Progress>> {
        &self.progress
    }

    #[must_use]
    pub const fn outcome(&self) -> &Observer<Option<Outcome>> {
        &self.outcome
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.supervisor_handle.is_finished()
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::Relaxed);
        // Both tasks should not be cancelled! The inner task will finish
        // ASAP after the abort flag has been set.
    }

    pub async fn join(self) -> anyhow::Result<Option<Outcome>> {
        let Self {
            outcome,
            supervisor_handle,
            ..
        } = self;
        supervisor_handle
            .await
            .map(|()| {
                let outcome = outcome.read().clone();
                debug_assert!(outcome.is_some());
                outcome
            })
            .map_err(|err| {
                debug_assert!(outcome.read().is_none());
                // The supervisor task is never cancelled.
                debug_assert!(!err.is_cancelled());
                err.into()
            })
    }
}

fn synchronize_vfs_task(
    state: &ObservableState,
    handle: &Handle,
    report_progress_fn: impl FnMut(Option<Progress>) + Clone + Send + 'static,
    abort_flag: Arc<AtomicBool>,
) -> Result<
    (
        impl Future<Output = anyhow::Result<Outcome>> + Send + 'static,
        SynchronizeVfsTaskContinuation,
    ),
    StateUnchanged,
> {
    log::debug!("Synchronizing collection...");
    let import_track_config = ImportTrackConfig {
        // TODO: Customize faceted tag mapping
        faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
        ..Default::default()
    };
    let mut report_progress_fn = report_progress_fn.clone();
    let report_progress_fn = move |progress| {
        report_progress_fn(Some(progress));
    };
    state.synchronize_vfs_task(handle, import_track_config, report_progress_fn, abort_flag)
}
