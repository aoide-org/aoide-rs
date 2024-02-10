// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    time::Instant,
};

use url::Url;

use aoide_backend_embedded::batch::{
    self,
    synchronize_collection_vfs::{
        OrphanedMediaSources, UnsynchronizedTracks, UntrackedFiles, UntrackedMediaSources,
    },
};
use aoide_core::{
    collection::{Collection, Entity, EntityUid, MediaSourceConfig},
    media::content::ContentPathConfig,
    util::url::BaseUrl,
};
use aoide_core_api::{
    collection::{EntityWithSummary, LoadScope, Summary},
    media::SyncMode,
};
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_repo::collection::{KindFilter, MediaSourceRootUrlFilter};

use crate::{fs::DirPath, Environment, Handle, Observable, ObservableReader, ObservableRef};

pub mod tasklet;

#[must_use]
pub const fn vfs_root_url(collection: &Collection) -> Option<&BaseUrl> {
    if let ContentPathConfig::VirtualFilePath { root_url } =
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestedMusicDirectoriesStrategy {
    /// Allow one collection per music directory without restrictions
    /// on nesting.
    Permit,

    /// Prevent the creation of new collections for a music directory
    /// if collections for sub-directories already exist. Instead
    /// select an existing collection with the closest match.
    Deny,
}

async fn try_refresh_entity_from_db(
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

/// A light-weight tag that denotes the [`State`] variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StateTag {
    #[default]
    Void,
    RestoringOrCreatingFromMusicDirectory,
    RestoringOrCreatingFromMusicDirectoryFailed,
    NestedMusicDirectoriesConflict,
    Loading,
    LoadingFailed,
    Synchronizing,
    SynchronizingFailed,
    Ready,
}

impl StateTag {
    /// Indicates if this is a transitional state while an effect is running.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        match self {
            Self::Void
            | Self::RestoringOrCreatingFromMusicDirectoryFailed
            | Self::NestedMusicDirectoriesConflict
            | Self::LoadingFailed
            | Self::SynchronizingFailed
            | Self::Ready => false,
            Self::RestoringOrCreatingFromMusicDirectory | Self::Loading | Self::Synchronizing => {
                true
            }
        }
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOrCreateState {
    kind: Option<String>,
    music_dir: DirPath<'static>,
    create_new_entity_if_not_found: bool,
    nested_music_dirs: NestedMusicDirectoriesStrategy,
}

impl Default for RestoreOrCreateState {
    fn default() -> Self {
        Self {
            kind: None,
            music_dir: Default::default(),
            create_new_entity_if_not_found: false,
            nested_music_dirs: NestedMusicDirectoriesStrategy::Deny,
        }
    }
}

fn parse_music_dir_path(path: &Path) -> anyhow::Result<(BaseUrl, PathBuf)> {
    let root_url = BaseUrl::try_autocomplete_from(
        Url::from_directory_path(path)
            .map_err(|()| anyhow::anyhow!("unrecognized music directory: {}", path.display()))?,
    )?;
    let root_path = root_url
        .to_file_path()
        .map_err(|()| anyhow::anyhow!("invalid music directory"))?;
    Ok((root_url, root_path))
}

impl RestoreOrCreateState {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::too_many_lines)] // TODO
    pub async fn restore_or_create(self, env: &Environment) -> anyhow::Result<State> {
        let Self {
            kind,
            music_dir,
            create_new_entity_if_not_found,
            nested_music_dirs,
        } = self;
        let (root_url, music_dir) = parse_music_dir_path(&music_dir)?;
        // Search for an existing collection with a root directory
        // that contains the music directory.
        let media_source_root_url_filter = match nested_music_dirs {
            NestedMusicDirectoriesStrategy::Permit => {
                MediaSourceRootUrlFilter::Equal(Some(root_url.clone()))
            }
            NestedMusicDirectoriesStrategy::Deny => {
                MediaSourceRootUrlFilter::PrefixOf(root_url.clone())
            }
        };
        let kind_filter = kind.as_ref().map(|kind| KindFilter {
            kind: Some(kind.clone().into()),
        });
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
            let state = RestoreOrCreateState {
                kind,
                music_dir: music_dir.into(),
                create_new_entity_if_not_found,
                nested_music_dirs,
            };
            return Ok(State::NestedMusicDirectoriesConflict { state, candidates });
        }
        // Create a new collection
        let new_collection = Collection {
            title: music_dir.display().to_string(),
            media_source_config: MediaSourceConfig {
                content_path: ContentPathConfig::VirtualFilePath { root_url },
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

/// State of a single collection that is based on directory in the
/// local directory using a virtual file system (VFS) for content paths.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum State {
    #[default]
    Void,
    RestoringOrCreatingFromMusicDirectory {
        state: RestoreOrCreateState,
        pending_since: Instant,
    },
    RestoringOrCreatingFromMusicDirectoryFailed {
        state: RestoreOrCreateState,
        error: String,
    },
    NestedMusicDirectoriesConflict {
        state: RestoreOrCreateState,
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
        pending_since: Instant,
        error: String,
    },
    Ready {
        entity: Entity,
        summary: Summary,
    },
}

impl State {
    #[must_use]
    pub const fn state_tag(&self) -> StateTag {
        match self {
            Self::Void => StateTag::Void,
            Self::RestoringOrCreatingFromMusicDirectory { .. } => {
                StateTag::RestoringOrCreatingFromMusicDirectory
            }
            Self::RestoringOrCreatingFromMusicDirectoryFailed { .. } => {
                StateTag::RestoringOrCreatingFromMusicDirectoryFailed
            }
            Self::NestedMusicDirectoriesConflict { .. } => StateTag::NestedMusicDirectoriesConflict,
            Self::Loading { .. } => StateTag::Loading,
            Self::LoadingFailed { .. } => StateTag::LoadingFailed,
            Self::Synchronizing { .. } => StateTag::Synchronizing,
            Self::SynchronizingFailed { .. } => StateTag::SynchronizingFailed,
            Self::Ready { .. } => StateTag::Ready,
        }
    }

    #[must_use]
    pub const fn pending_since(&self) -> Option<Instant> {
        match self {
            Self::Void
            | Self::NestedMusicDirectoriesConflict { .. }
            | Self::RestoringOrCreatingFromMusicDirectoryFailed { .. }
            | Self::LoadingFailed { .. }
            | Self::SynchronizingFailed { .. }
            | Self::Ready { .. } => None,
            Self::RestoringOrCreatingFromMusicDirectory { pending_since, .. }
            | Self::Loading { pending_since, .. }
            | Self::Synchronizing { pending_since, .. } => Some(*pending_since),
        }
    }

    #[must_use]
    pub fn is_pending(&self) -> bool {
        let is_pending = self.state_tag().is_pending();
        debug_assert_eq!(is_pending, self.pending_since().is_some());
        is_pending
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.state_tag().is_ready()
    }

    pub fn reset(&mut self) -> bool {
        if matches!(self, Self::Void) {
            // No effect
            return false;
        }
        let reset = Self::Void;
        log::debug!("Resetting state: {self:?} -> {reset:?}");
        *self = reset;
        true
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
            | Self::Ready { entity, .. } => vfs_music_dir(&entity.body),
            Self::RestoringOrCreatingFromMusicDirectory {
                state: RestoreOrCreateState { music_dir, .. },
                ..
            }
            | Self::RestoringOrCreatingFromMusicDirectoryFailed {
                state: RestoreOrCreateState { music_dir, .. },
                ..
            }
            | Self::NestedMusicDirectoriesConflict {
                state: RestoreOrCreateState { music_dir, .. },
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
            | Self::RestoringOrCreatingFromMusicDirectory { .. }
            | Self::RestoringOrCreatingFromMusicDirectoryFailed { .. }
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
            Self::RestoringOrCreatingFromMusicDirectoryFailed { error, .. }
            | Self::LoadingFailed { error, .. }
            | Self::SynchronizingFailed { error, .. } => Some(error.as_str()),
            Self::Void
            | Self::RestoringOrCreatingFromMusicDirectory { .. }
            | Self::NestedMusicDirectoriesConflict { .. }
            | Self::Loading { .. }
            | Self::Synchronizing { .. }
            | Self::Ready { .. } => None,
        }
    }

    pub fn update_music_dir(
        &mut self,
        new_kind: Option<Cow<'_, str>>,
        new_music_dir: DirPath<'_>,
        create_new_entity_if_not_found: bool,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> bool {
        if self.is_pending() {
            log::warn!("Illegal state for updating directory: {self:?}");
            return false;
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
                        return false;
                    }
                }
            }
            Self::NestedMusicDirectoriesConflict {
                state:
                    RestoreOrCreateState {
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
                    return false;
                }
            }
            _ => {
                // Proceed without any checks.
            }
        }
        let state = RestoreOrCreateState {
            kind: new_kind.map(Into::into),
            music_dir: new_music_dir.into_owned(),
            create_new_entity_if_not_found,
            nested_music_dirs,
        };
        let new_self = Self::RestoringOrCreatingFromMusicDirectory {
            state,
            pending_since: Instant::now(),
        };
        *self = new_self;
        true
    }

    #[must_use]
    fn refresh_from_db(&mut self) -> Option<RefreshStateFromDbParams> {
        let old_self = std::mem::replace(self, Self::Void);
        let (entity_uid, loaded_before) = match old_self {
            Self::Void => {
                return None;
            }
            Self::NestedMusicDirectoriesConflict { state, .. }
            | Self::RestoringOrCreatingFromMusicDirectory { state, .. }
            | Self::RestoringOrCreatingFromMusicDirectoryFailed { state, .. } => {
                let params = RefreshStateFromDbParams {
                    entity_uid: None,
                    restore_or_create: Some(state.clone()),
                };
                *self = Self::RestoringOrCreatingFromMusicDirectory {
                    state,
                    pending_since: Instant::now(),
                };
                return Some(params);
            }
            Self::LoadingFailed {
                entity_uid,
                loaded_before,
                ..
            } => (entity_uid, loaded_before),
            Self::Ready { entity, .. } => (entity.raw.hdr.uid, Some(entity.raw.body)),
            _ => {
                log::warn!("Illegal state for refreshing from database: {old_self:?}");
                *self = old_self;
                return None;
            }
        };
        let params = RefreshStateFromDbParams {
            entity_uid: Some(entity_uid.clone()),
            restore_or_create: None,
        };
        let new_self = Self::Loading {
            entity_uid,
            loaded_before,
            pending_since: Instant::now(),
        };
        *self = new_self;
        Some(params)
    }

    pub fn synchronize(&mut self) -> bool {
        let old_self = std::mem::replace(self, Self::Void);
        let Self::Ready { entity, .. } = old_self else {
            log::warn!("Illegal state for synchronizing: {old_self:?}");
            *self = old_self;
            return false;
        };
        let new_self = Self::Synchronizing {
            entity,
            pending_since: Instant::now(),
        };
        *self = new_self;
        true
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
}

