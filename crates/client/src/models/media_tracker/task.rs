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

use aoide_core::{entity::EntityUid, util::url::BaseUrl};

use aoide_core_ext::media::tracker::{
    import::Outcome as ImportOutcome, scan::Outcome as ScanOutcome,
    untrack::Outcome as UntrackOutcome, Progress, Status,
};

use crate::{receive_response_body, WebClientEnvironment};

use super::Effect;

#[derive(Debug)]
pub enum Task {
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    FetchProgress,
    StartScan {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    StartImport {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    Abort,
    Untrack {
        collection_uid: EntityUid,
        root_url: BaseUrl,
    },
    PurgeOrphanedAndUntracked {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
}

impl Task {
    pub async fn execute<E: WebClientEnvironment>(self, env: &E) -> Effect {
        tracing::debug!("Executing task: {:?}", self);
        match self {
            Self::FetchStatus {
                collection_uid,
                root_url,
            } => {
                let params = aoide_core_ext::media::tracker::query_status::Params { root_url };
                let res = fetch_status(env, &collection_uid, params).await;
                Effect::StatusFetched(res)
            }
            Self::FetchProgress => {
                let res = fetch_progress(env).await;
                Effect::ProgressFetched(res)
            }
            Self::StartScan {
                collection_uid,
                root_url,
            } => {
                let params = aoide_core_ext::media::tracker::FsTraversalParams {
                    root_url,
                    ..Default::default()
                };
                let res = start_scan(env, &collection_uid, params).await;
                Effect::ScanFinished(res)
            }
            Self::StartImport {
                collection_uid,
                root_url,
            } => {
                let params = aoide_core_ext::media::tracker::import::Params {
                    root_url,
                    ..Default::default()
                };
                let res = start_import(env, &collection_uid, params).await;
                Effect::ImportFinished(res)
            }
            Self::Abort => {
                let res = abort(env).await;
                Effect::Aborted(res)
            }
            Self::Untrack {
                collection_uid,
                root_url,
            } => {
                let params = aoide_core_ext::media::tracker::untrack::Params {
                    root_url,
                    status: None,
                };
                let res = untrack(env, &collection_uid, params).await;
                Effect::Untracked(res)
            }
            Self::PurgeOrphanedAndUntracked {
                collection_uid,
                root_url,
            } => {
                let params = aoide_core_ext::track::purge_untracked::Params {
                    root_url,
                    untrack_orphaned_directories: Some(true),
                };
                let res = purge_untracked(env, &collection_uid, params).await;
                Effect::PurgeOrphanedAndUntracked(res)
            }
        }
    }
}

async fn fetch_status<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_ext_serde::media::tracker::query_status::Params>,
) -> anyhow::Result<Status> {
    let request_url = env.join_api_url(&format!("c/{}/mt/query-status", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let status =
        serde_json::from_slice::<aoide_core_ext_serde::media::tracker::Status>(&response_body)
            .map(Into::into)?;
    tracing::debug!("Received status: {:?}", status);
    Ok(status)
}

async fn fetch_progress<E: WebClientEnvironment>(env: &E) -> anyhow::Result<Progress> {
    let request_url = env.join_api_url("mt/progress")?;
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let progress =
        serde_json::from_slice::<aoide_core_ext_serde::media::tracker::Progress>(&response_body)
            .map(Into::into)?;
    tracing::debug!("Received progress: {:?}", progress);
    Ok(progress)
}

async fn start_scan<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_ext_serde::media::tracker::FsTraversalParams>,
) -> anyhow::Result<ScanOutcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/scan", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<aoide_core_ext_serde::media::tracker::scan::Outcome>(
        &response_body,
    )
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    tracing::debug!("Scan finished: {:?}", outcome);
    Ok(outcome)
}

async fn start_import<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_ext_serde::media::tracker::import::Params>,
) -> anyhow::Result<ImportOutcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/import", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<aoide_core_ext_serde::media::tracker::import::Outcome>(
        &response_body,
    )
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    tracing::debug!("Import finished: {:?}", outcome);
    Ok(outcome)
}

pub async fn abort<E: WebClientEnvironment>(env: &E) -> anyhow::Result<()> {
    let request_url = env.join_api_url("mt/abort")?;
    let request = env.client().post(request_url);
    let response = request.send().await?;
    let _ = receive_response_body(response).await?;
    Ok(())
}

async fn untrack<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_ext_serde::media::tracker::untrack::Params>,
) -> anyhow::Result<UntrackOutcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/untrack", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<aoide_core_ext_serde::media::tracker::untrack::Outcome>(
        &response_body,
    )
    .map(Into::into)?;
    tracing::debug!("Untrack finished: {:?}", outcome);
    Ok(outcome)
}

async fn purge_untracked<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_ext_serde::track::purge_untracked::Params>,
) -> anyhow::Result<()> {
    let request_url = env.join_api_url(&format!("c/{}/mt/purge-untracked", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<serde_json::Value>(&response_body)?;
    tracing::debug!("Purge orphaned and untracked finished: {:?}", outcome);
    Ok(())
}
