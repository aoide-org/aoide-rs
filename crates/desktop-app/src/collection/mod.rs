// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, path::Path};

use discro::{new_pubsub, Publisher, Ref, Subscriber};
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
    collection::{EntityWithSummary, LoadScope},
    media::SyncMode,
};
use aoide_media::io::import::ImportTrackConfig;
use aoide_repo::collection::{KindFilter, MediaSourceRootUrlFilter};

use crate::{environment::Handle, fs::DirPath};

pub mod tasklet;

#[must_use]
pub fn vfs_root_url(collection: &Collection) -> Option<&BaseUrl> {
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
            |path_buf| Some(DirPath::from_owned(path_buf)),
        )
    })
}

#[derive(Debug, Clone, Copy)]
pub enum NestedMusicDirectoriesStrategy {
    /// Allow one collection per music directory without restrictions
    /// on nesting.
    Permit,

    /// Prevent the creation of new collections for a music directory
    /// if collections for sub-directories already exist. Instead
    /// select an existing collection with the closest match.
    Deny,
}

pub async fn try_refresh_entity_from_db(
    handle: &Handle,
    entity_uid: EntityUid,
    load_scope: LoadScope,
) -> anyhow::Result<Option<EntityWithSummary>> {
    aoide_backend_embedded::collection::try_load_one(
        handle.db_gatekeeper(),
        entity_uid.clone(),
        load_scope,
    )
    .await
    .map(|entity_with_summary| {
        if entity_with_summary.is_some() {
            log::info!("Reloaded collection with UID {entity_uid}");
        } else {
            log::warn!("Collection with UID {entity_uid} not found");
        }
        entity_with_summary
    })
    .map_err(Into::into)
}

pub async fn restore_or_create_entity_from_db(
    handle: &Handle,
    kind: Option<String>,
    music_dir: &Path,
    nested_music_dirs: NestedMusicDirectoriesStrategy,
    load_scope: LoadScope,
) -> anyhow::Result<State> {
    let root_url =
        BaseUrl::try_autocomplete_from(Url::from_directory_path(music_dir).map_err(|()| {
            anyhow::anyhow!("unrecognized music directory: {}", music_dir.display())
        })?)?;
    let music_dir = root_url
        .to_file_path()
        .map_err(|()| anyhow::anyhow!("invalid music directory"))?;
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
        kind: Some(kind.to_owned().into()),
    });
    let candidates = aoide_backend_embedded::collection::load_all(
        handle.db_gatekeeper(),
        kind_filter.clone(),
        Some(media_source_root_url_filter),
        load_scope,
        None,
    )
    .await?;
    log::info!(
        "Found {num_candidates} existing collection candidate(s)",
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
            "Skipping collection with UID {uid}: {collection:?}",
            uid = candidate.entity.hdr.uid,
            collection = candidate.entity.body
        );
    }
    if let Some(selected_candidate) = selected_candidate {
        return Ok(State::Ready(selected_candidate));
    }
    if !matches!(nested_music_dirs, NestedMusicDirectoriesStrategy::Permit) {
        // Search for an existing collection with a root directory
        // that is a child of the music directory.
        let candidates = aoide_backend_embedded::collection::load_all(
            handle.db_gatekeeper(),
            kind_filter,
            Some(MediaSourceRootUrlFilter::Prefix(root_url.clone())),
            LoadScope::Entity,
            None,
        )
        .await?;
        return Ok(State::NestedMusicDirectories {
            kind,
            music_dir: DirPath::from_owned(music_dir),
            candidates,
        });
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
        aoide_backend_embedded::collection::create(handle.db_gatekeeper(), new_collection)
            .await?
            .raw
            .hdr
            .uid;
    // Reload the newly created entity with its summary
    aoide_backend_embedded::collection::load_one(handle.db_gatekeeper(), entity_uid, load_scope)
        .await
        .map(State::Ready)
        .map_err(Into::into)
}

/// A light-weight tag that denotes the [`State`] variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTag {
    Initial,
    PendingMusicDir,
    PendingEntityUid,
    Ready,
    NestedMusicDirectories,
}

