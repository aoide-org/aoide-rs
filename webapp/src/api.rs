use aoide_core_ext_serde::collection::{
    import_entity_with_summary, EntityWithSummary as CollectionEntityWithSummary,
};
use seed::{prelude::*, *};

use thiserror::Error;

use aoide_core::entity::EntityUid;

use crate::domain::*;

const BASE_URL: &str = "/api";

/// GET /c
pub async fn fetch_all_collections() -> Result<CollectionItems> {
    let url = format!("{}/c", BASE_URL);
    let response = fetch(url).await?;
    let content: Vec<aoide_core_serde::collection::Entity> =
        response.check_status()?.json().await?;
    let mut items = Vec::with_capacity(content.len());
    for item in content {
        let entity = item.try_into().map_err(Error::DataShape)?;
        items.push(CollectionItem::without_summary(entity));
    }
    Ok(items)
}

/// GET /c/{UID}
pub async fn fetch_collection_with_summary(uid: EntityUid) -> Result<CollectionItem> {
    let url = format!("{}/c/{}?summary=true", BASE_URL, &uid);
    let response = fetch(url).await?;
    let content: CollectionEntityWithSummary = response.check_status()?.json().await?;
    content.try_into()
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

impl TryFrom<CollectionEntityWithSummary> for CollectionItem {
    type Error = Error;

    fn try_from(from: CollectionEntityWithSummary) -> Result<Self> {
        import_entity_with_summary(from)
            .map(|(entity, summary)| Self { entity, summary })
            .map_err(Error::DataShape)
    }
}
