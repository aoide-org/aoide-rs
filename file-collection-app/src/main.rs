// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    io,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, bail};
use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;
use log::LevelFilter;
use url::Url;

use aoide::{api::media::source::ResolveUrlFromContentPath, util::url::BaseUrl};

pub mod app;
use self::app::App;

pub mod fs;

pub mod library;
use self::library::Library;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static MIMALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug)]
pub struct NoReceiverForEvent;

/// Default log level for debug builds.
#[cfg(debug_assertions)]
const DEFAULT_LOG_FILTER_LEVEL: LevelFilter = LevelFilter::Info;

/// Reduce log verbosity for release builds.
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_FILTER_LEVEL: LevelFilter = LevelFilter::Warn;

fn default_data_dir() -> anyhow::Result<PathBuf> {
    let Some(dir_path) = app_data_dir() else {
        bail!("default data directory is unavailable");
    };
    debug_assert!(dir_path.exists());
    let readonly = dir_path
        .metadata()
        .map(|metadata| metadata.permissions().readonly())
        .context("metadata")?;
    if readonly {
        log::warn!(
            "Default data directory (read-only): {dir_path}",
            dir_path = dir_path.display()
        );
    }
    Ok(dir_path)
}

/// CLI arguments for headless operation.
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, Default, Subcommand)]
enum CliCommand {
    /// Open the interactive UI (default).
    #[default]
    Ui,
    /// Search files.
    ///
    /// Reads a JSON track search query and prints the file paths line by line to stdout.
    SearchTracks(CliSearchTracksOptions),
}

#[derive(Debug, Default, Parser)]
struct CliSearchTracksOptions {
    /// Override the root URL.
    ///
    /// Must be a valid base URL.
    #[arg(long)]
    override_root_url: Option<Url>,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum CliSearchOutputMode {
    /// Prints relative, platform-independent file paths with slashes as separator.
    #[default]
    RelativeSlash,
    /// Prints relative, platform-dependent file paths.
    Relative,
    /// Prints absolute, platform-dependent file paths.
    Absolute,
}

#[tokio::main]
#[expect(clippy::too_many_lines, reason = "TODO: Extract CLI code.")]
async fn main() {
    env_logger::Builder::new()
        .filter_level(DEFAULT_LOG_FILTER_LEVEL)
        // Parse environment variables after configuring all default option(s).
        .parse_default_env()
        .init();

    let Some(config_dir) = app_config_dir() else {
        log::error!("Config directory is unavailable");
        return;
    };
    debug_assert!(config_dir.exists());
    match config_dir
        .metadata()
        .map(|metadata| metadata.permissions().readonly())
    {
        Ok(readonly) => {
            if readonly {
                log::warn!(
                    "Config directory (read-only): {dir_path}",
                    dir_path = config_dir.display()
                );
            } else {
                log::info!(
                    "Config directory: {dir_path}",
                    dir_path = config_dir.display()
                );
            }
        }
        Err(err) => {
            log::error!("Failed to query permissions of config directory: {err}");
        }
    }

    let aoide_initial_settings =
        match aoide::desktop_app::settings::State::restore(&config_dir, default_data_dir) {
            Ok(settings) => settings,
            Err(err) => {
                log::error!("Failed to restore aoide settings: {err}");
                return;
            }
        };
    let aoide_db_config = match aoide_initial_settings.create_database_config() {
        Ok(db_config) => db_config,
        Err(err) => {
            log::error!("Failed to create aoide database config: {err}");
            return;
        }
    };
    log::debug!("Commissioning aoide library backend: {aoide_db_config:?}");
    let aoide_env = match aoide::desktop_app::Environment::commission(&aoide_db_config) {
        Ok(library_backend) => library_backend,
        Err(err) => {
            log::error!("Failed to commission aoide library backend: {err}");
            return;
        }
    };
    let library = Library::new(aoide_env, aoide_initial_settings);

    let rt = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(err) => {
            log::error!("No Tokio runtime: {err}");
            return;
        }
    };

