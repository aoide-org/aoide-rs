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

use crate::prelude::*;

use aoide_core::{
    entity::EntityUid,
    usecases::media::tracker::{Progress, Status},
};

use reqwest::{Client, Url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    Idle,
    Busy,
}

impl Default for ControlState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Default)]
pub struct RemoteState {
    status: RemoteData<Status>,
    progress: RemoteData<Progress>,
}

impl RemoteState {
    pub fn status(&self) -> &RemoteData<Status> {
        &self.status
    }

    pub fn progress(&self) -> &RemoteData<Progress> {
        &self.progress
    }
}

#[derive(Debug, Default)]
pub struct State {
    control: ControlState,
    remote: RemoteState,
}

impl State {
    pub fn control(&self) -> ControlState {
        self.control
    }

    pub fn remote(&self) -> &RemoteState {
        &self.remote
    }

    pub fn is_idle(&self) -> bool {
        self.control == ControlState::Idle
            && self.remote.progress.get_ready() == Some(&Progress::Idle)
    }
}

#[derive(Debug)]
pub enum Action {
    FetchProgress,
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    StartScan {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    StartImport {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    Abort,
    PropagateError(anyhow::Error),
}

#[derive(Debug)]
pub enum Event {
    FetchProgressRequested,
    ProgressFetched(anyhow::Result<Progress>),
    AbortRequested,
    Aborted(anyhow::Result<()>),
    FetchStatusRequested {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    StatusFetched(anyhow::Result<Status>),
    StartScanRequested {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    ScanFinished(anyhow::Result<()>),
    StartImportRequested {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    ImportFinished(anyhow::Result<()>),
    ErrorOccurred(anyhow::Error),
}

pub fn apply_event(state: &mut State, event: Event) -> (AppliedEvent, Option<Action>) {
    match event {
        Event::FetchProgressRequested => (
            AppliedEvent::Accepted {
                state_changed: false,
            },
            Some(Action::FetchProgress),
        ),
        Event::ProgressFetched(res) => match res {
            Ok(new_progress) => {
                let new_progress = RemoteData::ready(new_progress);
                let progress_changed = state.remote.progress != new_progress;
                if progress_changed {
                    state.remote.progress = new_progress;
                }
                (
                    AppliedEvent::Accepted {
                        state_changed: progress_changed,
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
        Event::AbortRequested => (
            AppliedEvent::Accepted {
                state_changed: false,
            },
            Some(Action::Abort),
        ),
        Event::Aborted(res) => {
            let next_action = match res {
                Ok(()) => Action::FetchProgress,
                Err(err) => Action::PropagateError(err),
            };
            (
                AppliedEvent::Accepted {
                    state_changed: false,
                },
                Some(next_action),
            )
        }
        Event::FetchStatusRequested {
            collection_uid,
            root_url,
        } => {
            if !state.is_idle() {
                log::warn!("Cannot fetch status while not idle");
                return (AppliedEvent::Dropped, None);
            }
            state.control = ControlState::Busy;
            state.remote.status.reset();
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                Some(Action::FetchStatus {
                    collection_uid,
                    root_url,
                }),
            )
        }
        Event::StatusFetched(res) => {
            debug_assert_eq!(state.control, ControlState::Busy);
            state.control = ControlState::Idle;
            let next_action = match res {
                Ok(new_status) => {
                    state.remote.status = RemoteData::ready(new_status);
                    None
                }
                Err(err) => Some(Action::PropagateError(err)),
            };
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                next_action,
            )
        }
        Event::StartScanRequested {
            collection_uid,
            root_url,
        } => {
            if !state.is_idle() {
                log::warn!("Cannot start scan while not idle");
                return (AppliedEvent::Dropped, None);
            }
            state.control = ControlState::Busy;
            state.remote.progress.reset();
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                Some(Action::StartScan {
                    collection_uid,
                    root_url,
                }),
            )
        }
        Event::ScanFinished(res) => {
            debug_assert_eq!(state.control, ControlState::Busy);
            state.control = ControlState::Idle;
            // Invalidate both status and progress to enforce refetching
            state.remote.status.reset();
            state.remote.progress.reset();
            let next_action = match res {
                Ok(()) => Action::FetchProgress,
                Err(err) => Action::PropagateError(err),
            };
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                Some(next_action),
            )
        }
        Event::StartImportRequested {
            collection_uid,
            root_url,
        } => {
            if !state.is_idle() {
                log::warn!("Cannot start import while not idle");
                return (AppliedEvent::Dropped, None);
            }
            state.control = ControlState::Busy;
            state.remote.progress.reset();
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                Some(Action::StartImport {
                    collection_uid,
                    root_url,
                }),
            )
        }
        Event::ImportFinished(res) => {
            debug_assert_eq!(state.control, ControlState::Busy);
            state.control = ControlState::Idle;
            // Invalidate both status and progress to enforce refetching
            state.remote.status.reset();
            state.remote.progress.reset();
            let next_action = match res {
                Ok(()) => Action::FetchProgress,
                Err(err) => Action::PropagateError(err),
            };
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                Some(next_action),
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
        Action::FetchProgress => {
            let res = fetch_progress(&shared_env.client, &shared_env.api_url).await;
            crate::emit_event(&event_tx, E::from(Event::ProgressFetched(res)));
        }
        Action::Abort => {
            let res = abort(&shared_env.client, &shared_env.api_url).await;
            crate::emit_event(&event_tx, E::from(Event::Aborted(res)));
        }
        Action::FetchStatus {
            collection_uid,
            root_url,
        } => {
            let res = fetch_status(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                root_url.as_ref(),
            )
            .await;
            crate::emit_event(&event_tx, E::from(Event::StatusFetched(res)));
        }
        Action::StartScan {
            collection_uid,
            root_url,
        } => {
            let res = run_scan_task(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                root_url.as_ref(),
            )
            .await;
            crate::emit_event(&event_tx, E::from(Event::ScanFinished(res)));
        }
        Action::StartImport {
            collection_uid,
            root_url,
        } => {
            let res = run_import_task(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                root_url.as_ref(),
            )
            .await;
            crate::emit_event(&event_tx, E::from(Event::ImportFinished(res)));
        }
        Action::PropagateError(error) => {
            crate::emit_event(&event_tx, E::from(Event::ErrorOccurred(error)));
        }
    }
}

pub async fn fetch_status(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<Status> {
    let url = base_url.join(&format!("c/{}/media-tracker/query-status", collection_uid))?;
    let body_json = root_url
        .map(|root_url| {
            serde_json::json!({
                "rootUrl": root_url.to_string(),
            })
        })
        .unwrap_or_default();
    let body = serde_json::to_vec(&body_json).map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to serialize request body when fetching media tracker status")
    })?;
    let response =
        client.post(url).body(body).send().await.map_err(|err| {
            anyhow::Error::from(err).context("Failed to query media tracker status")
        })?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch media tracker status: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload when fetching media tracker status")
    })?;
    let status = serde_json::from_slice::<aoide_core_serde::media::tracker::Status>(&bytes)
        .map(Into::into)
        .map_err(|err| {
            anyhow::Error::from(err).context(
                "Failed to deserialize response payload when fetching media tracker status",
            )
        })?;
    log::debug!("Received status: {:?}", status);
    Ok(status)
}

pub async fn run_scan_task(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<()> {
    let url = base_url.join(&format!("c/{}/media-tracker/scan", collection_uid))?;
    let body_json = root_url
        .map(|root_url| {
            serde_json::json!({
                "rootUrl": root_url.to_string(),
            })
        })
        .unwrap_or_default();
    let body = serde_json::to_vec(&body_json).map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to serialize request body when starting media tracker scan")
    })?;
    let response = client
        .post(url)
        .body(body)
        .send()
        .await
        .map_err(|err| anyhow::Error::from(err).context("media tracker scan failure"))?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Media tracker scan failed: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload of media tracker scan")
    })?;
    log::error!("TODO: Deserialize and return response payload: {:?}", bytes);
    Ok(())
}

