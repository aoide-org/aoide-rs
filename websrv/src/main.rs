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

use std::{env::current_exe, sync::Arc};

use parking_lot::Mutex;

use crate::{config::Config, launcher::Launcher};

mod config;
mod env;
mod launcher;
mod routing;
mod runtime;

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

    let mut config = Config::default();
    env::parse_config_into(&mut config);

    let launcher = Arc::new(Mutex::new(Launcher::new(config)));

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

    #[cfg(feature = "with-launcher-ui")]
    if !env::parse_launch_headless().unwrap_or(false) {
        let app = launcher::ui::App::new(Arc::clone(&launcher));
        let options = eframe::NativeOptions::default();
        log::info!("Running launcher UI");
        eframe::run_native(Box::new(app), options);
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

    log::info!("Exiting");
}
