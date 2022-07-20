// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, path::Path, sync::Arc};

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use discro::{new_pubsub, Publisher, Ref, Subscriber};
use url::Url;

use aoide_backend_embedded::batch;
use aoide_core::{
    collection::{Collection, Entity, EntityUid, MediaSourceConfig},
    media::content::ContentPathConfig,
    util::url::BaseUrl,
};
use aoide_core_api::collection::{EntityWithSummary, LoadScope, Summary};
use aoide_repo::{collection::MediaSourceRootUrlFilter, prelude::RepoError};

use crate::{
    fs::{DirPath, OwnedDirPath},
    Environment,
};

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
pub fn vfs_music_dir(collection: &Collection) -> Option<OwnedDirPath> {
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

pub async fn refresh_entity_with_summary_from_db(
    db_gatekeeper: &Gatekeeper,
    collection_uid: Option<&EntityUid>,
    music_dir: Option<&Path>,
    collection_kind: Option<&str>,
) -> anyhow::Result<EntityWithSummary> {
    let load_scope = LoadScope::EntityWithSummary;
    if let Some(entity_uid) = collection_uid {
        // Try to reload the current collection
        match aoide_backend_embedded::collection::load_one(
            db_gatekeeper,
            entity_uid.clone(),
            load_scope,
        )
        .await
        {
            Ok(collection_entity_with_summary) => {
                log::info!("Reloaded collection with UID {entity_uid}");
                return Ok(collection_entity_with_summary);
            }
            Err(aoide_backend_embedded::Error::Repository(RepoError::NotFound)) => {
                log::warn!("Collection with UID {entity_uid} not found");
                // Continue and find or create a new collection
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }
    let music_dir = music_dir.ok_or_else(|| anyhow::anyhow!("no music directory"))?;
    let root_url =
        BaseUrl::try_autocomplete_from(Url::from_directory_path(music_dir).map_err(|()| {
            anyhow::anyhow!("unrecognized music directory: {}", music_dir.display())
        })?)?;
    let music_dir = root_url
        .to_file_path()
        .map_err(|()| anyhow::anyhow!("invalid music directory"))?;
    // Search for an existing collection that matches the music directory
    let candidates = aoide_backend_embedded::collection::load_all(
        db_gatekeeper,
        collection_kind.map(ToOwned::to_owned),
        Some(MediaSourceRootUrlFilter::PrefixOf(root_url.clone())),
        load_scope,
        None,
    )
    .await?;
    log::info!(
        "Found {} existing collection candidate(s)",
        candidates.len()
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
            "Skipping collection with UID {}: {:?}",
            candidate.entity.hdr.uid,
            candidate.entity.body
        );
    }
    if let Some(selected_candidate) = selected_candidate {
        return Ok(selected_candidate);
    }
    // Create a new collection
    let new_collection = Collection {
        title: music_dir.display().to_string(),
        media_source_config: MediaSourceConfig {
            content_path: ContentPathConfig::VirtualFilePath { root_url },
        },
        kind: collection_kind.map(ToOwned::to_owned),
        notes: None,
        color: None,
    };
    let entity_uid = aoide_backend_embedded::collection::create(db_gatekeeper, new_collection)
        .await?
        .raw
        .hdr
        .uid;
    // Reload the newly created entity with its summary
    aoide_backend_embedded::collection::load_one(db_gatekeeper, entity_uid, load_scope)
        .await
        .map_err(Into::into)
}

#[derive(Debug)]
pub struct Ready {
    pub entity: Entity,
    pub summary: Option<Summary>,
}

impl Ready {
    #[must_use]
    pub fn vfs_root_url(&self) -> Option<&BaseUrl> {
        vfs_root_url(&self.entity.body)
    }

    #[must_use]
    pub fn vfs_music_dir(&self) -> Option<OwnedDirPath> {
        vfs_music_dir(&self.entity.body)
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum State {
    Initial,
    PendingMusicDir(OwnedDirPath),
    PendingEntityUid(EntityUid),
    Ready(Ready),
}

#[derive(Debug)]
pub struct RefreshingTask {
    entity_uid: Option<EntityUid>,
    music_dir: Option<OwnedDirPath>,
    collection_kind: Option<Cow<'static, str>>,
}

impl RefreshingTask {
    pub fn new(state: &State, collection_kind: Option<Cow<'static, str>>) -> anyhow::Result<Self> {
        let entity_uid = state.entity_uid().map(ToOwned::to_owned);
        let music_dir = state.music_dir().map(DirPath::into_owned);
        if entity_uid.is_none() && music_dir.is_none() {
            anyhow::bail!("Neither entity UID nor music directory available for refreshing");
        };
        Ok(Self {
            entity_uid,
            music_dir,
            collection_kind,
        })
    }

    pub async fn execute(self, db_gatekeeper: &Gatekeeper) -> anyhow::Result<EntityWithSummary> {
        let Self {
            music_dir,
            entity_uid,
            collection_kind,
        } = self;
        refresh_entity_with_summary_from_db(
            db_gatekeeper,
            entity_uid.as_ref(),
            music_dir.as_deref(),
            collection_kind.as_deref(),
        )
        .await
    }
}

impl State {
    #[must_use]
    pub const fn new() -> Self {
        Self::Initial
    }

    #[must_use]
    pub fn is_pending(&self) -> bool {
        match self {
            Self::Initial | Self::Ready(_) => false,
            Self::PendingMusicDir(_) | Self::PendingEntityUid(_) => true,
        }
    }

    pub fn reset(&mut self) -> bool {
        if matches!(self, Self::Initial) {
            return false;
        }
        *self = Self::Initial;
        true
    }

    #[must_use]
    pub fn music_dir(&self) -> Option<DirPath<'_>> {
        match self {
            Self::Initial | Self::PendingEntityUid(_) => None,
            Self::PendingMusicDir(music_dir) => Some(music_dir.borrowed()),
            Self::Ready(ready) => ready.vfs_music_dir(),
        }
    }

    pub fn update_music_dir(&mut self, new_music_dir: &Path) -> bool {
        match self {
            Self::Initial => (),
            Self::PendingMusicDir(_) | Self::PendingEntityUid(_) => {
                // Updating the music directory again while already pending is not allowed
                debug_assert!(false);
                log::error!("Illegal state when updating music directory: {:?}", self);
                return false;
            }
            Self::Ready(ready) => {
                let vfs_music_dir = ready.vfs_music_dir();
                // Using Path::as_os_str() os required to handle trailing slashes consistently!
                // https://www.reddit.com/r/rust/comments/ooh5wn/damn_trailing_slash/
                if vfs_music_dir.as_deref().map(Path::as_os_str) == Some(new_music_dir.as_os_str())
                {
                    // Unchanged
                    return false;
                }
                // If the music directory doesn't match that of the ready
                // collection then reset the collection back to pending.
            }
        }
        *self = Self::PendingMusicDir(new_music_dir.to_path_buf().into());
        true
    }

    pub fn reset_pending(&mut self) -> bool {
        match self {
            Self::Initial | Self::PendingMusicDir(_) | Self::PendingEntityUid(_) => false,
            Self::Ready(Ready { entity, .. }) => {
                let entity_uid = entity.hdr.uid.clone();
                *self = Self::PendingEntityUid(entity_uid);
                true
            }
        }
    }

    #[must_use]
    pub fn entity_uid(&self) -> Option<&EntityUid> {
        match self {
            Self::Initial | Self::PendingMusicDir(_) => None,
            Self::PendingEntityUid(entity_uid) => Some(entity_uid),
            Self::Ready(ready) => Some(&ready.entity.hdr.uid),
        }
    }

    #[must_use]
    pub fn entity(&self) -> Option<&Entity> {
        match self {
            Self::Initial | Self::PendingMusicDir(_) | Self::PendingEntityUid(_) => None,
            Self::Ready(ready) => Some(&ready.entity),
        }
    }

    #[must_use]
    pub fn refreshing_succeeded(&mut self, refreshed: EntityWithSummary) -> bool {
        match self {
            Self::Initial => {
                debug_assert!(false);
                log::error!("Illegal state when refreshing finished: {:?}", self);
                false
            }
            Self::PendingMusicDir(music_dir) => {
                let EntityWithSummary {
                    entity: new_entity,
                    summary: new_summary,
                } = refreshed;
                let ready = Ready {
                    entity: new_entity,
                    summary: new_summary,
                };
                if let Some(vfs_music_dir) = ready.vfs_music_dir() {
                    if !music_dir.starts_with(&*vfs_music_dir) {
                        log::warn!("Discarding refreshed collection with mismatching VFS music dir: expected = {}, actual = {}", music_dir.display(), vfs_music_dir.display());
                        return false;
                    }
                }
                *self = Self::Ready(ready);
                true
            }
            Self::PendingEntityUid(entity_uid) => {
                let EntityWithSummary {
                    entity: new_entity,
                    summary: new_summary,
                } = refreshed;
                let expected_entity_uid = entity_uid;
                let actual_entity_uid = &new_entity.hdr.uid;
                if expected_entity_uid != actual_entity_uid {
                    log::warn!("Discarding refreshed collection with mismatching UID: expected = {expected_entity_uid}, actual = {actual_entity_uid}");
                    return false;
                }
                let ready = Ready {
                    entity: new_entity,
                    summary: new_summary,
                };
                *self = Self::Ready(ready);
                true
            }
            Self::Ready(Ready { entity, summary }) => {
                let EntityWithSummary {
                    entity: new_entity,
                    summary: new_summary,
                } = refreshed;
                // Replace existing data
                *entity = new_entity;
                *summary = new_summary;
                true
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages the mutable, observable state
#[allow(missing_debug_implementations)]
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
        environment: &Environment,
        new_music_dir: Option<&Path>,
        collection_kind: Option<Cow<'static, str>>,
    ) -> anyhow::Result<bool> {
        let modified = if let Some(new_music_dir) = new_music_dir {
            if self.modify(|state| state.update_music_dir(new_music_dir)) {
                self.refresh_from_db(Arc::clone(environment.db_gatekeeper()), collection_kind)
                    .await?;
                true
            } else {
                false
            }
        } else {
            self.modify(State::reset)
        };
        Ok(modified)
    }

    pub async fn refresh_from_db(
        &self,
        db_gatekeeper: Arc<Gatekeeper>,
        collection_kind: Option<Cow<'static, str>>,
    ) -> anyhow::Result<()> {
        let task = RefreshingTask::new(&*self.read(), collection_kind)?;
        let refreshed = task.execute(&db_gatekeeper).await?;
        self.modify(|state| state.refreshing_succeeded(refreshed));
        Ok(())
    }
}

impl Default for ObservableState {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

pub async fn ingest_vfs<ReportProgressFn>(
    environment: &Environment,
    collection_uid: EntityUid,
    report_progress_fn: ReportProgressFn,
) -> anyhow::Result<batch::ingest_collection_vfs::Outcome>
where
    ReportProgressFn: FnMut(batch::ingest_collection_vfs::Progress) + Clone + Send + 'static,
{
    let params = batch::ingest_collection_vfs::Params {
        find_unsynchronized_tracks: true,
        find_untracked_files: true,
        import_track_config: Default::default(),
        max_depth: None,
        purge_orphaned_media_sources: true,
        purge_untracked_media_sources: true,
        root_url: None,
        sync_mode: None,
    };
    batch::ingest_collection_vfs::ingest_collection_vfs(
        &*environment.db_gatekeeper(),
        collection_uid,
        params,
        report_progress_fn,
    )
    .await
    .map_err(Into::into)
}
