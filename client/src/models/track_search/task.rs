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

use crate::{receive_response_body, Environment};

use super::{Effect, FetchResultPageRequest, FetchResultPageResponse};

use aoide_core::entity::EntityUid;

#[derive(Debug)]
pub enum Task {
    FetchResultPage {
        collection_uid: EntityUid,
        request: FetchResultPageRequest,
    },
}

impl Task {
    pub async fn execute_with(self, env: &Environment) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::FetchResultPage {
                collection_uid,
                request,
            } => {
                let response = fetch_result_page(env, &collection_uid, request).await;
                Effect::ResultPageFetched(response)
            }
        }
    }
}

async fn fetch_result_page(
    env: &Environment,
    collection_uid: &EntityUid,
    request: FetchResultPageRequest,
) -> anyhow::Result<FetchResultPageResponse> {
    let FetchResultPageRequest {
        search_params,
        resolve_url_from_path,
        pagination,
    } = request;
    let request_url = env.join_api_url(&format!(
        "c/{}/t/search?resolveUrlFromPath={}&offset={}&limit={}",
        collection_uid,
        if resolve_url_from_path {
            "true"
        } else {
            "false"
        },
        pagination.offset,
        pagination.limit
    ))?;
    let request_body = serde_json::to_vec(
        &aoide_core_serde::usecases::tracks::search::SearchParams::from(search_params),
    )?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let entities: Vec<_> =
        serde_json::from_slice::<Vec<aoide_core_serde::track::Entity>>(&response_body)?
            .into_iter()
            .map(Into::into)
            .collect();
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
