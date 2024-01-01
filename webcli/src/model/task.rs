// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::Path, time::Instant};

use aoide_client::{
    models::{collection, media_source, media_tracker},
    webapi::{receive_response_body, ClientEnvironment},
};
use aoide_core_api::{track::find_unsynchronized::UnsynchronizedTrackEntity, Pagination};
use aoide_core_api_json::track::search::{client_query_params, client_request_params};

use super::{CollectionUid, Effect, ExportTracksParams, Intent, Message};

#[derive(Debug)]
pub enum Task {
    ScheduleIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    ActiveCollection(collection::Task),
    MediaSources(media_source::Task),
    MediaTracker(media_tracker::Task),
    AbortPendingRequest,
    FindUnsynchronizedTracks {
        collection_uid: CollectionUid,
        params: aoide_core_api::track::find_unsynchronized::Params,
    },
    ExportTracks {
        collection_uid: CollectionUid,
        params: ExportTracksParams,
    },
}

impl From<collection::Task> for Task {
    fn from(task: collection::Task) -> Self {
        Self::ActiveCollection(task)
    }
}

impl From<media_source::Task> for Task {
    fn from(task: media_source::Task) -> Self {
        Self::MediaSources(task)
    }
}

impl From<media_tracker::Task> for Task {
    fn from(task: media_tracker::Task) -> Self {
        Self::MediaTracker(task)
    }
}

impl Task {
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Message {
        log::debug!("Executing task {self:?}");
        match self {
            Self::ScheduleIntent { not_before, intent } => {
                tokio::time::sleep_until(not_before.into()).await;
                (*intent).into()
            }
            Self::ActiveCollection(task) => Message::Effect(task.execute(env).await.into()),
            Self::MediaSources(task) => Message::Effect(task.execute(env).await.into()),
            Self::MediaTracker(task) => Message::Effect(task.execute(env).await.into()),
            Self::AbortPendingRequest => {
                let res = abort(env).await;
                Effect::AbortFinished(res).into()
            }
            Self::FindUnsynchronizedTracks {
                collection_uid,
                params,
            } => {
                let res = find_unsynchronized_tracks(env, &collection_uid, params).await;
                Effect::FindUnsynchronizedTracksFinished(res).into()
            }
            Self::ExportTracks {
                collection_uid,
                params,
            } => {
                let ExportTracksParams {
                    track_search: search_params,
                    output_file_path,
                } = params;
                let res =
                    export_tracks(env, &collection_uid, search_params, &output_file_path).await;
                Effect::ExportTracksFinished(res).into()
            }
        }
    }
}

async fn abort<E: ClientEnvironment>(env: &E) -> anyhow::Result<()> {
    let request_url = env.join_api_url("storage/abort-current-task")?;
    let request = env.client().post(request_url);
    let response = request.send().await?;
    drop(receive_response_body(response).await?);
    Ok(())
}

async fn find_unsynchronized_tracks<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: aoide_core_api::track::find_unsynchronized::Params,
) -> anyhow::Result<Vec<UnsynchronizedTrackEntity>> {
    // Explicitly define an offset with no limit to prevent using
    // the default limit if no pagination is given!
    let no_pagination = Pagination {
        offset: Some(0),
        limit: None, // unlimited
    };
    let aoide_core_api::track::find_unsynchronized::Params {
        resolve_url_from_content_path,
        content_path_predicate,
    } = params;
    let query_params = client_query_params(resolve_url_from_content_path, false, no_pagination);
    let query_params_urlencoded = serde_urlencoded::to_string(query_params)?;
    let request_url = env.join_api_url(&format!(
        "c/{collection_uid}/t/find-unsynchronized?{query_params_urlencoded}",
    ))?;
    let request_body = serde_json::to_vec(
        &content_path_predicate.map(aoide_core_api_json::filtering::StringPredicate::from),
    )?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let entities: Vec<aoide_core_api_json::track::find_unsynchronized::UnsynchronizedTrackEntity> =
        serde_json::from_slice(&response_body)?;
    Ok(entities.into_iter().map(Into::into).collect())
}

async fn export_tracks<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    search_params: aoide_core_api::track::search::Params,
    output_file_path: &Path,
) -> anyhow::Result<()> {
    // Explicitly define an offset with no limit to prevent using
    // the default limit if no pagination is given!
    let no_pagination = Pagination {
        offset: Some(0),
        limit: None, // unlimited
    };
    let (query_params, search_params) = client_request_params(search_params, false, no_pagination);
    let query_params_urlencoded = serde_urlencoded::to_string(query_params)?;
    let request_url = env.join_api_url(&format!(
        "c/{collection_uid}/t/search?{query_params_urlencoded}"
    ))?;
    let request_body = serde_json::to_vec(&search_params)?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    log::debug!(
        "Writing {num_bytes} bytes into output file '{path}'",
        num_bytes = response_body.len(),
        path = output_file_path.display()
    );
    tokio::fs::write(output_file_path, response_body.as_ref()).await?;
    Ok(())
}
