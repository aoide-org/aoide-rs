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

use std::{
    ffi::OsStr,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
};

use eframe::epi::{egui::CtxRef, Frame};
use egui::{Button, CentralPanel, TextEdit, TopBottomPanel};
use parking_lot::Mutex;
use rfd::FileDialog;

use crate::{
    app_dirs, app_name,
    config::{SqliteDatabaseConnection, SQLITE_DATABASE_CONNECTION_IN_MEMORY},
    join_runtime_thread,
    launcher::State as LauncherState,
    runtime::State as RuntimeState,
    save_app_config, LauncherMutex,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct NetworkConfig {
    endpoint: EndpointConfig,
}

impl From<crate::config::NetworkConfig> for NetworkConfig {
    fn from(from: crate::config::NetworkConfig) -> Self {
        let crate::config::NetworkConfig { endpoint } = from;
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl TryFrom<NetworkConfig> for crate::config::NetworkConfig {
    type Error = anyhow::Error;

    fn try_from(from: NetworkConfig) -> anyhow::Result<Self> {
        let NetworkConfig { endpoint } = from;
        let endpoint = endpoint.try_into()?;
        Ok(Self { endpoint })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EndpointConfig {
    ip_addr: String,
    port: String,
}

impl From<crate::config::EndpointConfig> for EndpointConfig {
    fn from(from: crate::config::EndpointConfig) -> Self {
        let crate::config::EndpointConfig { ip_addr, port } = from;
        Self {
            ip_addr: ip_addr.to_string(),
            port: port.to_string(),
        }
    }
}

impl TryFrom<EndpointConfig> for crate::config::EndpointConfig {
    type Error = anyhow::Error;

    fn try_from(from: EndpointConfig) -> anyhow::Result<Self> {
        let EndpointConfig { ip_addr, port } = from;
        let ip_addr = ip_addr.trim().parse()?;
        let port = port.trim().parse()?;
        Ok(Self { ip_addr, port })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DatabaseConfig {
    sqlite_connection: String,
}

impl From<crate::config::DatabaseConfig> for DatabaseConfig {
    fn from(from: crate::config::DatabaseConfig) -> Self {
        let crate::config::DatabaseConfig { connection, .. } = from;
        let sqlite_connection = match connection {
            crate::config::DatabaseConnection::Sqlite(sqlite) => sqlite.to_string(),
        };
        Self { sqlite_connection }
    }
}

impl TryFrom<DatabaseConfig> for crate::config::DatabaseConnection {
    type Error = anyhow::Error;

    fn try_from(from: DatabaseConfig) -> anyhow::Result<Self> {
        let DatabaseConfig { sqlite_connection } = from;
        let sqlite_connection = sqlite_connection.parse()?;
        Ok(Self::Sqlite(sqlite_connection))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    network: NetworkConfig,
    database: DatabaseConfig,
}

impl From<crate::config::Config> for Config {
    fn from(from: crate::config::Config) -> Self {
        let crate::config::Config { network, database } = from;
        Self {
            network: network.into(),
            database: database.into(),
        }
    }
}

#[derive(Debug)]
pub struct App {
    launcher: Arc<LauncherMutex>,
    exit_flag: Arc<AtomicBool>,
    last_config: crate::config::Config,
    state: State,
    config: Config,
    last_error: Arc<Mutex<Option<String>>>,
}

#[derive(Debug)]
enum State {
    Idle,
    Running {
        runtime_thread: JoinHandle<anyhow::Result<()>>,
    },
    Terminated,
}

impl App {
    pub fn new(launcher: Arc<LauncherMutex>, config: crate::config::Config) -> Self {
        let last_config = config.clone();
        Self {
            launcher,
            exit_flag: Arc::new(AtomicBool::new(false)),
            last_config,
            state: State::Idle,
            config: config.into(),
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    fn resync_state_on_update(&mut self, ctx: &CtxRef) {
        let launcher = self.launcher.lock();
        let launcher_state = launcher.state();
        if matches!(self.state, State::Terminated) {
            if matches!(launcher_state, LauncherState::Idle) {
                // Resync the app state with the launcher state. This is required,
                // because the launcher state is reset in a detached thread that
                // joins the runtime thread and then finally triggers a repaint.
                self.state = State::Idle;
            }
        } else if matches!(launcher_state, LauncherState::Terminated) {
            // If startup fails the launcher may terminate itself unattended
            drop(launcher);
            self.after_launcher_terminated(ctx);
        }
    }

    fn show_config_grid(&mut self, ui: &mut egui::Ui) {
        let editing_enabled = matches!(self.state, State::Idle);

        ui.label("Network IP:");
        ui.add_enabled(
            editing_enabled,
            TextEdit::singleline(&mut self.config.network.endpoint.ip_addr)
                .hint_text("IPv6/IPv4 address, e.g \"::\" (IPv6) or \"127.0.0.1\" (IPv4))"),
        );
        ui.end_row();

        ui.label("Network port:");
        ui.add_enabled(
            editing_enabled,
            TextEdit::singleline(&mut self.config.network.endpoint.port)
                .hint_text("port number, e.g. 8080 or 0 (ephemeral port)"),
        );
        ui.end_row();

        ui.label("SQLite database:");
        ui.with_layout(egui::Layout::left_to_right(), |ui| {
            ui.add_enabled(
                editing_enabled,
                TextEdit::singleline(&mut self.config.database.sqlite_connection).hint_text(
                    format!(".sqlite file or {}", SQLITE_DATABASE_CONNECTION_IN_MEMORY),
                ),
            );
            if ui
                .add_enabled(editing_enabled, Button::new("Select..."))
                .clicked()
            {
                let mut file_dialog = FileDialog::new()
                    .set_title("Select SQLite database file")
                    .add_filter("SQLite files", &["sqlite"])
                    .add_filter("All files", &["*"]);
                if let Ok(SqliteDatabaseConnection::File { path: file_path }) =
                    self.config.database.sqlite_connection.parse()
                {
                    if is_existing_file(&file_path) {
                        if let Some(file_name) = file_path.file_name().and_then(OsStr::to_str) {
                            file_dialog = file_dialog.set_file_name(file_name);
                        }
                        if let Some(parent_path) = file_path.parent() {
                            file_dialog = file_dialog.set_directory(parent_path);
                        }
                    } else if is_existing_directory(&file_path) {
                        file_dialog = file_dialog.set_directory(&file_path.display().to_string());
                    }
                }
                if let Some(file_name) = file_dialog.pick_file() {
                    self.config.database.sqlite_connection = file_name.display().to_string();
                }
            }
        });
        ui.end_row();

        if ui
            .add_enabled(editing_enabled, Button::new("Reset to defaults"))
            .clicked()
        {
            self.config = crate::config::Config::default().into();
        }
        ui.end_row();
    }

    fn show_launch_controls(&mut self, ctx: &CtxRef, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(), |ui| {
            let launcher_state = self.launcher.lock().state();
            let stop_button_text = match launcher_state {
                LauncherState::Running(RuntimeState::Stopping)
                | LauncherState::Running(RuntimeState::Terminating) => "Stopping...",
                _ => "Stop",
            };
            let stop_button_enabled = matches!(self.state, State::Running { .. })
                && matches!(
                    launcher_state,
                    LauncherState::Running(RuntimeState::Launching)
                        | LauncherState::Running(RuntimeState::Starting)
                        | LauncherState::Running(RuntimeState::Listening { .. })
                );
            if ui
                .add_enabled(stop_button_enabled, Button::new(stop_button_text))
                .clicked()
            {
                self.on_stop(ctx, true);
            }

            let start_button_text = match launcher_state {
                LauncherState::Running(RuntimeState::Launching)
                | LauncherState::Running(RuntimeState::Starting) => "Starting...",
                _ => "Start",
            };
            let start_button_enabled = matches!(self.state, State::Idle);
            if ui
                .add_enabled(start_button_enabled, Button::new(start_button_text))
                .clicked()
            {
                self.on_start(ctx);
            }
        });
    }

    fn on_start(&mut self, ctx: &CtxRef) {
        debug_assert!(matches!(self.state, State::Idle));
        let Config {
            network: network_config,
            database: database_config,
        } = self.config.clone();
        let mut next_config = self.last_config.to_owned();
        if let Ok(network_config) = network_config.try_into() {
            next_config.network = network_config;
        }
        if let Ok(database_connection) = database_config.try_into() {
            next_config.database.connection = database_connection;
        }
        let mut launcher = self.launcher.lock();
        *self.last_error.lock() = None;
        match launcher.launch_runtime(next_config.clone(), {
            let ctx = ctx.to_owned();
            move |state| {
                log::debug!("Launcher state changed: {:?}", state);
                trigger_repaint(&ctx);
            }
        }) {
            Ok(runtime_thread) => {
                debug_assert_eq!(launcher.config(), Some(&next_config));
                drop(launcher);
                self.config = next_config.clone().into();
                self.last_config = next_config;
                self.state = State::Running { runtime_thread };
                trigger_repaint(ctx);
            }
            Err(err) => {
                log::warn!("Failed to launch runtime: {}", err);
            }
        }
    }

    fn on_stop(&mut self, ctx: &CtxRef, abort_pending_tasks: bool) {
        debug_assert!(matches!(self.state, State::Running { .. }));
        if let Err(err) = self.launcher.lock().terminate_runtime(abort_pending_tasks) {
            log::error!("Failed to terminate runtime: {}", err);
            return;
        }
        self.after_launcher_terminated(ctx);
    }

    fn after_launcher_terminated(&mut self, ctx: &CtxRef) {
        if let State::Running { runtime_thread } =
            std::mem::replace(&mut self.state, State::Terminated)
        {
            // Join runtime thread in a detached thread
            std::thread::spawn({
                let launcher = Arc::clone(&self.launcher);
                let last_error = Arc::clone(&self.last_error);
                let ctx = ctx.to_owned();
                move || {
                    *last_error.lock() = join_runtime_thread(runtime_thread)
                        .err()
                        .map(|err| err.to_string());
                    trigger_repaint(&ctx);
                    while !matches!(launcher.lock().state(), LauncherState::Terminated) {
                        log::debug!("Awaiting termination of launcher...");
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    log::debug!("Launcher terminated");
                    launcher.lock().reset_after_terminated();
                    // The application state will be re-synchronized during
                    // the next invocation of update().
                    trigger_repaint(&ctx);
                }
            });
            trigger_repaint(ctx);
        }
    }
}

fn is_existing_directory(path: &Path) -> bool {
    path.canonicalize().map(|p| p.is_dir()).unwrap_or(false)
}

fn is_existing_file(path: &Path) -> bool {
    path.canonicalize().map(|p| p.is_file()).unwrap_or(false)
}

fn trigger_repaint(ctx: &CtxRef) {
    // Calling request_repaint() doesn't seem to be sufficient sometimes!?
    // Even setting needs_repaint = true doesn't have the desired effect
    // Example: When hitting the Start button and not moving the pointer
    // then the last displayed state will remain Running(Starting) even
    // though Running(Listening) has already been received and a repaint
    // has been triggered.
    log::debug!("Triggering repaint");
    ctx.output().needs_repaint = true;
    ctx.request_repaint();
}

impl eframe::epi::App for App {
    fn name(&self) -> &str {
        app_name()
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        if let Err(err) = ctrlc::set_handler({
            let ctx = ctx.to_owned();
            let exit_flag = Arc::clone(&self.exit_flag);
            move || {
                exit_flag.store(true, Ordering::Release);
                trigger_repaint(&ctx);
            }
        }) {
            log::error!("Failed to register signal handler: {}", err);
        }
    }

    fn on_exit(&mut self) {
        let App {
            launcher,
            last_config,
            state,
            ..
        } = self;
        let state = std::mem::replace(state, State::Idle);
        if let State::Running { runtime_thread } = state {
            let mut launcher = launcher.lock();
            match launcher.state() {
                LauncherState::Idle
                | LauncherState::Running(RuntimeState::Terminating)
                | LauncherState::Terminated => (),
                LauncherState::Running(_) => {
                    if let Err(err) = launcher.terminate_runtime(true) {
                        log::error!("Failed to terminate runtime on exit: {}", err);
                    }
                }
            }
            *self.last_error.lock() = join_runtime_thread(runtime_thread)
                .err()
                .map(|err| err.to_string());
        }
        if let Some(app_dirs) = app_dirs() {
            save_app_config(&app_dirs, last_config);
        }
    }

    fn update(&mut self, ctx: &CtxRef, frame: &Frame) {
        self.resync_state_on_update(ctx);
        if self.exit_flag.load(Ordering::Acquire) {
            frame.quit();
        }
        TopBottomPanel::top("config_panel").show(ctx, |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    self.show_config_grid(ui);
                });
        });
        CentralPanel::default().show(ctx, |_ui| {
            TopBottomPanel::top("launch_controls").show(ctx, |ui| {
                self.show_launch_controls(ctx, ui);
            });
        });
        TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Current state:");
                    ui.label(format!("{:?}", self.launcher.lock().state()));
                    ui.end_row();

                    ui.label("Last error:");
                    let last_error = self.last_error.lock();
                    if let Some(last_error) = last_error.as_deref() {
                        ui.label(last_error);
                    }
                });
        });
    }
}
