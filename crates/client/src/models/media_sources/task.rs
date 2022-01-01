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

use aoide_core::entity::EntityUid;

use crate::{receive_response_body, WebClientEnvironment};

use super::Effect;

#[derive(Debug)]
pub enum Task {
    PurgeOrphaned {
        collection_uid: EntityUid,
        params: aoide_core_api::media::source::purge_orphaned::Params,
    },
    PurgeUntracked {
        collection_uid: EntityUid,
        params: aoide_core_api::media::source::purge_untracked::Params,
    },
}

impl Task {
    pub async fn execute<E: WebClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::PurgeOrphaned {
                collection_uid,
                params,
            } => {
                let res = purge_orphaned(env, &collection_uid, params).await;
                Effect::PurgeOrphanedFinished(res)
            }
            Self::PurgeUntracked {
                collection_uid,
                params,
            } => {
                let res = purge_untracked(env, &collection_uid, params).await;
                Effect::PurgeUntrackedFinished(res)
            }
        }
    }
}

async fn purge_orphaned<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::source::purge_orphaned::Params>,
) -> anyhow::Result<aoide_core_api::media::source::purge_orphaned::Outcome> {
    let request_url = env.join_api_url(&format!("c/{}/ms/purge-orphaned", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::source::purge_orphaned::Outcome,
    >(&response_body)?
    .try_into()?;
    log::debug!("Purge orphaned media sources succeeded: {:?}", outcome);
    Ok(outcome)
}

async fn purge_untracked<E: WebClientEnvironment>(
    env: &E,
    collection_uid: &EntityUid,
    params: impl Into<aoide_core_api_json::media::source::purge_untracked::Params>,
) -> anyhow::Result<aoide_core_api::media::source::purge_untracked::Outcome> {
    let request_url = env.join_api_url(&format!("c/{}/ms/purge-untracked", collection_uid))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::source::purge_untracked::Outcome,
    >(&response_body)?
    .try_into()?;
    log::debug!("Purge untracked media sources succeeded: {:?}", outcome);
    Ok(outcome)
}