    let Cli { command } = Cli::parse();
    let command = command.unwrap_or_default();
    match command {
        CliCommand::Ui => {
            // Continue below.
        }
        CliCommand::SearchTracks(CliSearchTracksOptions { override_root_url }) => {
            // Run headless.
            library.spawn_background_tasks(&rt, config_dir);
            let search_params =
                match serde_json::from_reader::<_, aoide::api_json::track::search::SearchParams>(
                    io::stdin(),
                ) {
                    Ok(search_params) => search_params,
                    Err(err) => {
                        log::error!("Failed to read search params from stdin: {err:#}");
                        return;
                    }
                };
            let search_filter = search_params.filter.map(Into::into);
            let search_order = search_params
                .order
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();
            let resolve_url_from_content_path = override_root_url.map_or(
                ResolveUrlFromContentPath::CanonicalRootUrl,
                |override_root_url| {
                    BaseUrl::try_from(override_root_url.clone()).map_or_else(
                        |err| {
                            log::warn!(
                                "Cannot override root URL with \"{override_root_url}\": {err:#}"
                            );
                            ResolveUrlFromContentPath::CanonicalRootUrl
                        },
                        |root_url| ResolveUrlFromContentPath::OverrideRootUrl { root_url },
                    )
                },
            );
            let search_params = aoide::api::track::search::Params {
                resolve_url_from_content_path: Some(resolve_url_from_content_path),
                filter: search_filter,
                order: search_order,
            };
            log::info!("Searching library for tracks");
            let _ = library.search_tracks_with_params(search_params);
            let mut offset = 0;
            let mut subscriber = library.subscribe_track_search_state_changed();
            while let Ok(()) = subscriber.changed().await {
                {
                    let state = subscriber.read_ack();
                    if state.is_pending() {
                        log::debug!("Still pending...");
                        continue;
                    }
                    if let Some(err) = state.last_fetch_error() {
                        log::warn!("Fetching search results failed: {err:#}");
                        break;
                    }
                    let Some(fetched_tracks) = state.fetched_entities() else {
                        log::debug!("Not fetched yet...");
                        continue;
                    };
                    debug_assert!(fetched_tracks.len() >= offset);
                    let fetched_next_len = fetched_tracks.len() - offset;
                    log::info!("Fetched {fetched_next_len} track(s)");
                    for track_entity in &fetched_tracks[offset..] {
                        debug_assert!(track_entity.entity.body.content_url.is_some());
                        let Some(url) = &track_entity.entity.body.content_url else {
                            continue;
                        };
                        let Ok(file_path) = url.to_file_path() else {
                            log::warn!("Skipping track with URL <{url}>");
                            continue;
                        };
                        println!("{}", file_path.display());
                    }
                    offset = fetched_tracks.len();
                    if !matches!(state.can_fetch_more(), Some(true)) {
                        // Finished.
                        break;
                    }
                    // Implicitly drop the state lock when leaving this scope to to avoid a deadlock.
                }
                log::info!("Fetching more tracks");
                let _ = library.fetch_more_track_search_results(&rt);
            }
            return;
        }
    }

    eframe::run_native(
        app_name(),
        eframe::NativeOptions::default(),
        Box::new(move |ctx| {
            let mdl = app::Model::new(library);
            let app = App::new(ctx, rt, mdl, config_dir);
            Ok(Box::new(app))
        }),
    )
    .unwrap();
}

#[must_use]
const fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[must_use]
fn app_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", app_name())
}

fn init_app_dir(app_dir: &Path) {
    if let Err(err) = std::fs::create_dir_all(app_dir) {
        log::error!(
            "Failed to create app directory '{dir}': {err}",
            dir = app_dir.display(),
        );
    } else {
        debug_assert!(app_dir.exists());
    }
}

#[must_use]
fn init_config_dir(app_dirs: &ProjectDirs) -> &Path {
    let app_config_dir = app_dirs.config_local_dir();
    init_app_dir(app_config_dir);
    app_config_dir
}

#[must_use]
fn init_data_dir(app_dirs: &ProjectDirs) -> &Path {
    let app_data_dir = app_dirs.data_local_dir();
    init_app_dir(app_data_dir);
    app_data_dir
}

#[must_use]
fn app_config_dir() -> Option<PathBuf> {
    app_dirs()
        .as_ref()
        .map(init_config_dir)
        .map(Path::to_path_buf)
}

#[must_use]
fn app_data_dir() -> Option<PathBuf> {
    app_dirs()
        .as_ref()
        .map(init_data_dir)
        .map(Path::to_path_buf)
}
