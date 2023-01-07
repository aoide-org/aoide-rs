// aoide.org - Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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

use eframe::{egui::Context, Frame};
use egui::{Button, CentralPanel, TextEdit, TopBottomPanel};
use parking_lot::Mutex;
use rfd::FileDialog;

use aoide_storage_sqlite::connection::Storage as SqliteDatabaseStorage;

use crate::{
    app_dirs, join_runtime_thread, launcher::State as LauncherState,
    runtime::State as RuntimeState, save_app_config, LauncherMutex,
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
    sqlite_storage: String,
}

impl From<crate::config::DatabaseConfig> for DatabaseConfig {
    fn from(from: crate::config::DatabaseConfig) -> Self {
        let sqlite_storage = from.connection.storage.to_string();
        Self { sqlite_storage }
    }
}

impl TryFrom<DatabaseConfig> for SqliteDatabaseStorage {
    type Error = anyhow::Error;

    fn try_from(from: DatabaseConfig) -> anyhow::Result<Self> {
        let DatabaseConfig { sqlite_storage } = from;
        let sqlite_storage = sqlite_storage.parse()?;
        Ok(sqlite_storage)
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
pub(crate) struct App {
    launcher: Arc<LauncherMutex>,
    exit_flag: Arc<AtomicBool>,
    last_config: crate::config::Config,
    state: State,
    config: Config,
    last_error: Arc<Mutex<Option<String>>>,
}

#[derive(Debug)]
enum State {
    Setup,
    Idle,
    Running {
        runtime_thread: JoinHandle<anyhow::Result<()>>,
    },
    Terminated,
}

impl App {
    pub(crate) fn new(launcher: Arc<LauncherMutex>, config: crate::config::Config) -> Self {
        let last_config = config.clone();
        Self {
            launcher,
            exit_flag: Arc::new(AtomicBool::new(false)),
            last_config,
            state: State::Setup,
            config: config.into(),
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    fn resync_state_on_update(&mut self, ctx: &egui::Context) {
        let launcher = self.launcher.lock();
        let launcher_state = launcher.state();
        if matches!(self.state, State::Terminated) {
            if matches!(launcher_state, LauncherState::Idle) {
                // Resync the app state with the launcher state. This is required,
                // because the launcher state is reset in a detached thread that
                // joins the runtime thread and then finally triggers a repaint.
                self.state = State::Idle;
            }
        } else if matches!(launcher_state, LauncherState::Terminating) {
            // If startup fails the launcher may terminate itself unattended
            drop(launcher);
            self.on_launcher_terminating(ctx);
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
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_enabled(
                editing_enabled,
                TextEdit::singleline(&mut self.config.database.sqlite_storage).hint_text(format!(
                    ".sqlite file or {}",
                    SqliteDatabaseStorage::InMemory
                )),
            );
            if ui
                .add_enabled(editing_enabled, Button::new("Select..."))
                .clicked()
            {
                let mut file_dialog = FileDialog::new()
                    .set_title("Select SQLite database file")
                    .add_filter("SQLite files", &["sqlite"])
                    .add_filter("All files", &["*"]);
                if let Ok(SqliteDatabaseStorage::File { path: file_path }) =
                    self.config.database.sqlite_storage.parse()
                {
                    if is_existing_file(&file_path) {
                        if let Some(file_name) = file_path.file_name().and_then(OsStr::to_str) {
                            file_dialog = file_dialog.set_file_name(file_name);
                        }
                        if let Some(parent_path) = file_path.parent() {
                            file_dialog = file_dialog.set_directory(parent_path);
                        }
                    } else if is_existing_directory(&file_path) {
                        file_dialog = file_dialog.set_directory(file_path.display().to_string());
                    }
                }
                if let Some(file_name) = file_dialog.pick_file() {
                    self.config.database.sqlite_storage = file_name.display().to_string();
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

    fn show_launch_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let launcher_state = self.launcher.lock().state();
            let stop_button_text = match launcher_state {
                LauncherState::Running(RuntimeState::Stopping | RuntimeState::Terminating) => {
                    "Stopping..."
                }
                _ => "Stop",
            };
            let stop_button_enabled = matches!(self.state, State::Running { .. })
                && matches!(
                    launcher_state,
                    LauncherState::Running(
                        RuntimeState::Launching
                            | RuntimeState::Starting
                            | RuntimeState::Listening { .. }
                    )
                );
            if ui
                .add_enabled(stop_button_enabled, Button::new(stop_button_text))
                .clicked()
            {
                self.on_stop(ctx, true);
            }

            let start_button_text = match launcher_state {
                LauncherState::Running(RuntimeState::Launching | RuntimeState::Starting) => {
                    "Starting..."
                }
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

    fn on_start(&mut self, ctx: &egui::Context) {
        debug_assert!(matches!(self.state, State::Idle));
        let Config {
            network: network_config,
            database: database_config,
        } = self.config.clone();
        let mut next_config = self.last_config.clone();
        if let Ok(network_config) = network_config.try_into() {
            next_config.network = network_config;
        }
        if let Ok(storage) = database_config.try_into() {
            next_config.database.connection.storage = storage;
        }
        let mut launcher = self.launcher.lock();
        *self.last_error.lock() = None;
        match launcher.launch_runtime(next_config.clone(), {
            let ctx = ctx.clone();
            move |state| {
                log::debug!("Launcher state changed: {state:?}");
                ctx.request_repaint();
            }
        }) {
            Ok(runtime_thread) => {
                debug_assert_eq!(launcher.config(), Some(&next_config));
                drop(launcher);
                self.config = next_config.clone().into();
                self.last_config = next_config;
                self.state = State::Running { runtime_thread };
                ctx.request_repaint();
            }
            Err(err) => {
                log::warn!("Failed to launch runtime: {err}");
            }
        }
    }

    fn on_stop(&mut self, ctx: &egui::Context, abort_pending_tasks: bool) {
        debug_assert!(matches!(self.state, State::Running { .. }));
        if let Err(err) = self.launcher.lock().terminate_runtime(abort_pending_tasks) {
            log::error!("Failed to terminate runtime: {err}");
            return;
        }
        self.on_launcher_terminating(ctx);
    }

    fn on_launcher_terminating(&mut self, ctx: &egui::Context) {
        if let State::Running { runtime_thread } =
            std::mem::replace(&mut self.state, State::Terminated)
        {
            // Join runtime thread in a detached thread
            std::thread::spawn({
                let launcher = Arc::clone(&self.launcher);
                let last_error = Arc::clone(&self.last_error);
                let ctx = ctx.clone();
                move || {
                    *last_error.lock() = join_runtime_thread(runtime_thread)
                        .err()
                        .map(|err| err.to_string());
                    ctx.request_repaint();
                    while !matches!(launcher.lock().state(), LauncherState::Terminating) {
                        log::debug!("Awaiting termination of launcher...");
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    log::debug!("Launcher is terminating");
                    launcher.lock().reset_on_termination();
                    // The application state will be re-synchronized during
                    // the next invocation of update().
                    ctx.request_repaint();
                }
            });
            ctx.request_repaint();
        }
    }
}

fn is_existing_directory(path: &Path) -> bool {
    path.canonicalize().map_or(false, |p| p.is_dir())
}

fn is_existing_file(path: &Path) -> bool {
    path.canonicalize().map_or(false, |p| p.is_file())
}

impl eframe::App for App {
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
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
                | LauncherState::Terminating => (),
                LauncherState::Running(_) => {
                    if let Err(err) = launcher.terminate_runtime(true) {
                        log::error!("Failed to terminate runtime on exit: {err}");
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

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if matches!(self.state, State::Setup) {
            log::info!("Registering signal handler for Ctrl-C");
            if let Err(err) = ctrlc::set_handler({
                let ctx = ctx.clone();
                let exit_flag = Arc::clone(&self.exit_flag);
                move || {
                    exit_flag.store(true, Ordering::Release);
                    ctx.request_repaint();
                }
            }) {
                log::error!("Failed to register signal handler for Ctrl-C: {err}");
                self.exit_flag.store(true, Ordering::Release);
            }
            // The transition from Setup to Idle must only occur once!
            self.state = State::Idle;
            ctx.request_repaint();
        }
        self.resync_state_on_update(ctx);
        if self.exit_flag.load(Ordering::Acquire) {
            frame.close();
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
                self.show_launch_controls(ui, ctx);
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
