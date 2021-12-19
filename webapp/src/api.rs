use hyper::StatusCode;
use thiserror::Error;

use seed::{prelude::*, *};

use aoide_core::{entity::EntityUid, track::Track};

use aoide_core_ext::{
    media::tracker::{
        import::Outcome as ImportMediaSourcesOutcome,
        purge_untracked_sources::Outcome as PurgeUntrackedMediaSourcesOutcome,
        scan::Outcome as ScanMediaSourcesOutcome, untrack::Outcome as UntrackMediaSourcesOutcome,
        Progress as MediaTrackerProgress, Status as QueryMediaTrackerStatusOutcome,
    },
    track::search::Params as SearchParams,
};

use aoide_core_serde::{
    collection::{Collection as SerdeCollection, Entity as SerdeCollectionEntity},
    entity::{Entity as SerdeEntity, EntityHeader as SerdeEntityHeader},
    track::Track as SerdeTrack,
};

use aoide_core_ext_serde::{
    collection::{import_entity_with_summary, EntityWithSummary as CollectionEntityWithSummary},
    media::tracker::{
        import::{
            Outcome as SerdeImportMediaSourcesOutcome, Params as SerdeImportMediaSourcesParams,
        },
        purge_untracked_sources::{
            Outcome as SerdePurgeUntrackedMediaSourcesOutcome,
            Params as SerdePurgeUntrackedMediaSourcesParams,
        },
        query_status::Params as SerdeQueryMediaTrackerStatusParams,
        scan::Outcome as SerdeScanMediaSourcesOutcome,
        untrack::{
            Outcome as SerdeUntrackMediaSourcesOutcome, Params as SerdeUntrackMediaSourcesParams,
        },
        FsTraversalParams as SerdeFsTraversalParams, Progress as SerdeMediaTrackerProgress,
        Status as SerdeQueryMediaTrackerStatusOutcome,
    },
    Pagination as SerdePagination,
};

use crate::domain::*;

const BASE_URL: &str = "/api";

impl TryFrom<CollectionEntityWithSummary> for CollectionItem {
    type Error = Error;

    fn try_from(from: CollectionEntityWithSummary) -> Result<Self> {
        import_entity_with_summary(from)
            .map(|(entity, summary)| Self { entity, summary })
            .map_err(Error::DataShape)
    }
}

pub async fn fetch_all_collections() -> Result<CollectionItems> {
    let url = format!("{}/c", BASE_URL);
    let response = fetch(url).await?;
    let content: Vec<SerdeCollectionEntity> = response.check_status()?.json().await?;
    let capacity = content.len();
    content
        .into_iter()
        .try_fold(Vec::with_capacity(capacity), |mut collected, item| {
            collected.push(CollectionItem::without_summary(
                item.try_into().map_err(Error::DataShape)?,
            ));
            Ok(collected)
        })
}

