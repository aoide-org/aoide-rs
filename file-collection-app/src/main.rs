// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use anyhow::{bail, Context as _};
use directories::ProjectDirs;
use log::LevelFilter;

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

#[tokio::main]
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
    };

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
