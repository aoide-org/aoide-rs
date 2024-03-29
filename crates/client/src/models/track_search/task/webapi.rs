// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::CollectionUid;
use aoide_core_api_json::track::search::client_request_params;

use super::{
    super::{Effect, FetchResultPage, FetchResultPageRequest, FetchResultPageResponse},
    Task,
};
use crate::webapi::{receive_response_body, ClientEnvironment};

impl Task {
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task {self:?}");
        match self {
            Self::FetchResultPage(FetchResultPage {
                collection_uid,
                request,
            }) => {
                let response = fetch_result_page(env, &collection_uid, request).await;
                Effect::FetchResultPageFinished(response)
            }
        }
    }
}

async fn fetch_result_page<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    request: FetchResultPageRequest,
) -> anyhow::Result<FetchResultPageResponse> {
    let FetchResultPageRequest {
        params,
        encode_gigtags,
        pagination,
    } = request;
    let (query_params, search_params) =
        client_request_params(params, encode_gigtags, pagination.clone());
    let query_params_urlencoded = serde_urlencoded::to_string(query_params)?;
    let request_url = env.join_api_url(&format!(
        "c/{collection_uid}/t/search?{query_params_urlencoded}",
    ))?;
    let request_body = serde_json::to_vec(&search_params)?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let (entities, errors): (Vec<_>, _) =
        serde_json::from_slice::<Vec<aoide_core_json::track::Entity>>(&response_body)?
            .into_iter()
            .map(TryFrom::try_from)
            .partition(Result::is_ok);
    if let Some(err) = errors.into_iter().map(Result::unwrap_err).next() {
        return Err(err);
    }
    let entities: Vec<_> = entities.into_iter().map(Result::unwrap).collect();
    log::debug!(
        "Received {num_entities} entities with pagination {pagination:?}",
        num_entities = entities.len()
    );
    Ok(FetchResultPageResponse {
        entities,
        pagination,
    })
}
