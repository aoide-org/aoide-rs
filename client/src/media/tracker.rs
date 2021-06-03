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
            && (self.remote.progress.get_ready() == Some(&Progress::Idle)
                || self.remote.progress.is_unknown())
    }
}

#[derive(Debug)]
pub enum NextAction {
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

pub fn fetch_status(collection_uid: EntityUid, root_url: Option<Url>) -> Intent {
    Intent::FetchStatus {
        collection_uid,
        root_url,
    }
}

pub fn fetch_progress() -> Intent {
    Intent::FetchProgress
}

pub fn start_scan(collection_uid: EntityUid, root_url: Option<Url>) -> Intent {
    Intent::StartScan {
        collection_uid,
        root_url,
    }
}

pub fn start_import(collection_uid: EntityUid, root_url: Option<Url>) -> Intent {
    Intent::StartImport {
        collection_uid,
        root_url,
    }
}

pub fn abort() -> Intent {
    Intent::Abort
}

pub fn untrack(collection_uid: EntityUid, root_url: Url) -> Intent {
    Intent::Untrack {
        collection_uid,
        root_url,
    }
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

pub fn apply_intent(state: &mut State, intent: Intent) -> (StateMutation, Option<NextAction>) {
    match intent {
        Intent::FetchProgress => (StateMutation::Unchanged, Some(NextAction::FetchProgress)),
        Intent::Abort => (StateMutation::Unchanged, Some(NextAction::Abort)),
        Intent::FetchStatus {
            collection_uid,
            root_url,
        } => {
            if !state.is_idle() {
                log::warn!("Cannot fetch status while not idle");
                return (StateMutation::Unchanged, None);
            }
            state.control = ControlState::Busy;
            state.remote.status.reset();
            (
                StateMutation::MaybeChanged,
                Some(NextAction::FetchStatus {
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
                return (StateMutation::Unchanged, None);
            }
            state.control = ControlState::Busy;
            state.remote.progress.reset();
            state.remote.status.set_pending();
            state.remote.last_scan_outcome.set_pending();
            (
                StateMutation::MaybeChanged,
                Some(NextAction::StartScan {
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
                return (StateMutation::Unchanged, None);
            }
            state.control = ControlState::Busy;
            state.remote.progress.reset();
            state.remote.status.set_pending();
            state.remote.last_import_outcome.set_pending();
            (
                StateMutation::MaybeChanged,
                Some(NextAction::StartImport {
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
                return (StateMutation::Unchanged, None);
            }
            state.control = ControlState::Busy;
            state.remote.progress.reset();
            state.remote.status.set_pending();
            state.remote.last_untrack_outcome.set_pending();
            (
                StateMutation::MaybeChanged,
                Some(NextAction::Untrack {
                    collection_uid,
                    root_url,
                }),
            )
        }
    }
}

pub fn apply_effect(state: &mut State, effect: Effect) -> (StateMutation, Option<NextAction>) {
    match effect {
        Effect::ProgressFetched(res) => match res {
            Ok(new_progress) => {
                let new_progress = RemoteData::ready(new_progress);
                if state.remote.progress != new_progress {
                    state.remote.progress = new_progress;
                    (StateMutation::MaybeChanged, None)
                } else {
                    (StateMutation::Unchanged, None)
                }
            }
            Err(err) => (
                StateMutation::Unchanged,
                Some(NextAction::PropagateError(err)),
            ),
        },
        Effect::Aborted(res) => {
            let next_action = match res {
                Ok(()) => NextAction::FetchProgress,
                Err(err) => NextAction::PropagateError(err),
            };
            (StateMutation::Unchanged, Some(next_action))
        }
        Effect::StatusFetched(res) => {
            debug_assert_eq!(state.control, ControlState::Busy);
            state.control = ControlState::Idle;
            match res {
                Ok(new_status) => {
                    let new_status = RemoteData::ready(new_status);
                    if state.remote.status != new_status {
                        (StateMutation::MaybeChanged, None)
                    } else {
                        (StateMutation::Unchanged, None)
                    }
                }
                Err(err) => (
                    StateMutation::Unchanged,
                    Some(NextAction::PropagateError(err)),
                ),
            }
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
                    NextAction::FetchProgress
                }
                Err(err) => {
                    state.remote.last_scan_outcome.reset();
                    NextAction::PropagateError(err)
                }
            };
            (StateMutation::MaybeChanged, Some(next_action))
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
                    NextAction::FetchProgress
                }
                Err(err) => {
                    state.remote.last_import_outcome.reset();
                    NextAction::PropagateError(err)
                }
            };
            (StateMutation::MaybeChanged, Some(next_action))
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
                    NextAction::FetchProgress
                }
                Err(err) => {
                    state.remote.last_untrack_outcome.reset();
                    NextAction::PropagateError(err)
                }
            };
            (StateMutation::MaybeChanged, Some(next_action))
        }
        Effect::ErrorOccurred(error) => (
            StateMutation::Unchanged,
            Some(NextAction::PropagateError(error)),
        ),
    }
}

pub async fn dispatch_next_action<E: From<Effect> + From<Intent> + fmt::Debug>(
    shared_env: Arc<Environment>,
    event_tx: EventSender<E>,
    next_action: NextAction,
) {
    match next_action {
        NextAction::FetchStatus {
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
            emit_event(&event_tx, Effect::StatusFetched(res));
        }
        NextAction::FetchProgress => {
            let res = on_fetch_progress(&shared_env.client, &shared_env.api_url).await;
            emit_event(&event_tx, Effect::ProgressFetched(res));
        }
        NextAction::StartScan {
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
            emit_event(&event_tx, Effect::ScanFinished(res));
        }
        NextAction::StartImport {
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
            emit_event(&event_tx, Effect::ImportFinished(res));
        }
        NextAction::Abort => {
            let res = on_abort(&shared_env.client, &shared_env.api_url).await;
            emit_event(&event_tx, Effect::Aborted(res));
        }
        NextAction::Untrack {
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
            emit_event(&event_tx, Effect::Untracked(res));
        }
        NextAction::PropagateError(error) => {
            emit_event(&event_tx, Effect::ErrorOccurred(error));
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
