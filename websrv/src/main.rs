// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// rustflags
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
// rustflags (clippy)
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
// rustdocflags
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use std::{
    env::current_exe,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    thread::JoinHandle,
};

use directories::ProjectDirs;

use crate::{
    config::Config,
    launcher::{Launcher, State},
    runtime::State as RuntimeState,
};

mod config;
mod env;
mod launcher;
mod routing;
mod runtime;

#[must_use]
pub fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[must_use]
pub fn app_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("org", "aoide", app_name())
}

#[must_use]
fn app_config_dir(app_dirs: &ProjectDirs) -> &Path {
    app_dirs.config_dir()
}

#[must_use]
pub fn new_config_file_path(app_dirs: &ProjectDirs, file_suffix: &str) -> PathBuf {
    let mut path_buf = app_config_dir(app_dirs).to_path_buf();
    path_buf.push("config");
    path_buf.set_extension(file_suffix);
    path_buf
}

#[must_use]
pub fn load_app_config(app_dirs: &ProjectDirs) -> Config {
    let file_path = new_config_file_path(app_dirs, "ron");
    log::info!("Loading configuration from file: {}", file_path.display());
    match fs::read(&file_path) {
        Ok(bytes) => ron::de::from_bytes(&bytes)
            .map_err(|err| {
                log::warn!("Failed to parse configuration data: {err}");
            })
            .unwrap_or_default(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Default::default(),
        Err(err) => {
            log::warn!("Failed to read configuration data from file: {err}");
            Default::default()
        }
    }
}

pub fn save_app_config(app_dirs: &ProjectDirs, config: &Config) {
    let file_path = new_config_file_path(app_dirs, "ron");
    log::info!(
        "Saving current configuration into file: {}",
        file_path.display()
    );
    let mut bytes = vec![];
    if let Err(err) = ron::ser::to_writer_pretty(&mut bytes, &config, Default::default()) {
        log::warn!("Failed to store configuration data: {err}");
        return;
    }
    if let Some(parent_path) = file_path.parent() {
        if let Err(err) = fs::create_dir_all(parent_path) {
            log::warn!("Failed to create parent directories for configuration file: {err}");
        }
    }
    if let Err(err) = fs::write(&file_path, &bytes) {
        log::warn!("Failed to write configuration data into file: {err}");
    }
}

pub fn join_runtime_thread(join_handle: JoinHandle<anyhow::Result<()>>) -> anyhow::Result<()> {
    log::info!("Awaiting termination of runtime thread");
    match join_handle.join() {
        Ok(Ok(())) => {
            log::info!("Runtime thread terminated");
            Ok(())
        }
        Ok(Err(err)) => {
            log::warn!("Runtime thread terminated with error: {err}");
            Err(err)
        }
        Err(err) => {
            let err = anyhow::anyhow!("Failed to await termination of runtime thread: {err:?}");
            log::error!("{err}");
            Err(err)
        }
    }
}

type LauncherMutex = parking_lot::Mutex<Launcher>;

fn main() {
    env::init_environment();

    if let Err(err) = env::init_tracing_and_logging() {
        eprintln!("Failed to initialize tracing and logging: {err}");
        return;
    }

    if let Ok(exe_path) = current_exe() {
        log::info!("Executable: {}", exe_path.display());
    }
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let initial_config: Config = if env::parse_default_config().unwrap_or(false) {
        log::info!("Using initial default configuration");
        Default::default()
    } else {
        app_dirs().as_ref().map(load_app_config).unwrap_or_default()
    };
    log::debug!("Initial configuration: {initial_config:?}");

    // Override config with environment variables
    log::info!("Patching configuration from .env file and environment variables");
    let mut config = initial_config.clone();
    env::parse_config_into(&mut config);
    let save_config_on_exit = if config != initial_config {
        log::debug!("Patched configuration: {config:?}");
        // Don't save on exit if using a temporary configuration
        false
    } else {
        true
    };

    let launcher = Arc::new(LauncherMutex::new(Launcher::new()));

    #[cfg(feature = "launcher-ui")]
    if !env::parse_launch_headless().unwrap_or(false) {
        log::info!("Running launcher UI");
        let config = config.clone();
        eframe::run_native(
            app_name(),
            eframe::NativeOptions::default(),
            Box::new({
                let launcher = Arc::clone(&launcher);
                move |_creation_context| Box::new(launcher::ui::App::new(launcher, config))
            }),
        );
        log::info!("Exiting");
        return;
    }

    run_headless(launcher, config, save_config_on_exit);
    log::info!("Exiting");
}

fn run_headless(launcher: Arc<LauncherMutex>, config: Config, save_config_on_exit: bool) {
    log::info!("Running headless");

    log::info!("Registering signal handler for Ctrl-C");
    if let Err(err) = ctrlc::set_handler({
        let launcher = Arc::clone(&launcher);
        move || {
            if let Err(err) = launcher.lock().terminate_runtime(true) {
                log::error!("Failed to terminate runtime: {err}");
            }
        }
    }) {
        log::error!("Failed to register signal handler: {err}");
    }

    log::info!("Launching runtime");
    let runtime_thread = match launcher.lock().launch_runtime(config, |state| {
        if let State::Running(RuntimeState::Listening { socket_addr }) = state {
            // Publish socket address on stdout
            println!("{socket_addr}");
        }
    }) {
        Ok(join_handle) => join_handle,
        Err(err) => {
            log::error!("Failed to launch runtime: {err}");
            return;
        }
    };

    log::info!("Suspending main thread");
    // This method will log all outcomes
    join_runtime_thread(runtime_thread).ok();
    log::info!("Resuming main thread");

    if save_config_on_exit {
        if let (Some(app_dirs), Some(config)) = (app_dirs(), launcher.lock().config()) {
            save_app_config(&app_dirs, config);
        }
    } else {
        log::info!("Discarding current configuration");
    }

    log::info!("Exiting");
}