pub type StateSubscriber = discro::Subscriber<State>;

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

    pub fn reset(&self) -> bool {
        self.0.modify(State::reset)
    }

    pub async fn update_music_dir<'a>(
        &'a self,
        handle: &'a Handle,
        kind: Option<Cow<'static, str>>,
        new_music_dir: Option<DirPath<'static>>,
        create_new_entity_if_not_found: bool,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> bool {
        let Some(new_music_dir) = new_music_dir else {
            log::debug!("Resetting music directory");
            return self.0.modify(State::reset);
        };
        log::debug!(
            "Updating music directory: {new_music_dir}",
            new_music_dir = new_music_dir.display()
        );
        if !self.0.modify(|state| {
            state.update_music_dir(
                kind,
                new_music_dir,
                create_new_entity_if_not_found,
                nested_music_dirs,
            )
        }) {
            log::debug!("Music directory unchanged");
            return false;
        }
        log::debug!("Refreshing from database after music directory changed");
        self.refresh_from_db(handle).await;
        true
    }

    pub async fn refresh_from_db(&self, env: &Environment) -> bool {
        let mut pending_state_params = None;
        let modified = self.0.modify(|state| {
            let Some(params) = state.refresh_from_db() else {
                return false;
            };
            debug_assert!(state.is_pending());
            let pending_state = state.clone();
            pending_state_params = Some((pending_state, params));
            true
        });
        let Some((pending_state, params)) = pending_state_params else {
            return modified;
        };
        let refreshed_state_result = refresh_state_from_db(env, params).await;
        self.0.modify(|state| {
            if pending_state != *state {
                log::warn!(
                    "State changed while refreshing from database: expected {pending_state:?}, actual {state:?} - discarding {refreshed_state_result:?}",
                );
                return false;
            }
            match refreshed_state_result {
                Ok(refreshed_state) => {
                    if *state == refreshed_state {
                        return false;
                    }
                    log::debug!("Refreshed state from database: {refreshed_state:?}");
                    *state = refreshed_state;
                    true
                }
                Err(err) => {
                    let error = err.to_string();
                    let failed_state = match state {
                        State::RestoringOrCreatingFromMusicDirectory { state, .. } => {
                            State::RestoringOrCreatingFromMusicDirectoryFailed { state: std::mem::take(state), error }
                        }
                        State::Loading { entity_uid, loaded_before, .. } => {
                            State::LoadingFailed {
                                entity_uid: std::mem::take(entity_uid),
                                loaded_before: loaded_before.take(),
                                error,
                            }
                        }
                        _ => unreachable!(),
                    };
                    debug_assert_ne!(*state, failed_state);
                    *state = failed_state;
                    true
                }
            }
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
    fn read_observable(&self) -> ObservableStateRef<'_> {
        self.0.read_observable()
    }
}

#[derive(Debug, Clone)]
struct RefreshStateFromDbParams {
    entity_uid: Option<EntityUid>,
    restore_or_create: Option<RestoreOrCreateState>,
}

async fn refresh_state_from_db<'a>(
    env: &Environment,
    params: RefreshStateFromDbParams,
) -> anyhow::Result<State> {
    let RefreshStateFromDbParams {
        entity_uid,
        restore_or_create,
    } = params;
    let entity_with_summary = if let Some(entity_uid) = entity_uid.as_ref() {
        try_refresh_entity_from_db(env, entity_uid.clone()).await?
    } else {
        None
    };
    let Some(restore_or_create) = restore_or_create else {
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
        let RestoreOrCreateState {
            kind, music_dir, ..
        } = &restore_or_create;
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
    restore_or_create.restore_or_create(env).await
}

pub async fn synchronize_vfs<ReportProgressFn>(
    env: &Environment,
    entity_uid: EntityUid,
    import_track_config: ImportTrackConfig,
    report_progress_fn: ReportProgressFn,
) -> anyhow::Result<batch::synchronize_collection_vfs::Outcome>
where
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
        env.db_gatekeeper(),
        entity_uid,
        params,
        std::convert::identity,
        report_progress_fn,
    )
    .await
    .map_err(Into::into)
}
