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

use crate::{prelude::*, receive_response_body};

use aoide_core::{
    entity::EntityUid,
    usecases::media::tracker::{
        import::Outcome as ImportOutcome, scan::Outcome as ScanOutcome,
        untrack::Outcome as UntrackOutcome, Progress, Status,
    },
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
    last_scan_outcome: RemoteData<ScanOutcome>,
    last_import_outcome: RemoteData<ImportOutcome>,
    last_untrack_outcome: RemoteData<UntrackOutcome>,
}

impl RemoteState {
    pub fn status(&self) -> &RemoteData<Status> {
        &self.status
    }

    pub fn progress(&self) -> &RemoteData<Progress> {
        &self.progress
    }

    pub fn last_scan_outcome(&self) -> &RemoteData<ScanOutcome> {
        &self.last_scan_outcome
    }

    pub fn last_import_outcome(&self) -> &RemoteData<ImportOutcome> {
        &self.last_import_outcome
    }

    pub fn last_untrack_outcome(&self) -> &RemoteData<UntrackOutcome> {
        &self.last_untrack_outcome
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
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    FetchProgress,
    StartScan {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    StartImport {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    Abort,
    Untrack {
        collection_uid: EntityUid,
        root_url: Url,
    },
    PropagateError(anyhow::Error),
}

#[derive(Debug)]
pub enum Event {
    Intent(Intent),
    Effect(Effect),
}

#[derive(Debug)]
pub enum Intent {
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    FetchProgress,
    StartScan {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    StartImport {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    Abort,
    Untrack {
        collection_uid: EntityUid,
        root_url: Url,
    },
}

impl From<Intent> for Event {
    fn from(intent: Intent) -> Self {
        Self::Intent(intent)
    }
}

pub fn fetch_status(collection_uid: EntityUid, root_url: Option<Url>) -> Event {
    Intent::FetchStatus {
        collection_uid,
        root_url,
    }
    .into()
}

pub fn fetch_progress() -> Event {
    Intent::FetchProgress.into()
}

pub fn start_scan(collection_uid: EntityUid, root_url: Option<Url>) -> Event {
    Intent::StartScan {
        collection_uid,
        root_url,
    }
    .into()
}

pub fn start_import(collection_uid: EntityUid, root_url: Option<Url>) -> Event {
    Intent::StartImport {
        collection_uid,
        root_url,
    }
    .into()
}

pub fn abort() -> Event {
    Intent::Abort.into()
}

pub fn untrack(collection_uid: EntityUid, root_url: Url) -> Event {
    Intent::Untrack {
        collection_uid,
        root_url,
    }
    .into()
}

#[derive(Debug)]
pub enum Effect {
    ProgressFetched(anyhow::Result<Progress>),
    Aborted(anyhow::Result<()>),
    StatusFetched(anyhow::Result<Status>),
    ScanFinished(anyhow::Result<ScanOutcome>),
    ImportFinished(anyhow::Result<ImportOutcome>),
    Untracked(anyhow::Result<UntrackOutcome>),
    ErrorOccurred(anyhow::Error),
}

impl From<Effect> for Event {
    fn from(effect: Effect) -> Self {
        Self::Effect(effect)
    }
}

pub fn apply_event(state: &mut State, event: Event) -> (AppliedEvent, Option<Action>) {
    match event {
        Event::Intent(intent) => match intent {
            Intent::FetchProgress => (
                AppliedEvent::Accepted {
                    state_changed: false,
                },
                Some(Action::FetchProgress),
            ),
            Intent::Abort => (
                AppliedEvent::Accepted {
                    state_changed: false,
                },
                Some(Action::Abort),
            ),
            Intent::FetchStatus {
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
            Intent::StartScan {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start scan while not idle");
                    return (AppliedEvent::Dropped, None);
                }
                state.control = ControlState::Busy;
                state.remote.progress.reset();
                state.remote.status.set_pending();
                state.remote.last_scan_outcome.set_pending();
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
            Intent::StartImport {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start import while not idle");
                    return (AppliedEvent::Dropped, None);
                }
                state.control = ControlState::Busy;
                state.remote.progress.reset();
                state.remote.status.set_pending();
                state.remote.last_import_outcome.set_pending();
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
            Intent::Untrack {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot untrack while not idle");
                    return (AppliedEvent::Dropped, None);
                }
                state.control = ControlState::Busy;
                state.remote.progress.reset();
                state.remote.status.set_pending();
                state.remote.last_untrack_outcome.set_pending();
                (
                    AppliedEvent::Accepted {
                        state_changed: true,
                    },
                    Some(Action::Untrack {
                        collection_uid,
                        root_url,
                    }),
                )
            }
        },
        Event::Effect(effect) => match effect {
            Effect::ProgressFetched(res) => match res {
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
            Effect::Aborted(res) => {
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
            Effect::StatusFetched(res) => {
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
            Effect::ScanFinished(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_scan_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_scan_outcome = RemoteData::ready(outcome);
                        Action::FetchProgress
                    }
                    Err(err) => {
                        state.remote.last_scan_outcome.reset();
                        Action::PropagateError(err)
                    }
                };
                (
                    AppliedEvent::Accepted {
                        state_changed: true,
                    },
                    Some(next_action),
                )
            }
            Effect::ImportFinished(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_import_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_import_outcome = RemoteData::ready(outcome);
                        Action::FetchProgress
                    }
                    Err(err) => {
                        state.remote.last_import_outcome.reset();
                        Action::PropagateError(err)
                    }
                };
                (
                    AppliedEvent::Accepted {
                        state_changed: true,
                    },
                    Some(next_action),
                )
            }
            Effect::Untracked(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_untrack_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_untrack_outcome = RemoteData::ready(outcome);
                        Action::FetchProgress
                    }
                    Err(err) => {
                        state.remote.last_untrack_outcome.reset();
                        Action::PropagateError(err)
                    }
                };
                (
                    AppliedEvent::Accepted {
                        state_changed: true,
                    },
                    Some(next_action),
                )
            }
            Effect::ErrorOccurred(error) => (
                AppliedEvent::Accepted {
                    state_changed: false,
                },
                Some(Action::PropagateError(error)),
            ),
        },
    }
}

pub async fn dispatch_action<E: From<Event> + fmt::Debug>(
    shared_env: Arc<Environment>,
    event_tx: EventSender<E>,
    action: Action,
) {
    match action {
        Action::FetchStatus {
            collection_uid,
            root_url,
        } => {
            let res = on_fetch_status(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                root_url.as_ref(),
            )
            .await;
            crate::emit_event(&event_tx, E::from(Effect::StatusFetched(res).into()));
        }
        Action::FetchProgress => {
            let res = on_fetch_progress(&shared_env.client, &shared_env.api_url).await;
            crate::emit_event(&event_tx, E::from(Effect::ProgressFetched(res).into()));
        }
        Action::StartScan {
            collection_uid,
            root_url,
        } => {
            let res = on_start_scan(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                root_url.as_ref(),
            )
            .await;
            crate::emit_event(&event_tx, E::from(Effect::ScanFinished(res).into()));
        }
        Action::StartImport {
            collection_uid,
            root_url,
        } => {
            let res = on_start_import(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                root_url.as_ref(),
            )
            .await;
            crate::emit_event(&event_tx, E::from(Effect::ImportFinished(res).into()));
        }
        Action::Abort => {
            let res = on_abort(&shared_env.client, &shared_env.api_url).await;
            crate::emit_event(&event_tx, E::from(Effect::Aborted(res).into()));
        }
        Action::Untrack {
            collection_uid,
            root_url,
        } => {
            let res = on_untrack(
                &shared_env.client,
                &shared_env.api_url,
                &collection_uid,
                &root_url,
            )
            .await;
            crate::emit_event(&event_tx, E::from(Effect::Untracked(res).into()));
        }
        Action::PropagateError(error) => {
            crate::emit_event(&event_tx, E::from(Effect::ErrorOccurred(error).into()));
        }
    }
}

async fn on_fetch_status(
    client: &Client,
    api_url: &Url,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<Status> {
    let request_url = api_url.join(&format!("c/{}/media-tracker/query-status", collection_uid))?;
    let request_body = serde_json::to_vec(&root_url.map(|root_url| {
        serde_json::json!({
            "rootUrl": root_url.to_string(),
        })
    }))?;
    let request = client.post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let status = serde_json::from_slice::<aoide_core_serde::usecases::media::tracker::Status>(
        &response_body,
    )
    .map(Into::into)?;
    log::debug!("Received status: {:?}", status);
    Ok(status)
}

async fn on_fetch_progress(client: &Client, root_url: &Url) -> anyhow::Result<Progress> {
    let request_url = root_url.join("media-tracker/progress")?;
    let request = client.get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let progress = serde_json::from_slice::<aoide_core_serde::usecases::media::tracker::Progress>(
        &response_body,
    )
    .map(Into::into)?;
    log::debug!("Received progress: {:?}", progress);
    Ok(progress)
}

async fn on_start_scan(
    client: &Client,
    api_url: &Url,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<ScanOutcome> {
    let request_url = api_url.join(&format!("c/{}/media-tracker/scan", collection_uid))?;
    let request_body = serde_json::to_vec(&root_url.map(|root_url| {
        serde_json::json!({
            "rootUrl": root_url.to_string(),
        })
    }))?;
    let request = client.post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome =
        serde_json::from_slice::<aoide_core_serde::usecases::media::tracker::scan::Outcome>(
            &response_body,
        )
        .map(Into::into)?;
    log::debug!("Scan finished: {:?}", outcome);
    Ok(outcome)
}

async fn on_start_import(
    client: &Client,
    api_url: &Url,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<ImportOutcome> {
    let request_url = api_url.join(&format!("c/{}/media-tracker/import", collection_uid))?;
    let request_body = serde_json::to_vec(&root_url.map(|root_url| {
        serde_json::json!({
            "rootUrl": root_url.to_string(),
        })
    }))?;
    let request = client.post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_serde::usecases::media::tracker::import::Outcome,
    >(&response_body)
    .map(Into::into)?;
    log::debug!("Import finished: {:?}", outcome);
    Ok(outcome)
}

pub async fn on_abort(client: &Client, root_url: &Url) -> anyhow::Result<()> {
    let request_url = root_url.join("media-tracker/abort")?;
    let request = client.post(request_url);
    let response = request.send().await?;
    let _ = receive_response_body(response).await?;
    Ok(())
}

async fn on_untrack(
    client: &Client,
    api_url: &Url,
    collection_uid: &EntityUid,
    root_url: &Url,
) -> anyhow::Result<UntrackOutcome> {
    let request_url = api_url.join(&format!("c/{}/media-tracker/untrack", collection_uid))?;
    let request_body = serde_json::to_vec(&serde_json::json!({
        "rootUrl": root_url.to_string(),
    }))?;
    let request = client.post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_serde::usecases::media::tracker::untrack::Outcome,
    >(&response_body)
    .map(Into::into)?;
    log::debug!("Untrack finished: {:?}", outcome);
    Ok(outcome)
}
