// aoide.org - Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::thread::JoinHandle;

use tokio::sync::mpsc;

use crate::{
    config::Config,
    runtime::{run, Command as RuntimeCommand, State as RuntimeState},
};

#[cfg(feature = "launcher-ui")]
pub(crate) mod ui;

#[derive(Debug, Clone, Copy)]
pub(crate) enum State {
    Idle,
    Running(RuntimeState),
    Terminating,
}

#[derive(Debug)]
enum InternalState {
    Idle,
    Running {
        config: Config,

        current_state_rx: discro::Subscriber<State>,
        runtime_command_tx: mpsc::UnboundedSender<RuntimeCommand>,

        // Keep the Tokio runtime alive while running.
        runtime: Box<tokio::runtime::Runtime>,
    },
}

#[derive(Debug)]
pub(crate) struct Launcher {
    state: InternalState,
}

impl Launcher {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            state: InternalState::Idle,
        }
    }

    #[must_use]
    pub(crate) fn state(&self) -> State {
        match &self.state {
            InternalState::Idle => State::Idle,
            InternalState::Running {
                current_state_rx, ..
            } => *current_state_rx.read(),
        }
    }

    #[must_use]
    pub(crate) const fn config(&self) -> Option<&Config> {
        match &self.state {
            InternalState::Idle => None,
            InternalState::Running { config, .. } => Some(config),
        }
    }

    #[must_use]
    pub(crate) fn runtime_handle(&self) -> Option<&tokio::runtime::Handle> {
        match &self.state {
            InternalState::Idle => None,
            InternalState::Running { runtime, .. } => Some(runtime.handle()),
        }
    }

    pub(crate) fn launch_runtime(
        &mut self,
        config: Config,
        mut on_state_changed: impl FnMut(State) + Send + 'static,
    ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
        if !matches!(self.state(), State::Idle) {
            anyhow::bail!("invalid state: {:?}", self.state());
        }

        let runtime = tokio::runtime::Runtime::new()?;

        let current_state_tx = discro::Publisher::new(State::Idle);
        let current_state_rx = current_state_tx.subscribe();
        runtime.spawn({
            let mut current_state_rx = current_state_tx.subscribe();
            async move {
                while current_state_rx.changed().await.is_ok() {
                    let state = *current_state_rx.read_ack();
                    on_state_changed(state);
                }
                log::debug!("Stop listening for state changes after launcher has been terminated");
            }
        });

        let (runtime_command_tx, runtime_command_rx) = mpsc::unbounded_channel();
        let current_runtime_state_tx = discro::Publisher::new(None);

        runtime.spawn({
            let current_state_rx = current_state_tx.subscribe();
            let current_runtime_state_rx = current_runtime_state_tx.subscribe();
            debug_assert!(matches!(*current_state_rx.read(), State::Idle));
            let mut current_runtime_state_rx = current_runtime_state_rx;
            async move {
                while current_runtime_state_rx.changed().await.is_ok() {
                    if let Some(runtime_state) = *current_runtime_state_rx.read_ack() {
                        current_state_tx.write(State::Running(runtime_state));
                    }
                }
                log::debug!("Stop listening for state changes after runtime has been terminated");
                current_state_tx.write(State::Terminating);
            }
        });

        let join_handle = std::thread::spawn({
            let tokio_rt = runtime.handle().clone();
            let config = config.clone();
            move || tokio_rt.block_on(run(config, runtime_command_rx, current_runtime_state_tx))
        });

        self.state = InternalState::Running {
            config,
            current_state_rx,
            runtime_command_tx,
            runtime: Box::new(runtime),
        };

        Ok(join_handle)
    }

    pub(crate) fn terminate_runtime(&mut self, abort_pending_tasks: bool) -> anyhow::Result<()> {
        match &mut self.state {
            InternalState::Idle => anyhow::bail!("invalid state: {:?}", self.state()),
            InternalState::Running {
                runtime_command_tx, ..
            } => {
                let command = RuntimeCommand::Terminate {
                    abort_pending_tasks,
                };
                if let Err(command) = runtime_command_tx.send(command) {
                    anyhow::bail!(
                        "failed to send command {command:?} in state {:?}",
                        self.state()
                    );
                }
            }
        }
        Ok(())
    }

    #[cfg_attr(not(feature = "launcher-ui"), allow(unused))]
    pub(crate) fn reset_on_termination(&mut self) {
        debug_assert!(matches!(self.state(), State::Terminating));
        self.state = InternalState::Idle;
    }
}