impl StateTag {
    #[must_use]
    pub const fn is_idle(&self) -> bool {
        match self {
            Self::Initial | Self::Ready | Self::NestedMusicDirectories => true,
            Self::PendingMusicDir | Self::PendingEntityUid => false,
        }
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum State {
    #[default]
    Initial,
    PendingMusicDir {
        kind: Option<String>,
        music_dir: DirPath<'static>,
    },
    PendingEntityUid {
        kind: Option<String>,
        entity_uid: EntityUid,
    },
    Ready(EntityWithSummary),
    NestedMusicDirectories {
        kind: Option<String>,
        music_dir: DirPath<'static>,
        candidates: Vec<EntityWithSummary>,
    },
}

impl State {
    #[must_use]
    pub const fn state_tag(&self) -> StateTag {
        match self {
            Self::Initial => StateTag::Initial,
            Self::Ready(_) => StateTag::Ready,
            Self::PendingMusicDir { .. } => StateTag::PendingMusicDir,
            Self::PendingEntityUid { .. } => StateTag::PendingEntityUid,
            Self::NestedMusicDirectories { .. } => StateTag::NestedMusicDirectories,
        }
    }

    #[must_use]
    pub const fn is_idle(&self) -> bool {
        self.state_tag().is_idle()
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.state_tag().is_ready()
    }

    pub fn reset(&mut self) -> bool {
        if matches!(self, Self::Initial) {
            return false;
        }
        let reset = Self::Initial;
        log::debug!("Resetting state: {self:?} -> {reset:?}");
        *self = reset;
        true
    }

    #[must_use]
    pub fn music_dir(&self) -> Option<DirPath<'_>> {
        match self {
            Self::Initial | Self::PendingEntityUid { .. } => None,
            Self::PendingMusicDir { music_dir, .. } => Some(music_dir.borrowed()),
            Self::Ready(entity_with_summary) => vfs_music_dir(&entity_with_summary.entity.body),
            Self::NestedMusicDirectories { music_dir, .. } => Some(music_dir.borrowed()),
        }
    }

    pub fn update_music_dir(
        &mut self,
        new_kind: Option<Cow<'static, str>>,
        new_music_dir: DirPath<'_>,
    ) -> bool {
        match self {
            Self::Initial => (),
            Self::PendingMusicDir { .. } | Self::PendingEntityUid { .. } => {
                // Updating the music directory again while already pending is not allowed
                log::error!("Illegal state when updating music directory: {self:?}");
                return false;
            }
            Self::Ready(entity_with_summary) => {
                // When set the `kind` controls the selection of collections by music directory.
                if new_kind.is_none()
                    || new_kind.as_deref() == entity_with_summary.entity.body.kind.as_deref()
                {
                    let vfs_music_dir = vfs_music_dir(&entity_with_summary.entity.body);
                    if vfs_music_dir.as_ref() == Some(&new_music_dir) {
                        // Unchanged
                        log::debug!(
                            "Music directory unchanged and not updated: {}",
                            new_music_dir.display()
                        );
                        return false;
                    }
                }
            }
            Self::NestedMusicDirectories {
                kind, music_dir, ..
            } => {
                // When set the `kind` controls the selection of collections by music directory.
                if (new_kind.is_none() || new_kind.as_deref() == kind.as_deref())
                    && music_dir.borrowed() == new_music_dir
                {
                    // Unchanged
                    log::debug!(
                        "Music directory unchanged and not updated: {}",
                        new_music_dir.display()
                    );
                    return false;
                }
            }
        }
        let pending = Self::PendingMusicDir {
            kind: new_kind.map(Into::into),
            music_dir: new_music_dir.to_path_buf().into(),
        };
        log::debug!("Updating state: {self:?} -> {pending:?}");
        *self = pending;
        true
    }

    pub fn reset_to_pending(&mut self) -> bool {
        match self {
            Self::Initial | Self::PendingMusicDir { .. } | Self::PendingEntityUid { .. } => false,
            Self::Ready(EntityWithSummary { entity, .. }) => {
                let kind = entity.body.kind.take();
                let entity_uid = entity.hdr.uid.clone();
                let pending = Self::PendingEntityUid { kind, entity_uid };
                log::debug!("Resetting state to pending: {self:?} -> {pending:?}");
                *self = pending;
                true
            }
            Self::NestedMusicDirectories {
                kind, music_dir, ..
            } => {
                let kind = std::mem::take(kind).map(Into::into);
                let music_dir = std::mem::take(music_dir);
                let pending = Self::PendingMusicDir { kind, music_dir };
                log::debug!("Resetting state to pending: {self:?} -> {pending:?}");
                *self = pending;
                true
            }
        }
    }

    #[must_use]
    pub fn entity_uid(&self) -> Option<&EntityUid> {
        match self {
            Self::Initial | Self::PendingMusicDir { .. } | Self::NestedMusicDirectories { .. } => {
                None
            }
            Self::PendingEntityUid { entity_uid, .. } => Some(entity_uid),
            Self::Ready(ready) => Some(&ready.entity.hdr.uid),
        }
    }

    #[must_use]
    pub fn entity(&self) -> Option<&Entity> {
        match self {
            Self::Initial
            | Self::PendingMusicDir { .. }
            | Self::PendingEntityUid { .. }
            | Self::NestedMusicDirectories { .. } => None,
            Self::Ready(ready) => Some(&ready.entity),
        }
    }

    #[must_use]
    fn replace(&mut self, mut replacement: State) -> bool {
        // Only invoked while pending
        debug_assert!(!self.is_idle());
        if self == &replacement {
            return false;
        }
        log::debug!("Replacing state: {self:?} -> {replacement:?}");
        std::mem::swap(self, &mut replacement);
        true
    }
}

/// Manages the mutable, observable state
#[derive(Debug)]
pub struct ObservableState {
    state_pub: Publisher<State>,
}

impl ObservableState {
    #[must_use]
    pub fn new(initial_state: State) -> Self {
        let (state_pub, _) = new_pubsub(initial_state);
        Self { state_pub }
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, State> {
        self.state_pub.read()
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<State> {
        self.state_pub.subscribe()
    }

    #[allow(clippy::must_use_candidate)]
    pub fn modify(&self, modify_state: impl FnOnce(&mut State) -> bool) -> bool {
        self.state_pub.modify(modify_state)
    }

    pub async fn update_music_dir(
        &self,
        handle: &Handle,
        kind: Option<Cow<'static, str>>,
        new_music_dir: Option<DirPath<'_>>,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> anyhow::Result<bool> {
        let modified = if let Some(new_music_dir) = new_music_dir {
            log::debug!("Updating music directory: {}", new_music_dir.display());
            if self.modify(|state| state.update_music_dir(kind.clone(), new_music_dir)) {
                self.refresh_from_db(handle, nested_music_dirs).await?;
                true
            } else {
                false
            }
        } else {
            log::debug!("Resetting music directory");
            self.modify(State::reset)
        };
        Ok(modified)
    }

    pub async fn refresh_from_db(
        &self,
        handle: &Handle,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> anyhow::Result<()> {
        let (entity_uid, kind, music_dir) = match &*self.read() {
            State::PendingMusicDir { kind, music_dir } => {
                (None, kind.clone(), Some(music_dir.clone()))
            }
            State::PendingEntityUid { kind, entity_uid } => {
                (Some(entity_uid.clone()), kind.clone(), None)
            }
            State::Ready(entity_with_summary) => {
                let entity_uid = Some(entity_with_summary.entity.hdr.uid.clone());
                let kind = entity_with_summary.entity.body.kind.clone();
                let music_dir = vfs_music_dir(&entity_with_summary.entity.body);
                (entity_uid, kind, music_dir)
            }
            State::NestedMusicDirectories {
                kind, music_dir, ..
            } => (None, kind.clone(), Some(music_dir.clone())),
            _ => {
                anyhow::bail!("Illegal state when refreshing from database: {self:?}");
            }
        };
        let task = RefreshingStateTask::new(
            entity_uid,
            kind.map(Into::into),
            music_dir,
            nested_music_dirs,
        )?;
        let refreshed = task.execute(handle).await?;
        log::debug!("Refreshed state: {refreshed:?}");
        self.modify(|state| state.replace(refreshed));
        Ok(())
    }
}

impl Default for ObservableState {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[derive(Debug)]
pub struct RefreshingStateTask {
    entity_uid: Option<EntityUid>,
    kind: Option<Cow<'static, str>>,
    music_dir: Option<DirPath<'static>>,
    nested_music_dirs: NestedMusicDirectoriesStrategy,
}

impl RefreshingStateTask {
    pub fn new(
        entity_uid: Option<EntityUid>,
        kind: Option<Cow<'static, str>>,
        music_dir: Option<DirPath<'static>>,
        nested_music_dirs: NestedMusicDirectoriesStrategy,
    ) -> anyhow::Result<Self> {
        if entity_uid.is_none() && music_dir.is_none() {
            anyhow::bail!(
                "Neither entity UID nor music directory available for refreshing the state"
            );
        };
        Ok(Self {
            entity_uid,
            kind,
            music_dir,
            nested_music_dirs,
        })
    }

    pub async fn execute(self, handle: &Handle) -> anyhow::Result<State> {
        let Self {
            music_dir,
            entity_uid,
            kind,
            nested_music_dirs,
        } = self;
        let load_scope = LoadScope::EntityWithSummary;
        let entity_with_summary = if let Some(entity_uid) = entity_uid.as_ref() {
            try_refresh_entity_from_db(handle, entity_uid.clone(), load_scope).await?
        } else {
            None
        };
        if let Some(entity_with_summary) = entity_with_summary {
            if kind.is_none() || kind.as_deref() == entity_with_summary.entity.body.kind.as_deref()
            {
                if let Some(expected_music_dir) = &music_dir {
                    let actual_music_dir = vfs_music_dir(&entity_with_summary.entity.body);
                    if Some(expected_music_dir) == actual_music_dir.as_ref() {
                        return Ok(State::Ready(entity_with_summary));
                    }
                } else {
                    return Ok(State::Ready(entity_with_summary));
                }
            }
            log::debug!(
                "Discarding collection with UID {uid}",
                uid = entity_with_summary.entity.hdr.uid
            );
        }
        let music_dir = music_dir.ok_or_else(|| anyhow::anyhow!("no music directory"))?;
        restore_or_create_entity_from_db(
            handle,
            kind.map(Cow::into_owned),
            &music_dir,
            nested_music_dirs,
            load_scope,
        )
        .await
    }
}

pub async fn synchronize_vfs<ReportProgressFn>(
    handle: &Handle,
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
        handle.db_gatekeeper(),
        entity_uid,
        params,
        std::convert::identity,
        report_progress_fn,
    )
    .await
    .map_err(Into::into)
}
