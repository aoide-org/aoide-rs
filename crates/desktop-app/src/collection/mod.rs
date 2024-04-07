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
use discro::{Publisher, Subscriber};
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
    .map(|entity_with_summary| {
        if entity_with_summary.is_some() {
            log::info!("Reloaded collection {entity_uid}");
        } else {
            log::warn!("Collection {entity_uid} not found");
        }
        entity_with_summary
    })
    .map_err(Into::into)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RestoreState {
    kind: Option<String>,
    music_dir: DirPath<'static>,
    restore_entity: RestoreEntityStrategy,
    nested_music_dirs: NestedMusicDirectoriesStrategy,
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

impl RestoreState {
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
            let state = RestoreState {
                kind,
                music_dir: music_dir.into(),
                restore_entity,
                nested_music_dirs,
            };
            return Ok(State::NestedMusicDirectoriesConflict { state, candidates });
        }
        // No matching entity found.
        match restore_entity {
            RestoreEntityStrategy::Load => {
                let state = RestoreState {
                    kind,
                    music_dir: music_dir.into(),
                    restore_entity,
                    nested_music_dirs,
                };
                Ok(State::RestoringFromMusicDirectoryFailed {
                    state,
                    error: RestoreFromMusicDirectoryError::EntityNotFound,
                })
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

/// State of a single collection that is based on directory in the
/// local directory using a virtual file system (VFS) for content paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum State {
    #[default]
    Void,
    RestoringFromMusicDirectory {
        state: RestoreState,
        pending_since: Instant,
    },
    RestoringFromMusicDirectoryFailed {
        state: RestoreState,
        error: RestoreFromMusicDirectoryError,
    },
    NestedMusicDirectoriesConflict {
        state: RestoreState,
        candidates: Vec<EntityWithSummary>,
    },
    Loading {
        entity_uid: EntityUid,
        loaded_before: Option<Collection>,
        pending_since: Instant,
    },
    LoadingFailed {
        entity_uid: EntityUid,
        loaded_before: Option<Collection>,
        error: String,
    },
    Synchronizing {
        entity: Entity,
        pending_since: Instant,
    },
    SynchronizingFailed {
        entity: Entity,
        error: String,
    },
    SynchronizingSucceeded {
        entity: Entity,
    },
    SynchronizingAborted {
        entity: Entity,
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
            Self::Void
            | Self::NestedMusicDirectoriesConflict { .. }
            | Self::RestoringFromMusicDirectoryFailed { .. }
            | Self::LoadingFailed { .. }
            | Self::SynchronizingFailed { .. }
            | Self::SynchronizingSucceeded { .. }
            | Self::SynchronizingAborted { .. }
            | Self::Ready { .. } => None,
            Self::RestoringFromMusicDirectory { pending_since, .. }
            | Self::Loading { pending_since, .. }
            | Self::Synchronizing { pending_since, .. } => Some(*pending_since),
        }
    }

    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending_since().is_some()
    }

    #[must_use]
    pub const fn is_synchronizing(&self) -> bool {
        match self {
            Self::Synchronizing { .. }
            | Self::SynchronizingFailed { .. }
            | Self::SynchronizingSucceeded { .. }
            | Self::SynchronizingAborted { .. } => true,
            Self::Void
            | Self::NestedMusicDirectoriesConflict { .. }
            | Self::RestoringFromMusicDirectoryFailed { .. }
            | Self::LoadingFailed { .. }
            | Self::RestoringFromMusicDirectory { .. }
            | Self::Loading { .. }
            | Self::Ready { .. } => false,
        }
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
                loaded_before: None,
                ..
            }
            | Self::LoadingFailed {
                loaded_before: None,
                ..
            } => None,
            Self::Loading {
                loaded_before: Some(loaded_before),
                ..
            }
            | Self::LoadingFailed {
                loaded_before: Some(loaded_before),
                ..
            } => vfs_music_dir(loaded_before),
            Self::Synchronizing { entity, .. }
            | Self::SynchronizingFailed { entity, .. }
            | Self::SynchronizingSucceeded { entity, .. }
            | Self::SynchronizingAborted { entity, .. }
            | Self::Ready { entity, .. } => vfs_music_dir(&entity.body),
            Self::RestoringFromMusicDirectory {
                state: RestoreState { music_dir, .. },
                ..
            }
            | Self::RestoringFromMusicDirectoryFailed {
                state: RestoreState { music_dir, .. },
                ..
            }
            | Self::NestedMusicDirectoriesConflict {
                state: RestoreState { music_dir, .. },
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
            Self::Void
            | Self::RestoringFromMusicDirectory { .. }
            | Self::RestoringFromMusicDirectoryFailed { .. }
            | Self::NestedMusicDirectoriesConflict { .. } => None,
            Self::Loading {
                entity_uid,
                loaded_before,
                ..
            }
            | Self::LoadingFailed {
                entity_uid,
                loaded_before,
                ..
            } => Some((entity_uid, loaded_before.as_ref())),
            Self::Synchronizing { entity, .. }
            | Self::SynchronizingFailed { entity, .. }
            | Self::SynchronizingSucceeded { entity, .. }
            | Self::SynchronizingAborted { entity, .. }
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
            Self::RestoringFromMusicDirectoryFailed { error, .. } => Some(error.as_str()),
            Self::LoadingFailed { error, .. } | Self::SynchronizingFailed { error, .. } => {
                Some(error.as_str())
            }
            Self::Void
            | Self::RestoringFromMusicDirectory { .. }
            | Self::NestedMusicDirectoriesConflict { .. }
            | Self::Loading { .. }
            | Self::Synchronizing { .. }
            | Self::SynchronizingSucceeded { .. }
            | Self::SynchronizingAborted { .. }
            | Self::Ready { .. } => None,
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
            Self::NestedMusicDirectoriesConflict {
                state: RestoreState {
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
        let state = RestoreState {
            kind: new_kind.map(Into::into),
            music_dir: new_music_dir.into_owned(),
            restore_entity,
            nested_music_dirs,
        };
        let new_self = Self::RestoringFromMusicDirectory {
            state,
            pending_since: Instant::now(),
        };
        *self = new_self;
        Ok(())
    }

    fn refresh_from_db(&mut self) -> Result<RefreshStateFromDbParams, StateUnchanged> {
        let old_self = std::mem::replace(self, Self::Void);
        let (entity_uid, loaded_before) = match old_self {
            Self::Void => {
                return Err(StateUnchanged);
            }
            Self::NestedMusicDirectoriesConflict { state, .. }
            | Self::RestoringFromMusicDirectory { state, .. }
            | Self::RestoringFromMusicDirectoryFailed { state, .. } => {
                let params = RefreshStateFromDbParams {
                    entity_uid: None,
                    restore: Some(state.clone()),
                };
                *self = Self::RestoringFromMusicDirectory {
                    state,
                    pending_since: Instant::now(),
                };
                return Ok(params);
            }
            Self::LoadingFailed {
                entity_uid,
                loaded_before,
                ..
            } => (entity_uid, loaded_before),
            Self::Ready { entity, .. }
            | Self::SynchronizingFailed { entity, .. }
            | Self::SynchronizingSucceeded { entity, .. }
            | Self::SynchronizingAborted { entity, .. } => {
                (entity.raw.hdr.uid, Some(entity.raw.body))
            }
            _ => {
                log::warn!("Illegal state for refreshing from database: {old_self:?}");
                *self = old_self;
                return Err(StateUnchanged);
            }
        };
        let params = self.refresh_from_db_unchecked(entity_uid, loaded_before);
        Ok(params)
    }

    #[must_use]
    fn refresh_from_db_unchecked(
        &mut self,
        entity_uid: EntityUid,
        loaded_before: Option<Collection>,
    ) -> RefreshStateFromDbParams {
        debug_assert!(matches!(self, Self::Void));
        let params = RefreshStateFromDbParams {
            entity_uid: Some(entity_uid.clone()),
            restore: None,
        };
        let new_self = Self::Loading {
            entity_uid,
            loaded_before,
            pending_since: Instant::now(),
        };
        *self = new_self;
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
        let new_self = Self::Synchronizing {
            entity,
            pending_since: Instant::now(),
        };
        *self = new_self;
        Ok(entity_uid)
    }

    #[must_use]
    fn loading_succeeded(entity_with_summary: EntityWithSummary) -> Self {
        let EntityWithSummary { entity, summary } = entity_with_summary;
        if let Some(summary) = summary {
            State::Ready { entity, summary }
        } else {
            // Should never happen
            let entity_uid = entity.raw.hdr.uid;
            let loaded_before = Some(entity.raw.body);
            State::LoadingFailed {
                entity_uid,
                loaded_before,
                error: "no summary".to_owned(),
            }
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
        let next_state = match result {
            Ok(next_state) => {
                if *self == next_state {
                    return Err(StateUnchanged);
                }
                log::debug!("Refreshed state from database: {next_state:?}");
                next_state
            }
            Err(err) => {
                let error = err.to_string();
                match self {
                    State::RestoringFromMusicDirectory { state, .. } => {
                        log::warn!("Restoring from music directory failed: {error}");
                        let error = RestoreFromMusicDirectoryError::Other(error);
                        State::RestoringFromMusicDirectoryFailed {
                            state: std::mem::take(state),
                            error,
                        }
                    }
                    State::Loading {
                        entity_uid,
                        loaded_before,
                        ..
                    } => {
                        log::warn!("Loading failed: {error}");
                        State::LoadingFailed {
                            entity_uid: std::mem::take(entity_uid),
                            loaded_before: loaded_before.take(),
                            error,
                        }
                    }
                    _ => unreachable!(),
                }
            }
        };
        debug_assert_ne!(*self, next_state);
        *self = next_state;
        Ok(())
    }

    fn synchronize_vfs_task_joined(
        &mut self,
        joined_task: JoinedTask<SynchronizeVfsResult>,
        continuation: SynchronizeVfsTaskContinuation,
    ) -> Result<Option<Outcome>, StateUnchanged> {
        let SynchronizeVfsTaskContinuation { pending_state } = continuation;
        if pending_state != *self {
            log::warn!(
                "State changed while synchronizing: expected {pending_state:?}, actual {self:?}",
            );
            return Err(StateUnchanged);
        }
        let State::Synchronizing {
            entity,
            pending_since: _,
        } = pending_state
        else {
            unreachable!("illegal state");
        };
        let mut outcome = None;
        let next_state = match joined_task {
            JoinedTask::Cancelled => State::SynchronizingAborted { entity },
            JoinedTask::Completed(Ok(ok)) => {
                outcome = Some(ok);
                State::SynchronizingSucceeded { entity }
            }
            JoinedTask::Completed(Err(err)) | JoinedTask::Panicked(err) => {
                let error = err.to_string();
                State::SynchronizingFailed { entity, error }
            }
        };
        debug_assert_ne!(*self, next_state);
        *self = next_state;
        Ok(outcome)
    }

    /// Map an URL to the corresponding content path.
    ///
    /// Example: Map a local file path to a content path within the collection
    /// for excluding it during synchronization. These paths are provided as
    /// relative, platform-independent paths and not as absolute paths in the
    /// local file system.
    ///
    /// Returns `None` if no collection is available or if the URL
    /// has no corresponding content path within the collection.
    ///
    /// The root URL of the collection can be overridden by `override_root_url`.
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

#[derive(Debug)]
pub struct SynchronizeVfsTaskContinuation {
    pending_state: State,
}

pub type SynchronizeVfsResult = anyhow::Result<batch::synchronize_collection_vfs::Outcome>;

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
        joined_task: JoinedTask<SynchronizeVfsResult>,
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
    restore: Option<RestoreState>,
}

async fn refresh_state_from_db<E>(env: E, params: RefreshStateFromDbParams) -> anyhow::Result<State>
where
    E: AsRef<Environment> + Send + 'static,
{
    let RefreshStateFromDbParams {
        entity_uid,
        restore,
    } = params;
    let entity_with_summary = if let Some(entity_uid) = entity_uid.as_ref() {
        refresh_entity_from_db(env.as_ref(), entity_uid.clone()).await?
    } else {
        None
    };
    let Some(restore) = restore else {
        return Ok(entity_with_summary.map_or_else(
            || {
                if let Some(entity_uid) = entity_uid {
                    State::LoadingFailed {
                        entity_uid,
                        loaded_before: None,
                        error: "not found".to_owned(),
                    }
                } else {
                    State::Void
                }
            },
            State::loading_succeeded,
        ));
    };
    if let Some(entity_with_summary) = entity_with_summary {
        let RestoreState {
            kind, music_dir, ..
        } = &restore;
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
    restore.restore(env.as_ref()).await
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
pub struct SynchronizeVfsTask {
    started_at: Instant,
    progress: Subscriber<Option<Progress>>,
    outcome: Subscriber<Option<Outcome>>,
    abort_flag: Arc<AtomicBool>,
    abort_handle: tokio::task::AbortHandle,
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
        let progress = progress_pub.subscribe();
        let outcome_pub = Publisher::new(None);
        let outcome = outcome_pub.subscribe();
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
        let abort_handle = join_handle.abort_handle();
        let state = Arc::clone(state);
        // The join task is responsible for updating the state eventually and
        // cannot be aborted! It completes after the main task completed.
        let join_task = async move {
            let joined_task = JoinedTask::join(join_handle).await;
            log::debug!("Synchronize music directory task joined: {joined_task:?}");
            let result = state.synchronize_vfs_task_joined(joined_task, continuation);
            if let Ok(outcome) = result {
                outcome_pub.write(outcome);
            }
        };
        rt.spawn(join_task);
        Ok(Self {
            started_at,
            progress,
            outcome,
            abort_flag,
            abort_handle,
        })
    }

    #[must_use]
    pub const fn started_at(&self) -> Instant {
        self.started_at
    }

    #[must_use]
    pub const fn progress(&self) -> &Subscriber<Option<Progress>> {
        &self.progress
    }

    #[must_use]
    pub const fn outcome(&self) -> &Subscriber<Option<Outcome>> {
        &self.outcome
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.abort_handle.is_finished()
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::Relaxed);
        self.abort_handle.abort();
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
