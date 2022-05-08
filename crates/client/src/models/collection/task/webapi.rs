// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::collection::Entity as CollectionEntity;

use crate::webapi::{receive_response_body, ClientEnvironment};

use super::{super::Effect, Task};

impl Task {
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Effect {
        log::trace!("Executing task: {:?}", self);
        match self {
            Self::FetchAllKinds { token } => {
                let result = fetch_all_kinds(env).await;
                Effect::FetchAllKindsFinished { token, result }
            }
            Self::FetchFilteredEntities {
                token,
                filter_by_kind,
            } => {
                let result = fetch_filtered_entities(env, filter_by_kind.as_deref()).await;
                Effect::FetchFilteredEntitiesFinished {
                    token,
                    filtered_by_kind: filter_by_kind,
                    result,
                }
            }
            Self::CreateEntity { new_collection } => {
                let result = create_entity(env, new_collection).await;
                Effect::CreateEntityFinished(result)
            }
            Self::UpdateEntity {
                entity_header,
                modified_collection,
            } => {
                let result = update_entity(env, &entity_header, modified_collection).await;
                Effect::UpdateEntityFinished(result)
            }
            Self::PurgeEntity { entity_uid } => {
                let result = purge_entity(env, &entity_uid).await.map(|()| entity_uid);
                Effect::PurgeEntityFinished(result)
            }
        }
    }
}

async fn fetch_all_kinds<E: ClientEnvironment>(env: &E) -> anyhow::Result<Vec<String>> {
    let request_url = env.join_api_url("c/kinds")?;
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let kinds = serde_json::from_slice::<Vec<String>>(&response_body)?;
    log::debug!("Fetched {} kind(s)", kinds.len(),);
    Ok(kinds)
}

async fn fetch_filtered_entities<E: ClientEnvironment>(
    env: &E,
    filter_by_kind: impl Into<Option<&str>>,
) -> anyhow::Result<Vec<CollectionEntity>> {
    let mut request_url = env.join_api_url("c")?;
    let query_params = filter_by_kind
        .into()
        .and_then(|kind| serde_urlencoded::to_string(&kind).ok())
        .map(|kind| format!("kind={}", kind));
    request_url.set_query(query_params.as_deref());
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let (entities, errors): (Vec<_>, _) =
        serde_json::from_slice::<Vec<aoide_core_json::collection::Entity>>(&response_body)?
            .into_iter()
            .map(TryFrom::try_from)
            .partition(Result::is_ok);
    if let Some(err) = errors.into_iter().map(Result::unwrap_err).next() {
        return Err(err);
    }
    let entities: Vec<_> = entities.into_iter().map(Result::unwrap).collect();
    log::debug!("Fetched {} filtered entities(s)", entities.len());
    Ok(entities)
}

async fn create_entity<E: ClientEnvironment>(
    env: &E,
    new_collection: impl Into<aoide_core_json::collection::Collection>,
) -> anyhow::Result<CollectionEntity> {
    let url = env.join_api_url("c")?;
    let body = serde_json::to_vec(&new_collection.into())?;
    let request = env.client().post(url).body(body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let entity = serde_json::from_slice::<aoide_core_json::collection::Entity>(&response_body)
        .map_err(anyhow::Error::from)
        .and_then(TryInto::try_into)?;
    log::debug!("Creating new collection entity succeeded: {:?}", entity);
    Ok(entity)
}

async fn update_entity<E: ClientEnvironment>(
    env: &E,
    entity_header: &aoide_core::collection::EntityHeader,
    modified_collection: impl Into<aoide_core_json::collection::Collection>,
) -> anyhow::Result<CollectionEntity> {
    let url = env.join_api_url(&format!(
        "c/{}?rev={}",
        entity_header.uid, entity_header.rev
    ))?;
    let body = serde_json::to_vec(&modified_collection.into())?;
    let request = env.client().put(url).body(body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let entity = serde_json::from_slice::<aoide_core_json::collection::Entity>(&response_body)
        .map_err(anyhow::Error::from)
        .and_then(TryInto::try_into)?;
    log::debug!(
        "Updating modified collection entity succeeded: {:?}",
        entity
    );
    Ok(entity)
}

async fn purge_entity<E: ClientEnvironment>(
    env: &E,
    entity_uid: &aoide_core::entity::EntityUid,
) -> anyhow::Result<()> {
    let url = env.join_api_url(&format!("c/{}", entity_uid))?;
    let request = env.client().delete(url);
    let _response = request.send().await?;
    log::debug!("Purging collection entity succeeded: {:?}", entity_uid);
    Ok(())
}
