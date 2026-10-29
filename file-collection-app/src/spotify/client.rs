// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Minimal Spotify Web API client.
//!
//! Uses the endpoints introduced with the February 2026 API changes:
//! <https://developer.spotify.com/documentation/web-api/references/changes/february-2026>

use std::time::Duration;

use anyhow::bail;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::auth::Authorization;

const API_BASE_URL: &str = "https://api.spotify.com/v1";

/// Maximum page size of saved tracks.
const SAVED_TRACKS_LIMIT: u32 = 50;

/// Maximum number of search results (reduced from 50 in February 2026).
pub(super) const SEARCH_LIMIT: u32 = 10;

/// Maximum number of items per request when adding items to a playlist.
pub(super) const ADD_PLAYLIST_ITEMS_LIMIT: usize = 100;

const MAX_RETRIES: usize = 5;

/// Fallback if the `Retry-After` header is missing.
const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Artist {
    pub(super) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Track {
    /// `None` for local files.
    pub(super) id: Option<String>,
    pub(super) uri: String,
    pub(super) name: String,
    pub(super) artists: Vec<Artist>,
    pub(super) duration_ms: Option<u64>,
    #[serde(default)]
    pub(super) is_local: bool,
}

#[derive(Debug, Deserialize)]
struct Page<T> {
    items: Vec<T>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SavedTrack {
    track: Option<Track>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    tracks: Page<Track>,
}

#[derive(Debug, Serialize)]
pub(super) struct NewPlaylist<'a> {
    pub(super) name: &'a str,
    pub(super) public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ExternalUrls {
    pub(super) spotify: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Playlist {
    pub(super) id: String,
    pub(super) external_urls: ExternalUrls,
}

#[derive(Debug, Serialize)]
struct AddItems<'a> {
    uris: &'a [String],
}

#[derive(Debug)]
pub(super) struct Client {
    http: reqwest::Client,
    auth: Authorization,
}

impl Client {
    #[must_use]
    pub(super) const fn new(http: reqwest::Client, auth: Authorization) -> Self {
        Self { http, auth }
    }

    async fn request<T: DeserializeOwned>(
        &mut self,
        method: Method,
        url: &str,
        query: &[(&str, &str)],
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> anyhow::Result<T> {
        let mut retries = 0;
        loop {
            let access_token = self.auth.access_token(&self.http).await?;
            let mut request = self
                .http
                .request(method.clone(), url)
                .bearer_auth(access_token)
                .query(query);
            if let Some(body) = body {
                request = request.json(body);
            }
            let response = request.send().await?;
            let status = response.status();
            if status == StatusCode::TOO_MANY_REQUESTS && retries < MAX_RETRIES {
                let retry_after = response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse().ok())
                    .map_or(DEFAULT_RETRY_AFTER, Duration::from_secs);
                log::info!(
                    "Rate limited, retrying after {retry_after_secs} s",
                    retry_after_secs = retry_after.as_secs()
                );
                tokio::time::sleep(retry_after).await;
                retries += 1;
                continue;
            }
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                bail!("{method} {url} failed with status {status}: {body}");
            }
            return response.json().await.map_err(Into::into);
        }
    }

    /// Fetch all tracks that the current user has liked.
    pub(super) async fn saved_tracks(&mut self) -> anyhow::Result<Vec<Track>> {
        let mut tracks = Vec::new();
        let limit = SAVED_TRACKS_LIMIT.to_string();
        let mut page: Page<SavedTrack> = self
            .request(
                Method::GET,
                &format!("{API_BASE_URL}/me/tracks"),
                &[("limit", &limit)],
                None::<&()>,
            )
            .await?;
        loop {
            tracks.extend(page.items.into_iter().filter_map(|item| item.track));
            log::debug!("Fetched {count} saved track(s)", count = tracks.len());
            let Some(next) = page.next else {
                break;
            };
            // The URL of the next page already contains all query parameters.
            page = self.request(Method::GET, &next, &[], None::<&()>).await?;
        }
        Ok(tracks)
    }

    pub(super) async fn search_tracks(&mut self, query: &str) -> anyhow::Result<Vec<Track>> {
        let limit = SEARCH_LIMIT.to_string();
        let response: SearchResponse = self
            .request(
                Method::GET,
                &format!("{API_BASE_URL}/search"),
                &[
                    ("q", query),
                    ("type", "track"),
                    ("market", "from_token"),
                    ("limit", &limit),
                ],
                None::<&()>,
            )
            .await?;
        Ok(response.tracks.items)
    }

    pub(super) async fn create_playlist(
        &mut self,
        new_playlist: &NewPlaylist<'_>,
    ) -> anyhow::Result<Playlist> {
        self.request(
            Method::POST,
            &format!("{API_BASE_URL}/me/playlists"),
            &[],
            Some(new_playlist),
        )
        .await
    }

    pub(super) async fn add_playlist_items(
        &mut self,
        playlist_id: &str,
        uris: &[String],
    ) -> anyhow::Result<()> {
        debug_assert!(uris.len() <= ADD_PLAYLIST_ITEMS_LIMIT);
        let _response: serde_json::Value = self
            .request(
                Method::POST,
                &format!("{API_BASE_URL}/playlists/{playlist_id}/items"),
                &[],
                Some(&AddItems { uris }),
            )
            .await?;
        Ok(())
    }
}
