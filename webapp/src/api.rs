use crate::domain::*;
use seed::{prelude::*, *};
use thiserror::Error;

const BASE_URL: &str = "/api";

/// GET /c
pub async fn get_all_collections() -> Result<Collections> {
    let url = format!("{}/c", BASE_URL);
    let res = fetch(url).await?;
    let c: Collections = res.check_status()?.json().await?;
    Ok(c)
}

/// GET /c/{ID}
pub async fn get_collection(id: &str) -> Result<CollectionWithSummary> {
    let url = format!("{}/c/{}", BASE_URL, id);
    let res = fetch(url).await?;
    let c: CollectionWithSummary = res.check_status()?.json().await?;
    Ok(c)
}

// ------ ------
//     Error
// ------ ------

#[derive(Debug, Error)]
pub enum Error {
    #[error("A network problem has occurred")]
    Network,
    #[error("The form of the data to be processed is insufficient")]
    DataShape,
    #[error("Something went wrong in the browser: {0:?}")]
    Browser(Option<String>),
    #[error("The communication with the server failed ({code}): {msg}")]
    ServerCommunication { msg: String, code: u16 },
}

impl From<FetchError> for Error {
    fn from(e: FetchError) -> Self {
        use FetchError as E;
        match e {
            E::SerdeError(_) => Error::DataShape,
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
