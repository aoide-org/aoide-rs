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

use tokio::sync::mpsc;

use crate::{
    config::Config,
    runtime::{
        run, Command as RuntimeCommand, CurrentState as CurrentRuntimeState, State as RuntimeState,
    },
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
        current_runtime_state: Arc<CurrentRuntimeState>,
        runtime_command_tx: mpsc::UnboundedSender<RuntimeCommand>,
    },
}

impl From<&InternalState> for State {
    fn from(from: &InternalState) -> Self {
        use InternalState::*;
        match from {
            Idle => Self::Idle,
            Running {
                current_runtime_state,
                ..
            } => Self::Running(current_runtime_state.load()),
        }
    }
}

pub struct Launcher {
    state: InternalState,
    config: Config,
}

impl Launcher {
    pub const fn new(config: Config) -> Self {
        Self {
            state: InternalState::Idle,
            config,
        }
    }

    pub fn state(&self) -> State {
        (&self.state).into()
    }

    pub fn launch_runtime(&mut self) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        if !matches!(self.state(), State::Idle) {
            anyhow::bail!("Invalid state: {:?}", self.state());
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let current_runtime_state = Arc::new(CurrentRuntimeState::new(RuntimeState::Launching));
        let (runtime_command_tx, runtime_command_rx) = mpsc::unbounded_channel();

        let join_handle = std::thread::spawn({
            let current_state = Arc::clone(&current_runtime_state);
            let config = self.config.clone();
            move || runtime.block_on(run(config, runtime_command_rx, current_state))
        });

        self.state = InternalState::Running {
            current_runtime_state,
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
}
