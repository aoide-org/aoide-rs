// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{path::Path, time::Instant};

use aoide_client::{
    models::{collection, media_source, media_tracker},
    webapi::{receive_response_body, ClientEnvironment},
};
use aoide_core_api::{track::find_unsynchronized::UnsynchronizedTrackEntity, Pagination};
use aoide_core_api_json::track::search::{client_query_params, client_request_params};

use super::{CollectionUid, Effect, ExportTracksParams, Intent};

#[derive(Debug)]
pub enum Task {
    DeferredIntent {
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
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::DeferredIntent { not_before, intent } => {
                tokio::time::sleep_until(not_before.into()).await;
                Effect::ApplyIntent(*intent)
            }
            Self::ActiveCollection(task) => task.execute(env).await.into(),
            Self::MediaSources(task) => task.execute(env).await.into(),
            Self::MediaTracker(task) => task.execute(env).await.into(),
            Self::AbortPendingRequest => {
                let res = abort(env).await;
                Effect::AbortFinished(res)
            }
            Self::FindUnsynchronizedTracks {
                collection_uid,
                params,
            } => {
                let res = find_unsynchronized_tracks(env, &collection_uid, params).await;
                Effect::FindUnsynchronizedTracksFinished(res)
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
                Effect::ExportTracksFinished(res)
            }
        }
    }
}

async fn abort<E: ClientEnvironment>(env: &E) -> anyhow::Result<()> {
    let request_url = env.join_api_url("storage/abort-current-task")?;
    let request = env.client().post(request_url);
    let response = request.send().await?;
    let _ = receive_response_body(response).await?;
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
    let query_params = client_query_params(resolve_url_from_content_path, no_pagination);
    let request_url = env.join_api_url(&format!(
        "c/{}/t/find-unsynchronized?{}",
        collection_uid,
        serde_urlencoded::to_string(query_params)?
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
    let (query_params, search_params) = client_request_params(search_params, no_pagination);
    let request_url = env.join_api_url(&format!(
        "c/{}/t/search?{}",
        collection_uid,
        serde_urlencoded::to_string(query_params)?
    ))?;
    let request_body = serde_json::to_vec(&search_params)?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    log::debug!(
        "Writing {} bytes into output file {}",
        response_body.len(),
        output_file_path.display()
    );
    tokio::fs::write(output_file_path, response_body.as_ref()).await?;
    Ok(())
}
