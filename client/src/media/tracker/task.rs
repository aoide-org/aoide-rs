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

use crate::{receive_response_body, Environment};

use super::Effect;

use aoide_core::{
    entity::EntityUid,
    usecases::media::tracker::{
        import::Outcome as ImportOutcome, scan::Outcome as ScanOutcome,
        untrack::Outcome as UntrackOutcome, Progress, Status,
    },
};

use reqwest::Url;

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

impl Task {
    pub async fn execute_with(self, env: &Environment) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Task::FetchStatus {
                collection_uid,
                root_url,
            } => {
                let res = fetch_status(env, &collection_uid, root_url.as_ref()).await;
                Effect::StatusFetched(res)
            }
            Task::FetchProgress => {
                let res = fetch_progress(env).await;
                Effect::ProgressFetched(res)
            }
            Task::StartScan {
                collection_uid,
                root_url,
            } => {
                let res = start_scan(env, &collection_uid, root_url.as_ref()).await;
                Effect::ScanFinished(res)
            }
            Task::StartImport {
                collection_uid,
                root_url,
            } => {
                let res = start_import(env, &collection_uid, root_url.as_ref()).await;
                Effect::ImportFinished(res)
            }
            Task::Abort => {
                let res = abort(env).await;
                Effect::Aborted(res)
            }
            Task::Untrack {
                collection_uid,
                root_url,
            } => {
                let res = untrack(env, &collection_uid, &root_url).await;
                Effect::Untracked(res)
            }
        }
    }
}

async fn fetch_status(
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

async fn fetch_progress(env: &Environment) -> anyhow::Result<Progress> {
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

async fn start_scan(
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

async fn start_import(
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

pub async fn abort(env: &Environment) -> anyhow::Result<()> {
    let request_url = env.join_api_url("media-tracker/abort")?;
    let request = env.client().post(request_url);
    let response = request.send().await?;
    let _ = receive_response_body(response).await?;
    Ok(())
}

async fn untrack(
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
