use aoide_core::{entity::EntityUid, media::tracker::Status};
use reqwest::{Client, Url};

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

#[derive(Debug, Clone, Default)]
pub struct State {
    status: Option<Status>,
}

impl State {
    pub fn status(&self) -> &Option<Status> {
        &self.status
    }

    pub fn replace_status(&mut self, new_status: impl Into<Option<Status>>) -> Option<Status> {
        let old_status = self.status.take();
        self.status = new_status.into();
        old_status
    }

    pub fn reset_status(&mut self) -> Option<Status> {
        self.replace_status(None)
    }
}

pub async fn query_status(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
    root_url: &Url,
) -> anyhow::Result<Status> {
    let url = base_url.join(&format!("c/{}/media-tracker/query-status", collection_uid))?;
    let body = serde_json::to_vec(&serde_json::json!({
        "rootUrl": root_url.to_string(),
    }))
    .map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to serialize request body when querying media tracker status")
    })?;
    let response =
        client.post(url).body(body).send().await.map_err(|err| {
            anyhow::Error::from(err).context("Failed to query media tracker status")
        })?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to query media tracker status: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload when querying media tracker status")
    })?;
    let status = serde_json::from_slice::<aoide_core_serde::media::tracker::Status>(&bytes)
        .map(Into::into)
        .map_err(|err| {
            anyhow::Error::from(err).context(
                "Failed to deserialize response payload when querying media tracker status",
            )
        })?;
    log::debug!("Loaded status: {:?}", status);
    Ok(status)
}
