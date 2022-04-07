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

use tokio::sync::{mpsc, watch};

use crate::{
    config::Config,
    runtime::{run, Command as RuntimeCommand, State as RuntimeState},
};

#[cfg(feature = "launcher-ui")]
pub mod ui;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Idle,
    Running(RuntimeState),
    Terminated,
}

#[derive(Debug)]
enum InternalState {
    Idle,
    Running {
        config: Config,

        current_state_rx: watch::Receiver<State>,
        runtime_command_tx: mpsc::UnboundedSender<RuntimeCommand>,

        // A reference to the Tokio runtime is kept to allow scheduling
        // asynchronous worker tasks while running. Currently this ability
        // is neither used nor needed.
        _tokio_runtime: Arc<tokio::runtime::Runtime>,
    },
}

#[derive(Debug)]
pub struct Launcher {
    state: InternalState,
}

impl Launcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: InternalState::Idle,
        }
    }

    #[must_use]
    pub fn state(&self) -> State {
        match &self.state {
            InternalState::Idle => State::Idle,
            InternalState::Running {
                current_state_rx, ..
            } => *current_state_rx.borrow(),
        }
    }

    #[must_use]
    pub fn config(&self) -> Option<&Config> {
        if let InternalState::Running { config, .. } = &self.state {
            Some(config)
        } else {
            None
        }
    }

    pub fn launch_runtime(
        &mut self,
        config: Config,
        mut on_state_changed: impl FnMut(State) + Send + 'static,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        if !matches!(self.state(), State::Idle) {
            anyhow::bail!("Invalid state: {:?}", self.state());
        }

        let tokio_runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?,
        );

        let (current_state_tx, current_state_rx) = watch::channel(State::Idle);
        tokio_runtime.spawn({
            let mut current_state_rx = current_state_rx.clone();
            async move {
                while current_state_rx.changed().await.is_ok() {
                    let state = *current_state_rx.borrow();
                    on_state_changed(state);
                }
                // Channel closed
                log::debug!("Sender of state changes has been dropped");
            }
        });

        let (runtime_command_tx, runtime_command_rx) = mpsc::unbounded_channel();
        let (current_runtime_state_tx, current_runtime_state_rx) = watch::channel(None);
        let join_handle = std::thread::spawn({
            let config = config.clone();
            // TODO: If the Tokio runtime is only accessed within this thread
            // then wrapping into an Arc and cloning would not be needed.
            let tokio_runtime = Arc::clone(&tokio_runtime);
            move || {
                tokio_runtime.block_on(run(config, runtime_command_rx, current_runtime_state_tx))
            }
        });

        tokio_runtime.spawn({
            debug_assert!(matches!(*current_state_rx.borrow(), State::Idle));
            let mut current_runtime_state_rx = current_runtime_state_rx;
            async move {
                let send_state_changed = |state| {
                    if let Err(watch::error::SendError(state)) = current_state_tx.send(state) {
                        // Channel closed
                        log::debug!(
                            "All receivers of state changes have been dropped: {:?}",
                            state
                        );
                    }
                };
                while current_runtime_state_rx.changed().await.is_ok() {
                    if let Some(runtime_state) = *current_runtime_state_rx.borrow() {
                        send_state_changed(State::Running(runtime_state));
                    }
                }
                // Channel closed
                log::debug!("Channel is closed after runtime has been terminated");
                send_state_changed(State::Terminated);
            }
        });

        self.state = InternalState::Running {
            config,
            current_state_rx,
            runtime_command_tx,
            _tokio_runtime: tokio_runtime,
        };

        Ok(join_handle)
    }

    pub fn terminate_runtime(&mut self, abort_pending_tasks: bool) -> anyhow::Result<()> {
        match &mut self.state {
            InternalState::Idle => anyhow::bail!("Invalid state: {:?}", self.state()),
            InternalState::Running {
                runtime_command_tx, ..
            } => {
                let command = RuntimeCommand::Terminate {
                    abort_pending_tasks,
                };
                if let Err(command) = runtime_command_tx.send(command) {
                    anyhow::bail!(
                        "Failed to send command {:?} in state {:?}",
                        command,
                        self.state()
                    );
                }
            }
        }
        Ok(())
    }

    #[cfg_attr(not(feature = "launcher-ui"), allow(dead_code))]
    pub fn reset_after_terminated(&mut self) {
        debug_assert!(matches!(self.state(), State::Terminated));
        self.state = InternalState::Idle;
    }
}
