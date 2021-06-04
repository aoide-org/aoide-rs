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

use aoide_core::{
    collection::{Collection, Entity as CollectionEntity},
    entity::EntityUid,
};
use reqwest::{Client, Url};

use crate::{prelude::*, receive_response_body};

#[derive(Debug, Clone, Default)]
pub struct RemoteState {
    available_collections: RemoteData<Vec<CollectionEntity>>,
}

impl RemoteState {
    pub const fn available_collections(&self) -> &RemoteData<Vec<CollectionEntity>> {
        &self.available_collections
    }

    fn count_available_collections_by_uid(&self, uid: &EntityUid) -> Option<usize> {
        self.available_collections
            .get()
            .map(|v| v.iter().filter(|x| &x.hdr.uid == uid).count())
    }

    pub fn find_available_collections_by_uid(&self, uid: &EntityUid) -> Option<&CollectionEntity> {
        debug_assert!(
            self.count_available_collections_by_uid(uid)
                .unwrap_or_default()
                <= 1
        );
        self.available_collections
            .get()
            .and_then(|v| v.iter().find(|x| &x.hdr.uid == uid))
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    remote: RemoteState,
    active_collection_uid: Option<EntityUid>,
}

impl State {
    pub const fn remote(&self) -> &RemoteState {
        &self.remote
    }

    pub const fn active_collection_uid(&self) -> Option<&EntityUid> {
        self.active_collection_uid.as_ref()
    }

    pub fn active_collection(&self) -> Option<&CollectionEntity> {
        if let (Some(available), Some(active_collection_uid)) = (
            self.remote.available_collections.get(),
            &self.active_collection_uid,
        ) {
            available
                .iter()
                .find(|x| &x.hdr.uid == active_collection_uid)
        } else {
            None
        }
    }

    fn set_available_collections(&mut self, new_available_collections: Vec<CollectionEntity>) {
        self.remote.available_collections = RemoteData::ready(new_available_collections);
        let active_uid = self.active_collection_uid.take();
        self.set_active_collection_uid(active_uid);
    }

    fn set_active_collection_uid(&mut self, new_active_uid: impl Into<Option<EntityUid>>) {
        self.active_collection_uid = if let (Some(available), Some(new_active_uid)) = (
            self.remote.available_collections.get(),
            new_active_uid.into(),
        ) {
            if available.iter().any(|x| x.hdr.uid == new_active_uid) {
                Some(new_active_uid)
            } else {
                None
            }
        } else {
            None
        };
    }
}

#[derive(Debug)]
pub enum Action {
    ApplyEffect(Effect),
    DispatchTask(Task),
}

impl From<Effect> for Action {
    fn from(effect: Effect) -> Self {
        Self::ApplyEffect(effect)
    }
}

impl From<Task> for Action {
    fn from(task: Task) -> Self {
        Self::DispatchTask(task)
    }
}

#[derive(Debug)]
pub enum Task {
    CreateNewCollection(Collection),
    FetchAvailableCollections,
}

#[derive(Debug)]
pub enum Intent {
    CreateNewCollection(Collection),
    FetchAvailableCollections,
    ActivateCollection(Option<EntityUid>),
}

pub fn create_new_collection(arg: Collection) -> Intent {
    Intent::CreateNewCollection(arg)
}

pub fn fetch_available_collections() -> Intent {
    Intent::FetchAvailableCollections
}

pub fn activate_collection(arg: Option<EntityUid>) -> Intent {
    Intent::ActivateCollection(arg)
}

#[derive(Debug)]
pub enum Effect {
    NewCollectionCreated(anyhow::Result<CollectionEntity>),
    AvailableCollectionsFetched(anyhow::Result<Vec<CollectionEntity>>),
    ErrorOccurred(anyhow::Error),
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::CreateNewCollection(new_collection) => (
                StateMutation::Unchanged,
                Some(Task::CreateNewCollection(new_collection).into()),
            ),
            Self::FetchAvailableCollections => {
                state.remote.available_collections.set_pending();
                (
                    StateMutation::MaybeChanged,
                    Some(Task::FetchAvailableCollections.into()),
                )
            }
            Self::ActivateCollection(new_active_collection_uid) => {
                state.set_active_collection_uid(new_active_collection_uid);
                (StateMutation::MaybeChanged, None)
            }
        }
    }
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying event {:?} on {:?}", self, state);
        match self {
            Self::NewCollectionCreated(res) => match res {
                Ok(_) => (StateMutation::Unchanged, None),
                Err(err) => (
                    StateMutation::Unchanged,
                    Some(Self::ErrorOccurred(err).into()),
                ),
            },
            Self::AvailableCollectionsFetched(res) => match res {
                Ok(new_available_collections) => {
                    state.set_available_collections(new_available_collections);
                    (StateMutation::MaybeChanged, None)
                }
                Err(err) => (
                    StateMutation::Unchanged,
                    Some(Self::ErrorOccurred(err).into()),
                ),
            },
            Self::ErrorOccurred(error) => (
                StateMutation::Unchanged,
                Some(Self::ErrorOccurred(error).into()),
            ),
        }
    }
}

impl Task {
    pub async fn execute_with(self, env: &Environment) -> Effect {
        log::trace!("Executing task: {:?}", self);
        match self {
            Self::CreateNewCollection(new_collection) => {
                let res = on_create_new_collection(&env.client, &env.api_url, new_collection).await;
                Effect::NewCollectionCreated(res)
            }
            Self::FetchAvailableCollections => {
                let res = on_fetch_available_collections(&env.client, &env.api_url).await;
                Effect::AvailableCollectionsFetched(res)
            }
        }
    }
}

async fn on_create_new_collection(
    client: &Client,
    api_url: &Url,
    new_collection: Collection,
) -> anyhow::Result<CollectionEntity> {
    let url = api_url.join("c")?;
    let body = serde_json::to_vec(&aoide_core_serde::collection::Collection::from(
        new_collection,
    ))?;
    let request = client.post(url).body(body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let entity = serde_json::from_slice::<aoide_core_serde::collection::Entity>(&response_body)
        .map(Into::into)?;
    log::debug!("Created new collection entity: {:?}", entity);
    Ok(entity)
}

async fn on_fetch_available_collections(
    client: &Client,
    api_url: &Url,
) -> anyhow::Result<Vec<CollectionEntity>> {
    let request_url = api_url.join("c")?;
    let request = client.get(request_url);
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
