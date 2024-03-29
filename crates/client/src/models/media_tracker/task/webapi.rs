// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::CollectionUid;

use super::{
    super::Effect, FetchStatus, PendingTask, StartFindUntrackedFiles, StartImportFiles,
    StartScanDirectories, Task, UntrackDirectories,
};
use crate::webapi::{receive_response_body, ClientEnvironment};

impl Task {
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task {self:?}");
        match self {
            Self::Pending { token, task } => match task {
                PendingTask::FetchProgress => {
                    let result = fetch_progress(env).await;
                    Effect::FetchProgressFinished { token, result }
                }
                PendingTask::FetchStatus(FetchStatus {
                    collection_uid,
                    params,
                }) => {
                    let result = fetch_status(env, &collection_uid, params).await;
                    Effect::FetchStatusFinished { token, result }
                }
                PendingTask::StartScanDirectories(StartScanDirectories {
                    collection_uid,
                    params,
                }) => {
                    let result = start_scan_directories(env, &collection_uid, params).await;
                    Effect::ScanDirectoriesFinished { token, result }
                }
                PendingTask::StartImportFiles(StartImportFiles {
                    collection_uid,
                    params,
                }) => {
                    let result = start_import_files(env, &collection_uid, params).await;
                    Effect::ImportFilesFinished { token, result }
                }
                PendingTask::StartFindUntrackedFiles(StartFindUntrackedFiles {
                    collection_uid,
                    params,
                }) => {
                    let result = start_find_untracked_files(env, &collection_uid, params).await;
                    Effect::FindUntrackedFilesFinished { token, result }
                }
                PendingTask::UntrackDirectories(UntrackDirectories {
                    collection_uid,
                    params,
                }) => {
                    let result = untrack_directories(env, &collection_uid, params).await;
                    Effect::UntrackDirectoriesFinished { token, result }
                }
            },
        }
    }
}

async fn fetch_progress<E: ClientEnvironment>(
    env: &E,
) -> anyhow::Result<aoide_core_api::media::tracker::Progress> {
    let request_url = env.join_api_url("mt/progress")?;
    let request = env.client().get(request_url);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let progress =
        serde_json::from_slice::<aoide_core_api_json::media::tracker::Progress>(&response_body)
            .map(Into::into)?;
    log::debug!("Received progress: {progress:?}");
    Ok(progress)
}

async fn fetch_status<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::tracker::query_status::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::Status> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/mt/query-status"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let status =
        serde_json::from_slice::<aoide_core_api_json::media::tracker::Status>(&response_body)
            .map(Into::into)?;
    log::debug!("Received status: {status:?}");
    Ok(status)
}

async fn start_scan_directories<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::tracker::scan_directories::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::scan_directories::Outcome> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/mt/scan-directories"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::scan_directories::Outcome,
    >(&response_body)
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    log::debug!("Scanning directories succeeded: {outcome:?}");
    Ok(outcome)
}

async fn untrack_directories<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::tracker::untrack_directories::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::untrack_directories::Outcome> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/mt/untrack-directories"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::untrack_directories::Outcome,
    >(&response_body)
    .map_err(Into::into)
    .and_then(TryInto::try_into)?;
    log::debug!("Untracking directories succeeded: {outcome:?}");
    Ok(outcome)
}

async fn start_import_files<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::tracker::import_files::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::import_files::Outcome> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/mt/import-files"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::import_files::Outcome,
    >(&response_body)
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    log::debug!("Importing files succeeded: {outcome:?}");
    Ok(outcome)
}

async fn start_find_untracked_files<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::tracker::find_untracked_files::Params>,
) -> anyhow::Result<aoide_core_api::media::tracker::find_untracked_files::Outcome> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/mt/find-untracked-files"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::tracker::find_untracked_files::Outcome,
    >(&response_body)
    .map_err(anyhow::Error::from)
    .and_then(|outcome| outcome.try_into().map_err(anyhow::Error::from))?;
    log::debug!("Finding untracked files succeeded: {outcome:?}");
    Ok(outcome)
}