pub async fn fetch_collection_with_summary(uid: EntityUid) -> Result<CollectionItem> {
    let url = format!("{}/c/{}?summary=true", BASE_URL, &uid);
    let response = fetch(url).await?;
    let content: CollectionEntityWithSummary = response.check_status()?.json().await?;
    content.try_into()
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn create_collection(collection: impl Into<SerdeCollection>) -> Result<EntityHeader> {
    let url = format!("{}/c", BASE_URL);
    let request = Request::new(url)
        .method(Method::Post)
        .json(&collection.into())?;
    let response = request.fetch().await?;
    let content: SerdeEntityHeader = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn update_collection(
    entity_header: impl Into<SerdeEntityHeader>,
    collection: impl TryInto<SerdeCollection, Error = anyhow::Error>,
) -> Result<EntityHeader> {
    let url = format!("{}/c", BASE_URL);
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
pub async fn delete_collection(entity_header: impl Into<SerdeEntityHeader>) -> Result<()> {
    let url = format!("{}/c", BASE_URL);
    let request = Request::new(url)
        .method(Method::Delete)
        .json(&entity_header.into())?;
    let response = request.fetch().await?.check_status()?;
    let _status_code = response.check_status()?.status().code;
    debug_assert_eq!(StatusCode::NO_CONTENT, _status_code);
    Ok(())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn scan_media_sources(
    collection_uid: EntityUid,
    params: impl Into<SerdeFsTraversalParams>,
) -> Result<ScanMediaSourcesOutcome> {
    let url = format!("{}/c/{}/mt/scan", BASE_URL, collection_uid);
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeScanMediaSourcesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn import_media_sources(
    collection_uid: EntityUid,
    params: impl Into<SerdeImportMediaSourcesParams>,
) -> Result<ImportMediaSourcesOutcome> {
    let url = format!("{}/c/{}/mt/import", BASE_URL, collection_uid);
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeImportMediaSourcesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn untrack_media_sources(
    collection_uid: EntityUid,
    params: impl Into<SerdeUntrackMediaSourcesParams>,
) -> Result<UntrackMediaSourcesOutcome> {
    let url = format!("{}/c/{}/mt/untrack", BASE_URL, collection_uid);
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeUntrackMediaSourcesOutcome = response.check_status()?.json().await?;
    content
        .try_into()
        .map_err(anyhow::Error::from)
        .map_err(Error::DataShape)
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn purge_untracked_media_sources(
    collection_uid: EntityUid,
    params: impl Into<SerdePurgeUntrackedMediaSourcesParams>,
) -> Result<PurgeUntrackedMediaSourcesOutcome> {
    let url = format!(
        "{}/c/{}/mt/purge-untracked-sources",
        BASE_URL, collection_uid
    );
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
pub async fn query_media_tracker_status(
    collection_uid: EntityUid,
    params: impl Into<SerdeQueryMediaTrackerStatusParams>,
) -> Result<QueryMediaTrackerStatusOutcome> {
    let url = format!("{}/c/{}/mt/query-status", BASE_URL, collection_uid);
    let request = Request::new(url)
        .method(Method::Post)
        .json(&params.into())?;
    let response = request.fetch().await?;
    let content: SerdeQueryMediaTrackerStatusOutcome = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn get_media_tracker_progress() -> Result<MediaTrackerProgress> {
    let url = format!("{}/mt/progress", BASE_URL);
    let response = fetch(url).await?;
    let content: SerdeMediaTrackerProgress = response.check_status()?.json().await?;
    Ok(content.into())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn storage_abort_current_task() -> Result<()> {
    let url = format!("{}/storage/abort-current-task", BASE_URL);
    let request = Request::new(url).method(Method::Post);
    let response = request.fetch().await?.check_status()?;
    let _status_code = response.check_status()?.status().code;
    debug_assert_eq!(StatusCode::ACCEPTED, _status_code);
    Ok(())
}

#[allow(dead_code)] // TODO: Remove allow attribute after function is used
pub async fn search_tracks(
    collection_uid: EntityUid,
    params: SearchParams,
    pagination: impl Into<SerdePagination>,
) -> Result<Vec<Track>> {
    let (query_params, search_params) =
        aoide_core_ext_serde::track::search::client_request_params(params, pagination);
    let url = format!(
        "{}/c/{}/t/search?{}",
        BASE_URL,
        collection_uid,
        serde_urlencoded::to_string(query_params)
            .map_err(Into::into)
            .map_err(Error::DataShape)?,
    );
    let request = Request::new(url)
        .method(Method::Post)
        .json(&search_params)?;
    let response = request.fetch().await?;
    let content: Vec<SerdeTrack> = response.check_status()?.json().await?;
    let capacity = content.len();
    content
        .into_iter()
        .try_fold(Vec::with_capacity(capacity), |mut collected, item| {
            collected.push(item.try_into().map_err(Error::DataShape)?);
            Ok(collected)
        })
}

// ------ ------
//     Error
// ------ ------

#[derive(Debug, Error)]
pub enum Error {
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
            E::SerdeError(err) => Error::DataShape(err.into()),
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

pub type Result<T> = std::result::Result<T, Error>;
