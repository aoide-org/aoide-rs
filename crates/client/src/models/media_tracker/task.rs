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

use crate::{receive_response_body, WebClientEnvironment};

use super::Effect;

#[derive(Debug)]
pub enum Task {
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    FetchProgress,
    StartScanDirectories {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::scan_directories::Params,
    },
    StartImportFiles {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::import_files::Params,
    },
    StartFindUntrackedFiles {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::find_untracked_files::Params,
    },
    UntrackDirectories {
        collection_uid: EntityUid,
        params: aoide_core_api::media::tracker::untrack_directories::Params,
    },
}

impl Task {
    pub async fn execute<E: WebClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::FetchProgress => {
                let res = fetch_progress(env).await;
                Effect::FetchProgressFinished(res)
            }
            Self::FetchStatus {
                collection_uid,
                root_url,
            } => {
                let params = aoide_core_api::media::tracker::query_status::Params { root_url };
                let res = fetch_status(env, &collection_uid, params).await;
                Effect::FetchStatusFinished(res)
            }
            Self::StartScanDirectories {
                collection_uid,
                params,
            } => {
                let res = start_scan_directories(env, &collection_uid, params).await;
                Effect::ScanDirectoriesFinished(res)
            }
            Self::StartImportFiles {
                collection_uid,
                params,
            } => {
                let res = start_import_files(env, &collection_uid, params).await;
                Effect::ImportFilesFinished(res)
            }
            Self::StartFindUntrackedFiles {
                collection_uid,
                params,
            } => {
                let res = start_find_untracked_files(env, &collection_uid, params).await;
                Effect::FindUntrackedFilesFinished(res)
            }
            Self::UntrackDirectories {
                collection_uid,
                params,
            } => {
                let res = untrack_directories(env, &collection_uid, params).await;
                Effect::UntrackDirectoriesFinished(res)
            }
        }
    }
}

async fn fetch_progress<E: WebClientEnvironment>(
    env: &E,
) -> anyhow::Result<aoide_core_api::media::tracker::Progress> {
    let request_url = env.join_api_url("mt/progress")?;
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let progress =
        serde_json::from_slice::<aoide_core_api_json::media::tracker::Progress>(&response_body)
            .map(Into::into)?;
    log::debug!("Received progress: {:?}", progress);
    Ok(progress)
}

async fn fetch_status<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::tracker::query_status::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::Status> {
    let request_url = env.join_api_url(&format!("c/{}/mt/query-status", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let status =
        serde_json::from_slice::<aoide_core_api_json::media::tracker::Status>(&response_body)
            .map(Into::into)?;
    log::debug!("Received status: {:?}", status);
    Ok(status)
}

async fn start_scan_directories<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::tracker::scan_directories::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::scan_directories::Outcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/scan-directories", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::scan_directories::Outcome,
    >(&response_body)
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    log::debug!("Scanning finished: {:?}", outcome);
    Ok(outcome)
}

async fn untrack_directories<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::tracker::untrack_directories::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::untrack_directories::Outcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/untrack-directories", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::untrack_directories::Outcome,
    >(&response_body)
    .map_err(Into::into)
    .and_then(TryInto::try_into)?;
    log::debug!("Untracking finished: {:?}", outcome);
    Ok(outcome)
}

async fn start_import_files<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::tracker::import_files::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::import_files::Outcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/import-files", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::import_files::Outcome,
    >(&response_body)
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    log::debug!("Importing finished: {:?}", outcome);
    Ok(outcome)
}

async fn start_find_untracked_files<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::tracker::find_untracked_files::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::find_untracked_files::Outcome> {
    let request_url = env.join_api_url(&format!("c/{}/mt/find-untracked-files", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::find_untracked_files::Outcome,
    >(&response_body)
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    log::debug!("Finding untracked entries finished: {:?}", outcome);
    Ok(outcome)
}
