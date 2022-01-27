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
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};

use eframe::epi::{egui::CtxRef, Frame};
use egui::{Button, CentralPanel, Label, TextEdit, TopBottomPanel};
use parking_lot::Mutex;

use crate::{
    app_dirs, app_name,
    config::SQLITE_DATABASE_CONNECTION_IN_MEMORY,
    join_runtime_thread,
    launcher::{Launcher, State},
    runtime::State as RuntimeState,
    save_app_config,
};

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

struct EndpointConfig {
    ip_addr: String,
    port: u16,
}

impl From<crate::config::EndpointConfig> for EndpointConfig {
    fn from(from: crate::config::EndpointConfig) -> Self {
        let crate::config::EndpointConfig { ip_addr, port } = from;
        Self {
            ip_addr: ip_addr.to_string(),
            port,
        }
    }
}

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

pub struct App {
    config: Config,
    launcher: Arc<Mutex<Launcher>>,
    runtime_thread: Option<JoinHandle<anyhow::Result<()>>>,
    exit_flag: Arc<AtomicBool>,
}

impl App {
    pub fn new(launcher: Arc<Mutex<Launcher>>) -> Self {
        let config = launcher.lock().config().to_owned().into();
        Self {
            config,
            launcher,
            runtime_thread: None,
            exit_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn trigger_repaint(ctx: &CtxRef) {
    // Calling request_repaint() doesn't seem to be sufficient sometimes!?
    // Even setting needs_repaint = true doesn't have the desired effect
    // Example: When hitting the Start button and not moving the pointer
    // then the last displayed state will remain Running(Starting) even
    // though Running(Listening) has already been received and a repaint
    // has been triggered.
    ctx.output().needs_repaint = true;
    ctx.request_repaint();
}

fn on_start(ctx: &CtxRef, launcher: &mut Launcher) -> Option<JoinHandle<anyhow::Result<()>>> {
    let runtime_thread = launcher
        .launch_runtime({
            let ctx = ctx.to_owned();
            move |state| {
                log::debug!("State changed: {:?}", state);
                trigger_repaint(&ctx);
            }
        })
        .map_err(|err| {
            log::error!("Failed to launch runtime: {}", err);
        })
        .ok();
    runtime_thread
}

fn on_stop(
    ctx: &CtxRef,
    shared_launcher: &Arc<Mutex<Launcher>>,
    launcher: &mut Launcher,
    runtime_thread: &mut Option<JoinHandle<anyhow::Result<()>>>,
) {
    if let Err(err) = launcher.terminate_runtime(true) {
        log::error!("Failed to terminate runtime: {}", err);
        return;
    }
    if let Some(join_handle) = runtime_thread.take() {
        // Join runtime thread in a detached thread
        let launcher = Arc::clone(shared_launcher);
        let ctx = ctx.to_owned();
        std::thread::spawn(move || {
            join_runtime_thread(join_handle);
            let mut launcher = launcher.lock();
            if launcher.state() == State::Running(RuntimeState::Terminating) {
                launcher.reset_after_terminated();
                trigger_repaint(&ctx);
            }
        });
    }
}

fn show_config_grid(ui: &mut egui::Ui, config: &mut Config, launcher: &mut Launcher) {
    let editing_enabled = matches!(launcher.state(), State::Idle);

    ui.label("Network IP:");
    ui.add_enabled(
        editing_enabled,
        TextEdit::singleline(&mut config.network.endpoint.ip_addr).hint_text("IPv4/IPv6 address"),
    );
    ui.end_row();

    ui.label(format!("Network port: {}", config.network.endpoint.port));
    ui.add_enabled(
        editing_enabled,
        // TODO: Make editable, probably use a spin box
        Label::new(config.network.endpoint.port.to_string()),
    );
    ui.end_row();

    ui.label("Database connection:");
    ui.add_enabled(
        editing_enabled,
        TextEdit::singleline(&mut config.database.sqlite_connection).hint_text(format!(
            ".sqlite file or {}",
            SQLITE_DATABASE_CONNECTION_IN_MEMORY
        )),
    );
    ui.end_row();
}

fn show_launch_controls(
    ui: &mut egui::Ui,
    ctx: &CtxRef,
    shared_launcher: &Arc<Mutex<Launcher>>,
    launcher: &mut Launcher,
    runtime_thread: &mut Option<JoinHandle<anyhow::Result<()>>>,
) {
    ui.with_layout(egui::Layout::left_to_right(), |ui| {
        let stop_button_text = match launcher.state() {
            State::Running(RuntimeState::Stopping) | State::Running(RuntimeState::Terminating) => {
                "Stopping..."
            }
            _ => "Stop",
        };
        let stop_button_enabled = matches!(
            launcher.state(),
            State::Running(RuntimeState::Launching)
                | State::Running(RuntimeState::Starting)
                | State::Running(RuntimeState::Listening { .. })
        );
        if ui
            .add_enabled(stop_button_enabled, Button::new(stop_button_text))
            .clicked()
        {
            on_stop(ctx, shared_launcher, launcher, runtime_thread);
        }

        let start_button_text = match launcher.state() {
            State::Running(RuntimeState::Starting) => "Starting...",
            _ => "Start",
        };
        let start_button_enabled =
            matches!(launcher.state(), State::Idle) && runtime_thread.is_none();
        if ui
            .add_enabled(start_button_enabled, Button::new(start_button_text))
            .clicked()
        {
            debug_assert!(runtime_thread.is_none());
            *runtime_thread = on_start(ctx, launcher);
        }
    });
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
            runtime_thread,
            ..
        } = self;
        let mut launcher = launcher.lock();
        if let Some(join_handle) = runtime_thread.take() {
            match launcher.state() {
                State::Idle
                | State::Running(RuntimeState::Stopping)
                | State::Running(RuntimeState::Terminating) => (),
                State::Running(_) => {
                    if let Err(err) = launcher.terminate_runtime(true) {
                        log::error!("Failed to terminate runtime on exit: {}", err);
                    }
                }
            }
            join_runtime_thread(join_handle);
        }
        if let Some(app_dirs) = app_dirs() {
            save_app_config(&app_dirs, launcher.config());
        }
    }

    fn update(&mut self, ctx: &CtxRef, frame: &Frame) {
        let App {
            config,
            launcher: shared_launcher,
            runtime_thread,
            exit_flag,
        } = self;
        if exit_flag.load(Ordering::Acquire) {
            frame.quit();
        }
        let mut launcher = shared_launcher.lock();
        TopBottomPanel::top("config_panel").show(ctx, |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    show_config_grid(ui, config, &mut launcher);
                });
        });
        CentralPanel::default().show(ctx, |_ui| {
            TopBottomPanel::top("launch_controls").show(ctx, |ui| {
                show_launch_controls(ui, ctx, shared_launcher, &mut launcher, runtime_thread);
            });
        });
        TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.label(format!("{:?}", launcher.state()));
        });
    }
}
