// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Create Spotify playlists from track search queries.

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context as _, bail};
use aoide::media::content::ContentMetadata;
use clap::Parser;

use crate::{Library, read_search_params, search_tracks_headless};

mod auth;
mod client;
mod matching;

use self::{
    auth::Authorization,
    client::{ADD_PLAYLIST_ITEMS_LIMIT, Client, NewPlaylist, Track},
    matching::NormalizedTrack,
};

const DEFAULT_REDIRECT_PORT: u16 = 8888;

#[derive(Debug, Parser)]
pub struct CliOptions {
    /// JSON file with the track search query.
    ///
    /// Reads from stdin if omitted or "-".
    query_file: Option<PathBuf>,

    /// Name of the new playlist.
    #[arg(long)]
    name: String,

    /// Description of the new playlist.
    #[arg(long)]
    description: Option<String>,

    /// Create a public instead of a private playlist.
    #[arg(long)]
    public: bool,

    /// Client ID of your app in the Spotify developer dashboard.
    ///
    /// The app must have the redirect URI <http://127.0.0.1:PORT/callback> registered.
    #[arg(long, env = "SPOTIFY_CLIENT_ID")]
    client_id: String,

    /// Port of the local redirect URI for authorization.
    #[arg(long, default_value_t = DEFAULT_REDIRECT_PORT)]
    redirect_port: u16,

    /// Only print the matched tracks without creating a playlist.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchSource {
    Liked,
    Search,
}

#[derive(Debug, Clone)]
struct Match {
    track: Track,
    source: MatchSource,
    liked: bool,
    score: f64,
}

#[derive(Debug)]
struct LocalTrack {
    artist: String,
    title: String,
    duration: Option<Duration>,
}

/// Liked tracks of the current user, indexed for matching.
#[derive(Debug, Default)]
struct LikedTracks {
    by_core_title: HashMap<String, Vec<(NormalizedTrack, Track)>>,
    ids: HashSet<String>,
}

impl LikedTracks {
    fn new(tracks: Vec<Track>) -> Self {
        let mut liked = Self::default();
        for track in tracks {
            if track.is_local {
                continue;
            }
            let Some(id) = &track.id else {
                continue;
            };
            liked.ids.insert(id.clone());
            let normalized = normalize_remote(&track);
            liked
                .by_core_title
                .entry(normalized.title.core_key())
                .or_default()
                .push((normalized, track));
        }
        liked
    }

    fn contains(&self, track: &Track) -> bool {
        track.id.as_ref().is_some_and(|id| self.ids.contains(id))
    }

    fn find_best_match(&self, local: &NormalizedTrack) -> Option<Match> {
        self.by_core_title
            .get(&local.title.core_key())?
            .iter()
            .filter_map(|(normalized, track)| {
                local.match_score(normalized).map(|score| Match {
                    track: track.clone(),
                    source: MatchSource::Liked,
                    liked: true,
                    score,
                })
            })
            .max_by(|lhs, rhs| lhs.score.total_cmp(&rhs.score))
    }
}

#[must_use]
fn normalize_remote(track: &Track) -> NormalizedTrack {
    NormalizedTrack::new(
        &track.name,
        track.artists.iter().map(|artist| artist.name.as_str()),
        track.duration_ms.map(Duration::from_millis),
    )
}

#[must_use]
fn format_duration(duration: Option<Duration>) -> String {
    let Some(duration) = duration else {
        return "?:??".to_owned();
    };
    let secs = duration.as_secs();
    format!("{min}:{sec:02}", min = secs / 60, sec = secs % 60)
}

#[must_use]
fn quote_search_term(term: &str) -> String {
    format!("\"{}\"", term.replace('"', " "))
}

async fn search_best_match(
    client: &mut Client,
    liked: &LikedTracks,
    local: &LocalTrack,
    normalized: &NormalizedTrack,
) -> anyhow::Result<Option<Match>> {
    let core_title = normalized.title.core_key();
    let main_artist = normalized.artists.main_artist().unwrap_or_default();
    let queries = [
        // Strict query with field filters.
        format!(
            "track:{title} artist:{artist}",
            title = quote_search_term(&core_title),
            artist = quote_search_term(main_artist),
        ),
        // Relaxed free text query.
        format!(
            "{artist} {title}",
            artist = local.artist,
            title = local.title
        ),
    ];
    for query in queries {
        let candidates = client.search_tracks(&query).await?;
        let best_match = candidates
            .into_iter()
            .filter(|track| !track.is_local)
            .filter_map(|track| {
                let score = normalized.match_score(&normalize_remote(&track))?;
                let liked = liked.contains(&track);
                Some(Match {
                    track,
                    source: MatchSource::Search,
                    liked,
                    score,
                })
            })
            // Prefer liked tracks, then the most similar.
            .max_by(|lhs, rhs| {
                lhs.liked
                    .cmp(&rhs.liked)
                    .then(lhs.score.total_cmp(&rhs.score))
            });
        if best_match.is_some() {
            return Ok(best_match);
        }
    }
    Ok(None)
}

