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
use parking_lot::Mutex;

use crate::{
    app_dirs, app_name,
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
                // FIXME: How to enforce a repaint of the UI even if it is
                // currently invisible?
                ctx.request_repaint();
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
                State::Running(RuntimeState::Terminating) => (),
                State::Running(_) => {
                    if let Err(err) = launcher.terminate_runtime(true) {
                        log::error!("Failed to terminate runtime on exit: {}", err);
                    }
                }
                state => {
                    log::error!(
                        "Unexpected state {:?} while runtime thread is still running",
                        state
                    );
                }
            }
            log::info!("Awaiting termination of runtime thread");
            match join_handle.join() {
                Ok(Ok(())) => {
                    log::info!("Runtime thread terminated");
                }
                Ok(Err(err)) => {
                    log::warn!("Runtime thread terminated with error: {}", err);
                }
                Err(err) => {
                    log::error!("Failed to await termination of runtime thread: {:?}", err);
                }
            }
        } else {
            match launcher.state() {
                State::Idle => (),
                state => {
                    log::error!("Unexpected state {:?} without a runtime thread", state);
                }
            }
        }
        if let Some(app_dirs) = app_dirs() {
            save_app_config(&app_dirs, launcher.config());
        }
    }

    fn update(&mut self, ctx: &CtxRef, frame: &Frame) {
        let App {
            launcher,
            runtime_thread,
            exit_flag,
        } = self;
        if exit_flag.load(Ordering::Acquire) {
            frame.quit();
        }
        let mut launcher = launcher.lock();
        match launcher.state() {
            State::Idle => {
                // TODO: Launch runtime by user interaction instead of implicitly
                debug_assert!(runtime_thread.is_none());
                if runtime_thread.is_none() {
                    *runtime_thread = launcher
                        .launch_runtime()
                        .map_err(|err| {
                            log::error!("Failed to launch runtime: {}", err);
                        })
                        .ok();
                }
            }
            State::Running(RuntimeState::Terminating) => {
                frame.quit();
            }
            State::Running(_) => (),
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{:?}", launcher.state()));
        });
    }
}
