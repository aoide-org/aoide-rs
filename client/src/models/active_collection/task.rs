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

use std::convert::TryInto;

use aoide_core::collection::{Collection, Entity as CollectionEntity};

use crate::{receive_response_body, WebClientEnvironment};

use super::Effect;

#[derive(Debug)]
pub enum Task {
    CreateNewCollection(Collection),
    FetchAvailableCollections,
}

impl Task {
    pub async fn execute<E: WebClientEnvironment>(self, env: &E) -> Effect {
        log::trace!("Executing task: {:?}", self);
        match self {
            Self::CreateNewCollection(new_collection) => {
                let res = create_new_collection(env, new_collection).await;
                Effect::NewCollectionCreated(res)
            }
            Self::FetchAvailableCollections => {
                let res = fetch_available_collections(env).await;
                Effect::AvailableCollectionsFetched(res)
            }
        }
    }
}

pub async fn create_new_collection<E: WebClientEnvironment>(
    env: &E,
    new_collection: Collection,
) -> anyhow::Result<CollectionEntity> {
    let url = env.join_api_url("c")?;
    let body = serde_json::to_vec(&aoide_core_serde::collection::Collection::from(
        new_collection,
    ))?;
    let request = env.client().post(url).body(body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let entity = serde_json::from_slice::<aoide_core_serde::collection::Entity>(&response_body)
        .map_err(anyhow::Error::from)
        .and_then(TryInto::try_into)?;
    log::debug!("Created new collection entity: {:?}", entity);
    Ok(entity)
}

pub async fn fetch_available_collections<E: WebClientEnvironment>(
    env: &E,
) -> anyhow::Result<Vec<CollectionEntity>> {
    let request_url = env.join_api_url("c")?;
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let fetched_collections: Vec<_> =
        serde_json::from_slice::<Vec<aoide_core_serde::collection::Entity>>(&response_body)?;
    let mut available_collections = Vec::with_capacity(fetched_collections.len());
    for collection in fetched_collections {
        let collection = collection.try_into()?;
        available_collections.push(collection);
    }
    log::debug!(
        "Fetched {} available collection(s)",
        available_collections.len(),
    );
    Ok(available_collections)
}
