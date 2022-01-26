// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#![warn(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use std::{
    env::current_exe,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use directories::ProjectDirs;
use parking_lot::Mutex;

use crate::{config::Config, launcher::Launcher};

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
                log::warn!("Failed to parse configuration data: {}", err);
            })
            .unwrap_or_default(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Default::default(),
        Err(err) => {
            log::warn!("Failed to read configuration data from file: {}", err);
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
        log::warn!("Failed to store configuration data: {}", err);
        return;
    }
    if let Some(parent_path) = file_path.parent() {
        if let Err(err) = fs::create_dir_all(&parent_path) {
            log::warn!(
                "Failed to create parent directories for configuration file: {}",
                err
            );
        }
    }
    if let Err(err) = fs::write(&file_path, &bytes) {
        log::warn!("Failed to write configuration data into file: {}", err);
    }
}

fn main() {
    env::init_environment();

    if let Err(err) = env::init_tracing_and_logging() {
        eprintln!("Failed to initialize tracing and logging: {}", err);
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
    log::debug!("Initial configuration: {:?}", initial_config);

    // Override config with environment variables
    log::info!("Patching configuration from .env file and environment variables");
    let mut config = initial_config.clone();
    env::parse_config_into(&mut config);
    let save_config_on_exit;
    if config != initial_config {
        log::debug!("Patched configuration: {:?}", config);
        save_config_on_exit = false;
    } else {
        save_config_on_exit = true;
    }

    let launcher = Arc::new(Mutex::new(Launcher::new(config)));

    #[cfg(feature = "with-launcher-ui")]
    if !env::parse_launch_headless().unwrap_or(false) {
        let app = launcher::ui::App::new(Arc::clone(&launcher));
        let options = eframe::NativeOptions::default();
        log::info!("Running launcher UI");
        eframe::run_native(Box::new(app), options);
        // Never returns
    }

    // This code only runs when the launcher UI is disabled
    run_headless(launcher, save_config_on_exit);
}

fn run_headless(launcher: Arc<Mutex<Launcher>>, save_config_on_exit: bool) {
    log::info!("Running headless");

    if let Err(err) = ctrlc::set_handler({
        let launcher = Arc::clone(&launcher);
        move || {
            if let Err(err) = launcher.lock().terminate_runtime(true) {
                log::error!("Failed to terminate runtime: {}", err);
            }
        }
    }) {
        log::error!("Failed to register signal handler: {}", err);
    }

    let runtime_thread = match launcher.lock().launch_runtime() {
        Ok(join_handle) => join_handle,
        Err(err) => {
            log::error!("Failed to launch runtime: {}", err);
            return;
        }
    };

    log::info!("Suspending main thread");
    if let Err(err) = runtime_thread.join() {
        log::error!("Failed to join runtime thread: {:?}", err);
    }
    log::info!("Resuming main thread");

    if save_config_on_exit {
        if let Some(app_dirs) = app_dirs() {
            save_app_config(&app_dirs, launcher.lock().config());
        }
    } else {
        log::info!("Discarding current configuration");
    }

    log::info!("Exiting");
}
