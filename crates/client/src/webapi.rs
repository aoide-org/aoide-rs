// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use bytes::Bytes;

use reqwest::{Client, Response, Url};

pub trait ClientEnvironment {
    fn client(&self) -> &Client;
    fn join_api_url(&self, query_suffix: &str) -> anyhow::Result<Url>;
}

pub async fn receive_response_body(response: Response) -> anyhow::Result<Bytes> {
    let response_status = response.status();
    let bytes = response.bytes().await?;
    if !response_status.is_success() {
        let json = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default();
        let err = if json.is_null() {
            anyhow::anyhow!("{}", response_status)
        } else {
            anyhow::anyhow!("{}", response_status).context(json)
        };
        return Err(err);
    }
    Ok(bytes)
}
