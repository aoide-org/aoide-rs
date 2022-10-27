// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper::StatusCode;
use thiserror::Error;

use seed::{prelude::*, *};

use aoide_core::{collection::EntityUid as CollectionUid, track::Track};

use aoide_core_api::{
    media::{
        source::{
            purge_orphaned::Outcome as PurgeOrphanedMediaSourcesOutcome,
            purge_untracked::Outcome as PurgeUntrackedMediaSourcesOutcome,
        },
        tracker::{
            import_files::Outcome as ImportMediaFileOutcome,
            scan_directories::Outcome as ScanMediaDirectoriesOutcome,
            untrack_directories::Outcome as UntrackMediaDirectoriesOutcome,
            Progress as MediaTrackerProgress, Status as QueryMediaTrackerStatusOutcome,
        },
    },
    track::search::Params as SearchParams,
};

use aoide_core_json::{
    collection::{Collection as SerdeCollection, Entity as SerdeCollectionEntity},
    entity::{Entity as SerdeEntity, EntityHeader as SerdeEntityHeader},
    track::Track as SerdeTrack,
};

use aoide_core_api_json::{
    collection::{import_entity_with_summary, EntityWithSummary as CollectionEntityWithSummary},
    media::{
        source::{
            purge_orphaned::{
                Outcome as SerdePurgeOrphanedMediaSourcesOutcome,
                Params as SerdePurgeOrphanedMediaSourcesParams,
            },
            purge_untracked::{
                Outcome as SerdePurgeUntrackedMediaSourcesOutcome,
                Params as SerdePurgeUntrackedMediaSourcesParams,
            },
        },
        tracker::{
            import_files::{
                Outcome as SerdeImportMediaFileOutcome, Params as SerdeImportMediaFilesParams,
            },
            query_status::Params as SerdeQueryMediaTrackerStatusParams,
            scan_directories::Outcome as SerdeScanMediaDirectoriesOutcome,
            untrack_directories::{
                Outcome as SerdeUntrackMediaDirectoriesOutcome,
                Params as SerdeUntrackMediaDirectoriesParams,
            },
            FsTraversalParams as SerdeFsTraversalParams, Progress as SerdeMediaTrackerProgress,
            Status as SerdeQueryMediaTrackerStatusOutcome,
        },
    },
    Pagination as SerdePagination,
};

use crate::domain::*;

const BASE_URL: &str = "/api/";

impl TryFrom<CollectionEntityWithSummary> for CollectionItem {
    type Error = Error;

    fn try_from(from: CollectionEntityWithSummary) -> Result<Self> {
        import_entity_with_summary(from)
            .map(|(entity, summary)| Self { entity, summary })
            .map_err(Error::DataShape)
    }
}

pub(crate) async fn fetch_all_collections() -> Result<CollectionItems> {
    let url = format!("{BASE_URL}c");
    let response = fetch(url).await?;
    let content: Vec<SerdeCollectionEntity> = response.check_status()?.json().await?;
    content
        .into_iter()
        .map(|item| {
            item.try_into()
                .map(CollectionItem::without_summary)
                .map_err(Error::DataShape)
        })
        .collect()
}

