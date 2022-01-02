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

use super::{Effect, Intent};

use aoide_client::{
    models::{active_collection, media_sources, media_tracker},
    receive_response_body, WebClientEnvironment,
};

use std::time::Instant;

#[derive(Debug)]
pub enum Task {
    DeferredIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    ActiveCollection(active_collection::Task),
    MediaSources(media_sources::Task),
    MediaTracker(media_tracker::Task),
    AbortPendingRequest,
}

impl From<active_collection::Task> for Task {
    fn from(task: active_collection::Task) -> Self {
        Self::ActiveCollection(task)
    }
}

impl From<media_sources::Task> for Task {
    fn from(task: media_sources::Task) -> Self {
        Self::MediaSources(task)
    }
}

impl From<media_tracker::Task> for Task {
    fn from(task: media_tracker::Task) -> Self {
        Self::MediaTracker(task)
    }
}

impl Task {
    pub async fn execute<E: WebClientEnvironment>(self, env: &E) -> Effect {
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
        }
    }
}

pub async fn abort<E: WebClientEnvironment>(env: &E) -> anyhow::Result<()> {
    let request_url = env.join_api_url("storage/abort-current-task")?;
    let request = env.client().post(request_url);
    let response = request.send().await?;
    let _ = receive_response_body(response).await?;
    Ok(())
}
