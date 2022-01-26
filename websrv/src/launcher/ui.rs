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
use egui::{Button, CentralPanel, TopBottomPanel};
use parking_lot::Mutex;

use crate::{
    app_dirs, app_name, join_runtime_thread,
    launcher::{Launcher, State},
    runtime::State as RuntimeState,
    save_app_config,
};

pub struct App {
    launcher: Arc<Mutex<Launcher>>,
    runtime_thread: Option<JoinHandle<anyhow::Result<()>>>,
    exit_flag: Arc<AtomicBool>,
}

impl App {
    pub fn new(launcher: Arc<Mutex<Launcher>>) -> Self {
        Self {
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
        .launch_runtime()
        .map_err(|err| {
            log::error!("Failed to launch runtime: {}", err);
        })
        .ok();
    if runtime_thread.is_some() {
        let ctx = ctx.to_owned();
        launcher
            .on_state_changed_while_running(move |state| {
                log::debug!("State changed: {:?}", state);
                trigger_repaint(&ctx);
            })
            .unwrap();
    }
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
            launcher: shared_launcher,
            runtime_thread,
            exit_flag,
        } = self;
        if exit_flag.load(Ordering::Acquire) {
            frame.quit();
        }
        let mut launcher = shared_launcher.lock();
        TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(format!("{:?}", launcher.state()));
        });
        CentralPanel::default().show(ctx, |ui| {
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
                *runtime_thread = on_start(ctx, &mut launcher);
            }
            let stop_button_text = match launcher.state() {
                State::Running(RuntimeState::Stopping)
                | State::Running(RuntimeState::Terminating) => "Stopping...",
                _ => "Stop",
            };

            let stop_button_enabled = matches!(
                launcher.state(),
                State::Running(RuntimeState::Launching)
                    | State::Running(RuntimeState::Starting)
                    | State::Running(RuntimeState::Listening)
            );
            if ui
                .add_enabled(stop_button_enabled, Button::new(stop_button_text))
                .clicked()
            {
                on_stop(ctx, shared_launcher, &mut launcher, runtime_thread);
            }
        });
    }
}
