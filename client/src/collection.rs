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

use aoide_core::{collection::Entity as CollectionEntity, entity::EntityUid};
use reqwest::{Client, Url};

use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct RemoteState {
    available: RemoteData<Vec<CollectionEntity>>,
}

impl RemoteState {
    pub const fn available(&self) -> &RemoteData<Vec<CollectionEntity>> {
        &self.available
    }

    fn count_available_by_uid(&self, uid: &EntityUid) -> Option<usize> {
        self.available
            .get()
            .map(|v| v.iter().filter(|x| &x.hdr.uid == uid).count())
    }

    pub fn find_available_by_uid(&self, uid: &EntityUid) -> Option<&CollectionEntity> {
        debug_assert!(self.count_available_by_uid(uid).unwrap_or_default() <= 1);
        self.available
            .get()
            .and_then(|v| v.iter().find(|x| &x.hdr.uid == uid))
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    remote: RemoteState,
    active_uid: Option<EntityUid>,
}

impl State {
    pub const fn remote(&self) -> &RemoteState {
        &self.remote
    }

    pub const fn active_uid(&self) -> Option<&EntityUid> {
        self.active_uid.as_ref()
    }

    pub fn active(&self) -> Option<&CollectionEntity> {
        if let (Some(available), Some(active_uid)) = (self.remote.available.get(), &self.active_uid)
        {
            available.iter().find(|x| &x.hdr.uid == active_uid)
        } else {
            None
        }
    }

    fn set_available(&mut self, new_available: Vec<CollectionEntity>) {
        self.remote.available = RemoteData::ready(new_available);
        let active_uid = self.active_uid.take();
        self.set_active_uid(active_uid);
    }

    fn set_active_uid(&mut self, new_active_uid: impl Into<Option<EntityUid>>) {
        self.active_uid = if let (Some(available), Some(new_active_uid)) =
            (self.remote.available.get(), new_active_uid.into())
        {
            if available.iter().any(|x| x.hdr.uid == new_active_uid) {
                Some(new_active_uid)
            } else {
                None
            }
        } else {
            None
        };
    }

    fn reset_active_uid(&mut self) {
        self.set_active_uid(None)
    }
}

#[derive(Debug)]
pub enum Action {
    FetchAvailable,
    PropagateError(anyhow::Error),
}

#[derive(Debug)]
pub enum Event {
    FetchAvailableRequested,
    AvailableFetched(anyhow::Result<Vec<CollectionEntity>>),
    ActiveUidSelected(EntityUid),
    ActiveUidReset,
    ErrorOccurred(anyhow::Error),
}

pub fn apply_event(state: &mut State, event: Event) -> (AppliedEvent, Option<Action>) {
    match event {
        Event::FetchAvailableRequested => (
            AppliedEvent::Accepted {
                state_changed: false,
            },
            Some(Action::FetchAvailable),
        ),
        Event::AvailableFetched(res) => match res {
            Ok(new_available) => {
                state.set_available(new_available);
                (
                    AppliedEvent::Accepted {
                        state_changed: true,
                    },
                    None,
                )
            }
            Err(err) => (
                AppliedEvent::Accepted {
                    state_changed: false,
                },
                Some(Action::PropagateError(err)),
            ),
        },
        Event::ActiveUidSelected(new_active_uid) => {
            state.set_active_uid(new_active_uid);
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                None,
            )
        }
        Event::ActiveUidReset => {
            state.reset_active_uid();
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                None,
            )
        }
        Event::ErrorOccurred(error) => (
            AppliedEvent::Accepted {
                state_changed: false,
            },
            Some(Action::PropagateError(error)),
        ),
    }
}

pub async fn dispatch_action<E: From<Event> + fmt::Debug>(
    shared_env: Arc<Environment>,
    event_tx: EventSender<E>,
    action: Action,
) {
    match action {
        Action::FetchAvailable => {
            let res = fetch_available_collections(&shared_env.client, &shared_env.api_url).await;
            crate::emit_event(&event_tx, E::from(Event::AvailableFetched(res)));
        }
        Action::PropagateError(error) => {
            crate::emit_event(&event_tx, E::from(Event::ErrorOccurred(error)));
        }
    }
}

pub async fn fetch_available_collections(
    client: &Client,
    api_url: &Url,
) -> anyhow::Result<Vec<CollectionEntity>> {
    let url = api_url.join("c")?;
    let response =
        client.get(url).send().await.map_err(|err| {
            anyhow::Error::from(err).context("Failed to fetch available collections")
        })?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch available collections: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload when fetching available collections")
    })?;
    let available_collections: Vec<_> = serde_json::from_slice::<
        Vec<aoide_core_serde::collection::Entity>,
    >(&bytes)
    .map(|collections| {
        collections
            .into_iter()
            .map(CollectionEntity::from)
            .collect()
    })
    .map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to deserialize response payload when fetching available collections")
    })?;
    log::debug!(
        "Loaded {} available collection(s)",
        available_collections.len()
    );
    Ok(available_collections)
}
