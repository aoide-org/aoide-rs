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

use std::{fmt, sync::Arc};

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
    CreateNewCollection(Collection),
    FetchAvailableCollections,
    PropagateError(anyhow::Error),
}

#[derive(Debug)]
pub enum Event {
    Intent(Intent),
    Effect(Effect),
}

#[derive(Debug)]
pub enum Intent {
    CreateNewCollection(Collection),
    FetchAvailableCollections,
    ActivateCollection(Option<EntityUid>),
}

pub fn create_new_collection(arg: Collection) -> Event {
    Intent::CreateNewCollection(arg).into()
}

pub fn fetch_available_collections() -> Event {
    Intent::FetchAvailableCollections.into()
}

pub fn activate_collection(arg: Option<EntityUid>) -> Event {
    Intent::ActivateCollection(arg).into()
}

impl From<Intent> for Event {
    fn from(intent: Intent) -> Self {
        Self::Intent(intent)
    }
}

#[derive(Debug)]
pub enum Effect {
    NewCollectionCreated(anyhow::Result<CollectionEntity>),
    AvailableCollectionsFetched(anyhow::Result<Vec<CollectionEntity>>),
    ErrorOccurred(anyhow::Error),
}

impl From<Effect> for Event {
    fn from(effect: Effect) -> Self {
        Self::Effect(effect)
    }
}

pub fn apply_event(state: &mut State, event: Event) -> EventApplied<Action> {
    match event {
        Event::Intent(intent) => match intent {
            Intent::CreateNewCollection(new_collection) => EventApplied::Accepted {
                next_action: Some(Action::CreateNewCollection(new_collection)),
            },
            Intent::FetchAvailableCollections => {
                state.remote.available_collections.set_pending();
                EventApplied::Accepted {
                    next_action: Some(Action::FetchAvailableCollections),
                }
            }
            Intent::ActivateCollection(new_active_collection_uid) => {
                state.set_active_collection_uid(new_active_collection_uid);
                EventApplied::StateChanged { next_action: None }
            }
        },
        Event::Effect(effect) => match effect {
            Effect::NewCollectionCreated(res) => match res {
                Ok(_) => EventApplied::Accepted { next_action: None },
                Err(err) => EventApplied::Accepted {
                    next_action: Some(Action::PropagateError(err)),
                },
            },
            Effect::AvailableCollectionsFetched(res) => match res {
                Ok(new_available_collections) => {
                    state.set_available_collections(new_available_collections);
                    EventApplied::StateChanged { next_action: None }
                }
                Err(err) => EventApplied::Accepted {
                    next_action: Some(Action::PropagateError(err)),
                },
            },
            Effect::ErrorOccurred(error) => EventApplied::Accepted {
                next_action: Some(Action::PropagateError(error)),
            },
        },
    }
}

pub async fn dispatch_action<E: From<Event> + fmt::Debug>(
    shared_env: Arc<Environment>,
    event_tx: EventSender<E>,
    action: Action,
) {
    match action {
        Action::CreateNewCollection(new_collection) => {
            let res =
                on_create_new_collection(&shared_env.client, &shared_env.api_url, new_collection)
                    .await;
            send_event(&event_tx, E::from(Effect::NewCollectionCreated(res).into()));
        }
        Action::FetchAvailableCollections => {
            let res = on_fetch_available_collections(&shared_env.client, &shared_env.api_url).await;
            send_event(
                &event_tx,
                E::from(Effect::AvailableCollectionsFetched(res).into()),
            );
        }
        Action::PropagateError(error) => {
            send_event(&event_tx, E::from(Effect::ErrorOccurred(error).into()));
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
