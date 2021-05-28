use aoide_core::{
    entity::EntityUid,
    usecases::media::tracker::{ImportingProgress, Progress, ScanningProgress, Status},
};
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
    progress: Option<Progress>,
}

impl State {
    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn progress(&self) -> Option<&Progress> {
        self.progress.as_ref()
    }

    pub fn is_idle(&self) -> bool {
        self.progress == Some(Progress::Idle)
    }

    pub fn scanning_progress(&self) -> Option<&ScanningProgress> {
        match self.progress() {
            Some(Progress::Scanning(progress)) => Some(progress),
            _ => None,
        }
    }

    pub fn importing_progress(&self) -> Option<&ImportingProgress> {
        match self.progress() {
            Some(Progress::Importing(progress)) => Some(progress),
            _ => None,
        }
    }

    pub fn replace_status(&mut self, new_status: impl Into<Option<Status>>) -> Option<Status> {
        let old_status = self.status.take();
        self.status = new_status.into();
        old_status
    }

    pub fn reset_status(&mut self) -> Option<Status> {
        self.replace_status(None)
    }

    pub fn replace_progress(
        &mut self,
        new_progress: impl Into<Option<Progress>>,
    ) -> Option<Progress> {
        let old_progress = self.progress.take();
        self.progress = new_progress.into();
        old_progress
    }

    pub fn reset_progress(&mut self) -> Option<Progress> {
        self.replace_progress(None)
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
    log::debug!("Received status: {:?}", status);
    Ok(status)
}

pub async fn scan(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
    root_url: &Url,
) -> anyhow::Result<()> {
    let url = base_url.join(&format!("c/{}/media-tracker/scan", collection_uid))?;
    let body = serde_json::to_vec(&serde_json::json!({
        "rootUrl": root_url.to_string(),
    }))
    .map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to serialize request body when starting media tracker scan")
    })?;
    log::info!("Scanning {}...", root_url);
    let response = client
        .post(url)
        .body(body)
        .send()
        .await
        .map_err(|err| anyhow::Error::from(err).context("media tracker scan failure"))?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Media tracker scan failed: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload of media tracker scan")
    })?;
    log::error!("TODO: Deserialize and return response payload: {:?}", bytes);
    Ok(())
}

pub async fn import(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
    root_url: &Url,
) -> anyhow::Result<()> {
    let url = base_url.join(&format!("c/{}/media-tracker/import", collection_uid))?;
    let body = serde_json::to_vec(&serde_json::json!({
        "rootUrl": root_url.to_string(),
    }))
    .map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to serialize request body when starting media tracker import")
    })?;
    log::info!("Importing {}...", root_url);
    let response = client
        .post(url)
        .body(body)
        .send()
        .await
        .map_err(|err| anyhow::Error::from(err).context("media tracker import failed"))?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Media tracker import failed: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload of media tracker import")
    })?;
    log::error!("TODO: Deserialize and return response payload: {:?}", bytes);
    Ok(())
}

pub async fn get_progress(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
) -> anyhow::Result<Progress> {
    let url = base_url.join(&format!("c/{}/media-tracker/progress", collection_uid))?;
    let response =
        client.get(url).send().await.map_err(|err| {
            anyhow::Error::from(err).context("Failed to get media tracker progress")
        })?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to get media tracker progress: response status = {}",
            response.status()
        );
    }
    let bytes = response.bytes().await.map_err(|err| {
        anyhow::Error::from(err)
            .context("Failed to receive response playload when getting media tracker progress")
    })?;
    let progress =
        serde_json::from_slice::<aoide_core_serde::usecases::media::tracker::Progress>(&bytes)
            .map(Into::into)
            .map_err(|err| {
                anyhow::Error::from(err).context(
                    "Failed to deserialize response payload when getting media tracker progress",
                )
            })?;
    log::debug!("Received progress: {:?}", progress);
    Ok(progress)
}

pub async fn abort(
    client: &Client,
    base_url: &Url,
    collection_uid: &EntityUid,
) -> anyhow::Result<()> {
    let url = base_url.join(&format!("c/{}/media-tracker/abort", collection_uid))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| anyhow::Error::from(err).context("Failed to abort media tracker"))?;
    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to abort media tracker: response status = {}",
            response.status()
        );
    }
    Ok(())
}
