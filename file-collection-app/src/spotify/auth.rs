// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Authorization Code Flow with PKCE.
//!
//! <https://developer.spotify.com/documentation/web-api/tutorials/code-pkce-flow>

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context as _, anyhow, bail};
use rand::{RngExt as _, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpListener,
};
use url::Url;

const AUTHORIZE_URL: &str = "https://accounts.spotify.com/authorize";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

const SCOPES: &[&str] = &[
    "user-library-read",
    "playlist-modify-private",
    "playlist-modify-public",
];

const REDIRECT_PATH: &str = "/callback";

const TOKEN_CACHE_FILE_NAME: &str = "spotify_token.json";

/// Refresh access tokens ahead of their expiration.
const EXPIRATION_MARGIN: Duration = Duration::from_secs(60);

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenCache {
    client_id: String,
    scope: String,
    refresh_token: String,
}

#[derive(Debug)]
pub(super) struct Authorization {
    client_id: String,
    cache_file_path: PathBuf,
    access_token: String,
    expires_at: Instant,
    refresh_token: String,
}

impl Authorization {
    /// Authorize, reusing a cached refresh token if possible.
    pub(super) async fn obtain(
        http: &reqwest::Client,
        client_id: String,
        redirect_port: u16,
        config_dir: &Path,
    ) -> anyhow::Result<Self> {
        let cache_file_path = config_dir.join(TOKEN_CACHE_FILE_NAME);
        if let Some(refresh_token) = load_cached_refresh_token(&cache_file_path, &client_id) {
            log::info!("Refreshing cached Spotify authorization");
            match request_token(
                http,
                &[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", &refresh_token),
                    ("client_id", &client_id),
                ],
            )
            .await
            {
                Ok(response) => {
                    return Self::from_response(
                        client_id,
                        cache_file_path,
                        response,
                        refresh_token,
                    );
                }
                Err(err) => {
                    log::warn!("Failed to refresh cached Spotify authorization: {err:#}");
                }
            }
        }
        let redirect_uri = format!("http://127.0.0.1:{redirect_port}{REDIRECT_PATH}");
        let code_verifier = random_string(64);
        let code_challenge =
            data_encoding::BASE64URL_NOPAD.encode(&Sha256::digest(code_verifier.as_bytes()));
        let state = random_string(16);
        let mut authorize_url = Url::parse(AUTHORIZE_URL).expect("valid URL");
        authorize_url
            .query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("code_challenge_method", "S256")
            .append_pair("code_challenge", &code_challenge)
            .append_pair("scope", &SCOPES.join(" "))
            .append_pair("state", &state);
        // Bind before asking the user to open the URL.
        let listener = TcpListener::bind(("127.0.0.1", redirect_port))
            .await
            .with_context(|| format!("bind redirect listener on port {redirect_port}"))?;
        eprintln!("Open the following URL in your browser to authorize access to Spotify:");
        eprintln!();
        eprintln!("{authorize_url}");
        eprintln!();
        let code = receive_authorization_code(&listener, &state).await?;
        drop(listener);
        let response = request_token(
            http,
            &[
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", &redirect_uri),
                ("client_id", &client_id),
                ("code_verifier", &code_verifier),
            ],
        )
        .await?;
        Self::from_response(client_id, cache_file_path, response, String::new())
    }

    fn from_response(
        client_id: String,
        cache_file_path: PathBuf,
        response: TokenResponse,
        previous_refresh_token: String,
    ) -> anyhow::Result<Self> {
        let TokenResponse {
            access_token,
            expires_in,
            refresh_token,
            scope,
        } = response;
        // The refresh token might not be renewed.
        let refresh_token = refresh_token.unwrap_or(previous_refresh_token);
        if refresh_token.is_empty() {
            bail!("missing refresh token");
        }
        if let Some(missing_scope) = SCOPES.iter().find(|required| {
            !scope
                .split_whitespace()
                .any(|granted| granted == **required)
        }) {
            // Not cached to enforce re-authorization next time.
            bail!("missing scope {missing_scope}");
        }
        store_cached_refresh_token(
            &cache_file_path,
            &TokenCache {
                client_id: client_id.clone(),
                scope,
                refresh_token: refresh_token.clone(),
            },
        );
        Ok(Self {
            client_id,
            cache_file_path,
            access_token,
            expires_at: Instant::now() + Duration::from_secs(expires_in),
            refresh_token,
        })
    }