pub(crate) async fn fetch_collection_with_summary(
    collection_uid: CollectionUid,
) -> Result<CollectionItem> {
    let url = format!("{BASE_URL}c/{collection_uid}?summary=true");
    let response = fetch(url).await?;
    let content: CollectionEntityWithSummary = response.check_status()?.json().await?;
    content.try_into()
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn create_collection(
    collection: impl Into<SerdeCollection>,
) -> Result<EntityHeader> {
    let url = format!("{BASE_URL}c");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&collection.into())?;
    let response = request.fetch().await?;
    let content: SerdeEntityHeader = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn update_collection(
    entity_header: impl Into<SerdeEntityHeader>,
    collection: impl TryInto<SerdeCollection, Error = anyhow::Error>,
) -> Result<EntityHeader> {
    let url = format!("{BASE_URL}c");
    let entity = SerdeEntity(
        entity_header.into(),
        collection.try_into().map_err(Error::DataShape)?,
    );
    let request = Request::new(url).method(Method::Put).json(&entity)?;
    let response = request.fetch().await?;
    let content: SerdeEntityHeader = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn delete_collection(entity_header: impl Into<SerdeEntityHeader>) -> Result<()> {
    let url = format!("{BASE_URL}c");
    let request = Request::new(url)
        .method(Method::Delete)
        .json(&entity_header.into())?;
    let response = request.fetch().await?.check_status()?;
    let status_code = response.check_status()?.status().code;
    debug_assert_eq!(StatusCode::NO_CONTENT, status_code);
    Ok(())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn scan_media_directories(
    collection_uid: CollectionUid,
    params: impl Into<SerdeFsTraversalParams>,
) -> Result<ScanMediaDirectoriesOutcome> {
    let url = format!("{BASE_URL}c/{collection_uid}/mt/scan-directories");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeScanMediaDirectoriesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn import_media_files(
    collection_uid: CollectionUid,
    params: impl Into<SerdeImportMediaFilesParams>,
) -> Result<ImportMediaFileOutcome> {
    let url = format!("{BASE_URL}c/{collection_uid}/mt/import-files");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeImportMediaFileOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn untrack_media_directories(
    collection_uid: CollectionUid,
    params: impl Into<SerdeUntrackMediaDirectoriesParams>,
) -> Result<UntrackMediaDirectoriesOutcome> {
    let url = format!("{BASE_URL}c/{collection_uid}/mt/untrack-directories");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeUntrackMediaDirectoriesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn purge_orphaned_media_sources(
    collection_uid: CollectionUid,
    params: impl Into<SerdePurgeOrphanedMediaSourcesParams>,
) -> Result<PurgeOrphanedMediaSourcesOutcome> {
    let url = format!("{BASE_URL}c/{collection_uid}/ms/purge-orphaned");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdePurgeOrphanedMediaSourcesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn purge_untracked_media_sources(
    collection_uid: CollectionUid,
    params: impl Into<SerdePurgeUntrackedMediaSourcesParams>,
) -> Result<PurgeUntrackedMediaSourcesOutcome> {
    let url = format!("{BASE_URL}c/{collection_uid}/ms/purge-untracked");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdePurgeUntrackedMediaSourcesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn query_media_tracker_status(
    collection_uid: CollectionUid,
    params: impl Into<SerdeQueryMediaTrackerStatusParams>,
) -> Result<QueryMediaTrackerStatusOutcome> {
    let url = format!("{BASE_URL}c/{collection_uid}/mt/query-status");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeQueryMediaTrackerStatusOutcome = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn get_media_tracker_progress() -> Result<MediaTrackerProgress> {
    let url = format!("{BASE_URL}mt/progress");
    let response = fetch(url).await?;
    let content: SerdeMediaTrackerProgress = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn storage_abort_current_task() -> Result<()> {
    let url = format!("{BASE_URL}storage/abort-current-task");
    let request = Request::new(url).method(Method::Post);
    let response = request.fetch().await?.check_status()?;
    let status_code = response.check_status()?.status().code;
    debug_assert_eq!(StatusCode::ACCEPTED, status_code);
    Ok(())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub(crate) async fn search_tracks(
    collection_uid: CollectionUid,
    params: SearchParams,
    pagination: impl Into<SerdePagination>,
) -> Result<Vec<Track>> {
    let (query_params, search_params) =
        aoide_core_api_json::track::search::client_request_params(params, pagination);
    let query_params_urlencoded = serde_urlencoded::to_string(query_params)
        .map_err(Into::into)
        .map_err(Error::DataShape)?;
    let url = format!("{BASE_URL}c/{collection_uid}/t/search?{query_params_urlencoded}");
    let request = Request::new(url)
        .method(Method::Post)
        .json(&search_params)?;
    let response = request.fetch().await?;
    let content: Vec<SerdeTrack> = response.check_status()?.json().await?;
    content
        .into_iter()
        .map(|item| item.try_into().map_err(Error::DataShape))
        .collect()
}

// ------ ------
//     Error
// ------ ------

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("A network problem has occurred")]
    Network,
    #[error("The form of the data to be processed is insufficient")]
    DataShape(anyhow::Error),
    #[error("Something went wrong in the browser: {0:?}")]
    Browser(Option<String>),
    #[error("The communication with the server failed ({code}): {msg}")]
    ServerCommunication { msg: String, code: u16 },
}

impl From<FetchError> for Error {
    fn from(e: FetchError) -> Self {
        use FetchError as E;
        match e {
            // TODO: Fix after https://github.com/seed-rs/seed/issues/673 has been resolved
            E::JsonError(err) => {
                Error::DataShape(anyhow::anyhow!("TODO Handle JSON error: {err:?}"))
            }
            E::NetworkError(_) => Error::Network,
            E::DomException(exception) => Error::Browser(exception.as_string()),
            E::PromiseError(js_value) | E::RequestError(js_value) => {
                Error::Browser(js_value.as_string())
            }
            E::StatusError(status) => Error::ServerCommunication {
                code: status.code,
                msg: status.text,
            },
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