async fn find_local_tracks(
    library: &Library,
    rt: &tokio::runtime::Handle,
    query_file: Option<&Path>,
) -> anyhow::Result<Vec<LocalTrack>> {
    let search_params = match query_file {
        None => read_search_params(io::stdin(), None),
        Some(path) if path.as_os_str() == "-" => read_search_params(io::stdin(), None),
        Some(path) => File::open(path)
            .map_err(Into::into)
            .and_then(|file| read_search_params(io::BufReader::new(file), None)),
    }
    .context("read search params")?;
    let mut local_tracks = Vec::new();
    search_tracks_headless(library, rt, search_params, |fetched_tracks| {
        for fetched in fetched_tracks {
            let track = &fetched.entity.body.track;
            let (Some(artist), Some(title)) = (track.track_artist(), track.track_title()) else {
                log::warn!(
                    "Skipping track without artist or title: {content_link}",
                    content_link = track.media_source.content.link.path
                );
                continue;
            };
            let ContentMetadata::Audio(audio_metadata) = &track.media_source.content.metadata;
            let duration = audio_metadata
                .duration
                .map(|duration| duration.value() / 1000.0)
                .filter(|secs| secs.is_finite() && *secs > 0.0)
                .map(Duration::from_secs_f64);
            local_tracks.push(LocalTrack {
                artist: artist.to_owned(),
                title: title.to_owned(),
                duration,
            });
        }
    })
    .await?;
    Ok(local_tracks)
}

/// Match local tracks and return the unique Spotify URIs in order.
async fn match_tracks(
    client: &mut Client,
    liked: &LikedTracks,
    local_tracks: &[LocalTrack],
    dry_run: bool,
) -> anyhow::Result<Vec<String>> {
    let mut matched_uris = Vec::with_capacity(local_tracks.len());
    let mut unique_uris = HashSet::with_capacity(local_tracks.len());
    let mut unmatched = Vec::new();
    let mut matched_liked_count = 0;
    let mut matched_search_count = 0;
    for (index, local) in local_tracks.iter().enumerate() {
        let normalized =
            NormalizedTrack::new(&local.title, [local.artist.as_str()], local.duration);
        let best_match = match liked.find_best_match(&normalized) {
            Some(best_match) => Some(best_match),
            None => search_best_match(client, liked, local, &normalized).await?,
        };
        let Some(Match {
            track,
            source,
            liked,
            score,
        }) = best_match
        else {
            log::info!(
                "[{index}] No match: {artist} - {title}",
                artist = local.artist,
                title = local.title
            );
            unmatched.push(local);
            continue;
        };
        if liked {
            matched_liked_count += 1;
        }
        if source == MatchSource::Search {
            matched_search_count += 1;
        }
        let remote_artists = track
            .artists
            .iter()
            .map(|artist| artist.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let summary = format!(
            "{artist} - {title} ({duration})\t=> {remote_artists} - {remote_title} \
             ({remote_duration})\t[{source:?}{liked}, score: {score:.2}]\t{uri}",
            artist = local.artist,
            title = local.title,
            duration = format_duration(local.duration),
            remote_title = track.name,
            remote_duration = format_duration(track.duration_ms.map(Duration::from_millis)),
            liked = if liked { ", liked" } else { "" },
            uri = track.uri,
        );
        if dry_run {
            println!("{summary}");
        } else {
            log::info!("[{index}] {summary}");
        }
        if unique_uris.insert(track.uri.clone()) {
            matched_uris.push(track.uri);
        }
    }

    eprintln!(
        "Matched {matched} of {total} track(s) ({matched_liked_count} liked, \
         {matched_search_count} found by search), {unique} unique",
        matched = local_tracks.len() - unmatched.len(),
        total = local_tracks.len(),
        unique = matched_uris.len(),
    );
    if !unmatched.is_empty() {
        eprintln!("Unmatched track(s):");
        for LocalTrack { artist, title, .. } in unmatched {
            eprintln!("  {artist} - {title}");
        }
    }

    Ok(matched_uris)
}

pub(crate) async fn run_cli(
    library: &Library,
    rt: &tokio::runtime::Handle,
    config_dir: &Path,
    options: CliOptions,
) -> anyhow::Result<()> {
    let CliOptions {
        query_file,
        name,
        description,
        public,
        client_id,
        redirect_port,
        dry_run,
    } = options;

    let local_tracks = find_local_tracks(library, rt, query_file.as_deref()).await?;
    eprintln!(
        "Found {count} track(s) in library",
        count = local_tracks.len()
    );
    if local_tracks.is_empty() {
        bail!("no tracks found");
    }

    let http = reqwest::Client::new();
    let auth = Authorization::obtain(&http, client_id, redirect_port, config_dir).await?;
    let mut client = Client::new(http, auth);

    eprintln!("Loading liked tracks from Spotify...");
    let liked = LikedTracks::new(client.saved_tracks().await?);
    eprintln!("Loaded {count} liked track(s)", count = liked.ids.len());

    let matched_uris = match_tracks(&mut client, &liked, &local_tracks, dry_run).await?;
    if dry_run {
        return Ok(());
    }
    if matched_uris.is_empty() {
        bail!("no tracks matched");
    }

    let playlist = client
        .create_playlist(&NewPlaylist {
            name: &name,
            public,
            description: description.as_deref(),
        })
        .await
        .context("create playlist")?;
    for chunk in matched_uris.chunks(ADD_PLAYLIST_ITEMS_LIMIT) {
        client
            .add_playlist_items(&playlist.id, chunk)
            .await
            .context("add tracks to playlist")?;
    }
    eprintln!(
        "Created playlist \"{name}\" with {count} track(s)",
        count = matched_uris.len()
    );
    println!(
        "{url}",
        url = playlist
            .external_urls
            .spotify
            .unwrap_or_else(|| format!("spotify:playlist:{id}", id = playlist.id))
    );
    Ok(())
}
