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

use crate::{prelude::*, receive_response_body};

use aoide_core::{
    entity::EntityUid,
    usecases::media::tracker::{
        import::Outcome as ImportOutcome, scan::Outcome as ScanOutcome,
        untrack::Outcome as UntrackOutcome, Progress, Status,
    },
};

use reqwest::Url;

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
            && (self.remote.progress.get() == Some(&Progress::Idle)
                || self.remote.progress.is_unknown())
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

impl Intent {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::FetchProgress => {
                state.remote.progress.set_pending();
                (StateMutation::Unchanged, Some(Task::FetchProgress.into()))
            }
            Self::Abort => (StateMutation::Unchanged, Some(Task::Abort.into())),
            Self::FetchStatus {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot fetch status while not idle");
                    return (StateMutation::Unchanged, None);
                }
                state.control = ControlState::Busy;
                state.remote.status.set_pending();
                (
                    StateMutation::MaybeChanged,
                    Some(
                        Task::FetchStatus {
                            collection_uid,
                            root_url,
                        }
                        .into(),
                    ),
                )
            }
            Self::StartScan {
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
                    Some(
                        Task::StartScan {
                            collection_uid,
                            root_url,
                        }
                        .into(),
                    ),
                )
            }
            Self::StartImport {
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
                    Some(
                        Task::StartImport {
                            collection_uid,
                            root_url,
                        }
                        .into(),
                    ),
                )
            }
            Self::Untrack {
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
                    Some(
                        Task::Untrack {
                            collection_uid,
                            root_url,
                        }
                        .into(),
                    ),
                )
            }
        }
    }
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ProgressFetched(res) => match res {
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
                    Some(Self::ErrorOccurred(err).into()),
                ),
            },
            Self::Aborted(res) => {
                let next_action = match res {
                    Ok(()) => Task::FetchProgress.into(),
                    Err(err) => Self::ErrorOccurred(err).into(),
                };
                (StateMutation::Unchanged, Some(next_action))
            }
            Self::StatusFetched(res) => {
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
                        Some(Self::ErrorOccurred(err).into()),
                    ),
                }
            }
            Self::ScanFinished(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_scan_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_scan_outcome = RemoteData::ready(outcome);
                        Task::FetchProgress.into()
                    }
                    Err(err) => {
                        state.remote.last_scan_outcome.reset();
                        Self::ErrorOccurred(err).into()
                    }
                };
                (StateMutation::MaybeChanged, Some(next_action))
            }
            Self::ImportFinished(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_import_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_import_outcome = RemoteData::ready(outcome);
                        Task::FetchProgress.into()
                    }
                    Err(err) => {
                        state.remote.last_import_outcome.reset();
                        Self::ErrorOccurred(err).into()
                    }
                };
                (StateMutation::MaybeChanged, Some(next_action))
            }
            Self::Untracked(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_untrack_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_untrack_outcome = RemoteData::ready(outcome);
                        Task::FetchProgress.into()
                    }
                    Err(err) => {
                        state.remote.last_untrack_outcome.reset();
                        Self::ErrorOccurred(err).into()
                    }
                };
                (StateMutation::MaybeChanged, Some(next_action))
            }
            Self::ErrorOccurred(err) => (
                StateMutation::Unchanged,
                Some(Self::ErrorOccurred(err).into()),
            ),
        }
    }
}

impl Task {
    pub async fn execute_with(self, env: &Environment) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Task::FetchStatus {
                collection_uid,
                root_url,
            } => {
                let res = on_fetch_status(env, &collection_uid, root_url.as_ref()).await;
                Effect::StatusFetched(res)
            }
            Task::FetchProgress => {
                let res = on_fetch_progress(env).await;
                Effect::ProgressFetched(res)
            }
            Task::StartScan {
                collection_uid,
                root_url,
            } => {
                let res = on_start_scan(env, &collection_uid, root_url.as_ref()).await;
                Effect::ScanFinished(res)
            }
            Task::StartImport {
                collection_uid,
                root_url,
            } => {
                let res = on_start_import(env, &collection_uid, root_url.as_ref()).await;
                Effect::ImportFinished(res)
            }
            Task::Abort => {
                let res = on_abort(env).await;
                Effect::Aborted(res)
            }
            Task::Untrack {
                collection_uid,
                root_url,
            } => {
                let res = on_untrack(env, &collection_uid, &root_url).await;
                Effect::Untracked(res)
            }
        }
    }
}

async fn on_fetch_status(
    env: &Environment,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<Status> {
    let request_url =
        env.join_api_url(&format!("c/{}/media-tracker/query-status", collection_uid))?;
    let request_body = serde_json::to_vec(&root_url.map(|root_url| {
        serde_json::json!({
            "rootUrl": root_url.to_string(),
        })
    }))?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let status = serde_json::from_slice::<aoide_core_serde::usecases::media::tracker::Status>(
        &response_body,
    )
    .map(Into::into)?;
    log::debug!("Received status: {:?}", status);
    Ok(status)
}

async fn on_fetch_progress(env: &Environment) -> anyhow::Result<Progress> {
    let request_url = env.join_api_url("media-tracker/progress")?;
    let request = env.client().get(request_url);
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
    env: &Environment,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<ScanOutcome> {
    let request_url = env.join_api_url(&format!("c/{}/media-tracker/scan", collection_uid))?;
    let request_body = serde_json::to_vec(&root_url.map(|root_url| {
        serde_json::json!({
            "rootUrl": root_url.to_string(),
        })
    }))?;
    let request = env.client().post(request_url).body(request_body);
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
    env: &Environment,
    collection_uid: &EntityUid,
    root_url: Option<&Url>,
) -> anyhow::Result<ImportOutcome> {
    let request_url = env.join_api_url(&format!("c/{}/media-tracker/import", collection_uid))?;
    let request_body = serde_json::to_vec(&root_url.map(|root_url| {
        serde_json::json!({
            "rootUrl": root_url.to_string(),
        })
    }))?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_serde::usecases::media::tracker::import::Outcome,
    >(&response_body)
    .map(Into::into)?;
    log::debug!("Import finished: {:?}", outcome);
    Ok(outcome)
}

pub async fn on_abort(env: &Environment) -> anyhow::Result<()> {
    let request_url = env.join_api_url("media-tracker/abort")?;
    let request = env.client().post(request_url);
    let response = request.send().await?;
    let _ = receive_response_body(response).await?;
    Ok(())
}

async fn on_untrack(
    env: &Environment,
    collection_uid: &EntityUid,
    root_url: &Url,
) -> anyhow::Result<UntrackOutcome> {
    let request_url = env.join_api_url(&format!("c/{}/media-tracker/untrack", collection_uid))?;
    let request_body = serde_json::to_vec(&serde_json::json!({
        "rootUrl": root_url.to_string(),
    }))?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_serde::usecases::media::tracker::untrack::Outcome,
    >(&response_body)
    .map(Into::into)?;
    log::debug!("Untrack finished: {:?}", outcome);
    Ok(outcome)
}
