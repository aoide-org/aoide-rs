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

use super::Effect;

use crate::{prelude::Environment, receive_response_body};

use aoide_core::collection::{Collection, Entity as CollectionEntity};

#[derive(Debug)]
pub enum Task {
    CreateNewCollection(Collection),
    FetchAvailableCollections,
}

impl Task {
    pub async fn execute_with(self, env: &Environment) -> Effect {
        log::trace!("Executing task: {:?}", self);
        match self {
            Self::CreateNewCollection(new_collection) => {
                let res = create_new_collection(&env, new_collection).await;
                Effect::NewCollectionCreated(res)
            }
            Self::FetchAvailableCollections => {
                let res = fetch_available_collections(&env).await;
                Effect::AvailableCollectionsFetched(res)
            }
        }
    }
}

pub async fn create_new_collection(
    env: &Environment,
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
        .map(Into::into)?;
    log::debug!("Created new collection entity: {:?}", entity);
    Ok(entity)
}

pub async fn fetch_available_collections(
    env: &Environment,
) -> anyhow::Result<Vec<CollectionEntity>> {
    let request_url = env.join_api_url("c")?;
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let available_collections: Vec<_> = serde_json::from_slice::<
        Vec<aoide_core_serde::collection::Entity>,
    >(&response_body)
    .map(|collections| {
        collections
            .into_iter()
            .map(CollectionEntity::from)
            .collect()
    })?;
    log::debug!(
        "Fetched {} available collection(s)",
        available_collections.len(),
    );
    Ok(available_collections)
}
