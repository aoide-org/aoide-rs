// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aoide_core::entity::EntityUid;
use aoide_core_api_json::track::search::client_request_params;

use crate::web::{receive_response_body, ClientEnvironment};

use super::{Effect, FetchResultPageRequest, FetchResultPageResponse};

#[derive(Debug)]
pub enum Task {
    FetchResultPage {
        collection_uid: EntityUid,
        request: FetchResultPageRequest,
    },
}

impl Task {
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::FetchResultPage {
                collection_uid,
                request,
            } => {
                let response = fetch_result_page(env, &collection_uid, request).await;
                Effect::FetchResultPageFinished(response)
            }
        }
    }
}

async fn fetch_result_page<E: ClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    request: FetchResultPageRequest,
) -> anyhow::Result<FetchResultPageResponse> {
    let FetchResultPageRequest {
        search_params,
        pagination,
    } = request;
    let (query_params, search_params) = client_request_params(search_params, pagination.clone());
    let request_url = env.join_api_url(&format!(
        "c/{}/t/search?{}",
        collection_uid,
        serde_urlencoded::to_string(query_params)?
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
        "Received {} entities with pagination {:?}",
        entities.len(),
        pagination
    );
    Ok(FetchResultPageResponse {
        entities,
        pagination,
    })
}