    /// A valid access token, refreshed if needed.
    pub(super) async fn access_token(&mut self, http: &reqwest::Client) -> anyhow::Result<&str> {
        if Instant::now() + EXPIRATION_MARGIN >= self.expires_at {
            log::info!("Refreshing Spotify access token");
            let response = request_token(
                http,
                &[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", &self.refresh_token),
                    ("client_id", &self.client_id),
                ],
            )
            .await?;
            *self = Self::from_response(
                std::mem::take(&mut self.client_id),
                std::mem::take(&mut self.cache_file_path),
                response,
                std::mem::take(&mut self.refresh_token),
            )?;
        }
        Ok(&self.access_token)
    }
}

#[must_use]
fn random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

async fn request_token(
    http: &reqwest::Client,
    form: &[(&str, &str)],
) -> anyhow::Result<TokenResponse> {
    let response = http.post(TOKEN_URL).form(form).send().await?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!("token request failed with status {status}: {body}");
    }
    response.json().await.map_err(Into::into)
}

#[must_use]
fn load_cached_refresh_token(file_path: &Path, client_id: &str) -> Option<String> {
    let json = std::fs::read(file_path).ok()?;
    let cache = serde_json::from_slice::<TokenCache>(&json)
        .inspect_err(|err| log::warn!("Failed to parse cached Spotify token: {err:#}"))
        .ok()?;
    (cache.client_id == client_id).then_some(cache.refresh_token)
}

fn store_cached_refresh_token(file_path: &Path, cache: &TokenCache) {
    let result = serde_json::to_vec_pretty(cache)
        .map_err(anyhow::Error::from)
        .and_then(|json| write_private_file(file_path, &json).map_err(Into::into));
    if let Err(err) = result {
        log::warn!(
            "Failed to cache Spotify token in {file_path}: {err:#}",
            file_path = file_path.display()
        );
    }
}

/// Write a file that is only readable by the current user.
fn write_private_file(file_path: &Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(file_path)?.write_all(contents)
}

/// Wait for the redirect from the browser.
async fn receive_authorization_code(
    listener: &TcpListener,
    expected_state: &str,
) -> anyhow::Result<String> {
    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut buf = vec![0; 8192];
        let mut len = 0;
        // Only the request line is needed.
        while len < buf.len() && !buf[..len].windows(2).any(|w| w == b"\r\n") {
            let read = stream.read(&mut buf[len..]).await?;
            if read == 0 {
                break;
            }
            len += read;
        }
        let request = String::from_utf8_lossy(&buf[..len]);
        let target = request
            .lines()
            .next()
            .and_then(|request_line| request_line.strip_prefix("GET "))
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or_default();
        let url = Url::parse("http://127.0.0.1")
            .expect("valid URL")
            .join(target)?;
        if url.path() != REDIRECT_PATH {
            // Ignore unrelated requests, e.g. for favicon.ico.
            drop(
                stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                    .await,
            );
            continue;
        }
        let query_param = |name: &str| {
            url.query_pairs()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.into_owned())
        };
        let result = if query_param("state").as_deref() != Some(expected_state) {
            Err(anyhow!("state mismatch"))
        } else if let Some(error) = query_param("error") {
            Err(anyhow!("authorization denied: {error}"))
        } else {
            query_param("code").ok_or_else(|| anyhow!("missing authorization code"))
        };
        let message = match &result {
            Ok(_) => "Authorization succeeded. You may close this window.".to_owned(),
            Err(err) => format!("Authorization failed: {err:#}"),
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: \
             {len}\r\nConnection: close\r\n\r\n{message}",
            len = message.len()
        );
        drop(stream.write_all(response.as_bytes()).await);
        return result;
    }
}
