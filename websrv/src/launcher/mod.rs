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

#[cfg(feature = "with-launcher-ui")]
pub mod ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Running(RuntimeState),
}

#[derive(Debug)]
enum InternalState {
    Idle,
    Running {
        tokio_runtime: Arc<tokio::runtime::Runtime>,
        current_state_rx: watch::Receiver<State>,
        runtime_command_tx: mpsc::UnboundedSender<RuntimeCommand>,
    },
}

impl From<&InternalState> for State {
    fn from(from: &InternalState) -> Self {
        use InternalState::*;
        match from {
            Idle => Self::Idle,
            Running {
                current_state_rx, ..
            } => current_state_rx.borrow().to_owned(),
        }
    }
}

pub struct Launcher {
    state: InternalState,
    config: Config,
}

impl Launcher {
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self {
            state: InternalState::Idle,
            config,
        }
    }

    #[must_use]
    pub fn state(&self) -> State {
        (&self.state).into()
    }

    pub fn on_state_changed_while_running(
        &self,
        mut on_state_changed: impl FnMut(State) + Send + 'static,
    ) -> anyhow::Result<()> {
        use InternalState::*;
        match &self.state {
            Running {
                tokio_runtime,
                current_state_rx,
                ..
            } => {
                let mut current_state_rx = current_state_rx.to_owned();
                tokio_runtime.spawn(async move {
                    while current_state_rx.changed().await.is_ok() {
                        let state = current_state_rx.borrow().to_owned();
                        on_state_changed(state);
                    }
                    // Channel closed
                    log::debug!("Sender of state changes has been dropped");
                });
                Ok(())
            }
            state => {
                anyhow::bail!("Not running: {:?}", state);
            }
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[allow(dead_code)]
    pub fn decommission(self) -> Result<Config, Self> {
        if matches!(self.state(), State::Idle) {
            let Self { config, .. } = self;
            Ok(config)
        } else {
            Err(self)
        }
    }

    pub fn launch_runtime(&mut self) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        if !matches!(self.state(), State::Idle) {
            anyhow::bail!("Invalid state: {:?}", self.state());
        }

        let tokio_runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?,
        );
        let (runtime_command_tx, runtime_command_rx) = mpsc::unbounded_channel();

        let (current_runtime_state_tx, current_runtime_state_rx) = watch::channel(None);
        let join_handle = std::thread::spawn({
            let config = self.config.clone();
            let tokio_runtime = Arc::clone(&tokio_runtime);
            move || {
                tokio_runtime.block_on(run(config, runtime_command_rx, current_runtime_state_tx))
            }
        });

        let (current_state_tx, current_state_rx) = watch::channel(State::Idle);
        tokio_runtime.spawn({
            let mut current_runtime_state_rx = current_runtime_state_rx;
            async move {
                while current_runtime_state_rx.changed().await.is_ok() {
                    if let Some(runtime_state) = current_runtime_state_rx.borrow().to_owned() {
                        if current_state_tx
                            .send(State::Running(runtime_state))
                            .is_err()
                        {
                            // Channel closed
                            log::debug!("All receivers of state changes have been dropped");
                            return;
                        }
                    }
                }
                // Channel closed
                log::debug!("Sender of runtime state changes has been dropped");
            }
        });

        self.state = InternalState::Running {
            tokio_runtime,
            current_state_rx,
            runtime_command_tx,
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

    pub fn reset_after_terminated(&mut self) {
        debug_assert_eq!(State::Running(RuntimeState::Terminating), self.state());
        self.state = InternalState::Idle;
    }
}
