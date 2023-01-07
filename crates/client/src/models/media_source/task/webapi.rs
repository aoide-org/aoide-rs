// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    prelude::*,
    webapi::{receive_response_body, ClientEnvironment},
};

use super::{super::Effect, Task};

impl Task {
    pub async fn execute<E: ClientEnvironment>(self, env: &E) -> Effect {
        log::debug!("Executing task {self:?}");
        match self {
            Self::PurgeOrphaned {
                token,
                collection_uid,
                params,
            } => {
                let result = purge_orphaned(env, &collection_uid, params).await;
                Effect::PurgeOrphanedFinished { token, result }
            }
            Self::PurgeUntracked {
                token,
                collection_uid,
                params,
            } => {
                let result = purge_untracked(env, &collection_uid, params).await;
                Effect::PurgeUntrackedFinished { token, result }
            }
        }
    }
}

async fn purge_orphaned<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::source::purge_orphaned::Params>,
) -> anyhow::Result<aoide_core_api::media::source::purge_orphaned::Outcome> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/ms/purge-orphaned"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::source::purge_orphaned::Outcome,
    >(&response_body)?
    .try_into()?;
    log::debug!("Purge orphaned media sources succeeded: {outcome:?}");
    Ok(outcome)
}

async fn purge_untracked<E: ClientEnvironment>(
    env: &E,
    collection_uid: &CollectionUid,
    params: impl Into<aoide_core_api_json::media::source::purge_untracked::Params>,
) -> anyhow::Result<aoide_core_api::media::source::purge_untracked::Outcome> {
    let request_url = env.join_api_url(&format!("c/{collection_uid}/ms/purge-untracked"))?;
    let request_body = serde_json::to_vec(&params.into())?;
    let request = env.client().post(request_url).body(request_body);
    let response = request.send().await?;
    let response_body = receive_response_body(response).await?;
    let outcome = serde_json::from_slice::<
        aoide_core_api_json::media::source::purge_untracked::Outcome,
    >(&response_body)?
    .try_into()?;
    log::debug!("Purge untracked media sources succeeded: {outcome:?}");
    Ok(outcome)
}
