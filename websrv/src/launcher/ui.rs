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

use std::{sync::Arc, thread::JoinHandle};

use eframe::epi::{egui::CtxRef, Frame};
use parking_lot::Mutex;

use crate::{
    launcher::{Launcher, State},
    runtime::State as RuntimeState,
};

pub struct App {
    launcher: Arc<Mutex<Launcher>>,
    runtime_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl App {
    pub const fn new(launcher: Arc<Mutex<Launcher>>) -> Self {
        Self {
            launcher,
            runtime_thread: None,
        }
    }
}

impl eframe::epi::App for App {
    fn name(&self) -> &str {
        env!("CARGO_PKG_NAME")
    }

    fn on_exit(&mut self) {
        let App {
            launcher,
            runtime_thread,
        } = self;
        let mut launcher = launcher.lock();
        if let Some(join_handle) = runtime_thread.take() {
            match launcher.state() {
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
    }

    fn update(&mut self, ctx: &CtxRef, frame: &Frame) {
        let App {
            launcher,
            runtime_thread,
        } = self;
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