pub async fn run_import_task(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<()> {
    let url = base_url.join(&format!("c/{}/media-tracker/import", collection_uid))?;
    let body_json = root_url
        .map(|root_url| {
            serde_json::json!({
                "rootUrl": root_url.to_string(),
            })
        })
        .unwrap_or_default();
    let body = serde_json::to_vec(&body_json).map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to serialize request body when starting media tracker import")
    })?;
    let response = client
        .post(url)
        .body(body)
        .send()
        .await
        .map_err(|err| anyhow::Error::from(err).context("media tracker import failed"))?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Media tracker import failed: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload of media tracker import")
    })?;
    log::error!("TODO: Deserialize and return response payload: {:?}", bytes);
    Ok(())
}

pub async fn fetch_progress(client: &Client, base_url: &Url) -> anyhow::Result<Progress> {
    let url = base_url.join("media-tracker/progress")?;
    let response = client.get(url).send().await.map_err(|err| {
        anyhow::Error::from(err).context("Failed to fetch media tracker progress")
    })?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to get media tracker progress: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload when fetching media tracker progress")
    })?;
    let progress =
        serde_json::from_slice::<aoide_core_serde::usecases::media::tracker::Progress>(&bytes)
            .map(Into::into)
            .map_err(|err| {
                anyhow::Error::from(err).context(
                    "Failed to deserialize response payload when fetching media tracker progress",
                )
            })?;
    log::debug!("Received progress: {:?}", progress);
    Ok(progress)
}

pub async fn abort(client: &Client, base_url: &Url) -> anyhow::Result<()> {
    let url = base_url.join("media-tracker/abort")?;
    let response = client
        .post(url)
        .send()
        .await
        .map_err(|err| anyhow::Error::from(err).context("Failed to abort media tracker"))?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to abort media tracker: response status = {}",
            response.status()
        );
    }
    Ok(())
}
