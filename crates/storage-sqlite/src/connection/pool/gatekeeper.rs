// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{sync::RwLock, task::spawn_blocking, time::sleep};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::{get_pooled_connection, ConnectionPool, PooledConnection};

/// Manage database connections for asynchronous tasks
///
/// Only a single writer is allowed to access the `SQLite` database
/// at any given time. This is required to prevent both synchronous
/// locking when obtaining a connection and timeouts when concurrently
/// trying to execute write operations on a shared `SQLite` database
/// instance.
#[allow(missing_debug_implementations)]
pub struct Gatekeeper {
    connection_pool: Arc<RwLock<ConnectionPool>>,
    acquire_read_timeout: Duration,
    acquire_write_timeout: Duration,
    request_counter_state: Arc<RequestCounterState>,
    abort_current_task_flag: Arc<AtomicBool>,
    decommisioned: AtomicBool,
}

#[derive(Debug, Default)]
struct RequestCounterState {
    read_count: AtomicUsize,
    write_count: AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestCounterMode {
    Read,
    Write,
}

struct RequestCounterScope {
    shared_state: Arc<RequestCounterState>,
    mode: RequestCounterMode,
}

impl RequestCounterScope {
    #[must_use]
    fn new(shared_state: Arc<RequestCounterState>, mode: RequestCounterMode) -> Self {
        match mode {
            RequestCounterMode::Read => {
                let pending_read_requests_before =
                    shared_state.read_count.fetch_add(1, Ordering::Relaxed);
                log::debug!(
                    "Starting read request: {} pending read request(s)",
                    pending_read_requests_before + 1
                );
            }
            RequestCounterMode::Write => {
                let pending_write_requests_before =
                    shared_state.write_count.fetch_add(1, Ordering::Relaxed);
                log::debug!(
                    "Starting write request: {} pending write request(s)",
                    pending_write_requests_before + 1
                );
            }
        }
        Self { shared_state, mode }
    }
}

impl Drop for RequestCounterScope {
    fn drop(&mut self) {
        match self.mode {
            RequestCounterMode::Read => {
                let pending_read_requests_before =
                    self.shared_state.read_count.fetch_sub(1, Ordering::Relaxed);
                debug_assert!(pending_read_requests_before > 0);
                log::debug!(
                    "Finished read request: {} pending read request(s)",
                    pending_read_requests_before - 1
                );
            }
            RequestCounterMode::Write => {
                let pending_write_requests_before = self
                    .shared_state
                    .write_count
                    .fetch_sub(1, Ordering::Relaxed);
                debug_assert!(pending_write_requests_before > 0);
                log::debug!(
                    "Finished write request: {} pending write request(s)",
                    pending_write_requests_before - 1
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTasks {
    pub read: usize,
    pub write: usize,
}

impl Gatekeeper {
    #[must_use]
    pub fn new(connection_pool: ConnectionPool, config: Config) -> Self {
        let Config {
            acquire_read_timeout_millis,
            acquire_write_timeout_millis,
        } = config;
        let acquire_read_timeout = Duration::from_millis(acquire_read_timeout_millis.get());
        let acquire_write_timeout = Duration::from_millis(acquire_write_timeout_millis.get());
        Self {
            connection_pool: Arc::new(RwLock::new(connection_pool)),
            acquire_read_timeout,
            acquire_write_timeout,
            request_counter_state: Default::default(),
            abort_current_task_flag: Default::default(),
            decommisioned: AtomicBool::new(false),
        }
    }

    pub fn decommission(&self) {
        self.decommisioned.store(true, Ordering::Release);
    }

    fn check_not_decommissioned(&self) -> Result<()> {
        if self.decommisioned.load(Ordering::Acquire) {
            return Err(Error::TaskTimeout {
                reason: "connection pool has been decommissioned".to_string(),
            });
        }
        Ok(())
    }

    pub async fn spawn_blocking_read_task_with_timeout<H, R>(
        &self,
        connection_handler: H,
        acquire_read_timeout: Duration,
    ) -> Result<R>
    where
        H: FnOnce(PooledConnection, Arc<AtomicBool>) -> R + Send + 'static,
        R: Send + 'static,
    {
        self.check_not_decommissioned()?;
        let _request_counter_scope = RequestCounterScope::new(
            Arc::clone(&self.request_counter_state),
            RequestCounterMode::Read,
        );
        let timeout = sleep(acquire_read_timeout);
        tokio::pin!(timeout);
        let abort_current_task_flag = Arc::clone(&self.abort_current_task_flag);
        tokio::select! {
            _ = &mut timeout => Err(Error::TaskTimeout {reason: "database is locked".to_string() }),
            guard = self.connection_pool.read() => {
                self.check_not_decommissioned()?;
                let connection = get_pooled_connection(&guard)?;
                self.check_not_decommissioned()?;
                // Every tasks gets the chance to run when ready
                abort_current_task_flag.store(false, Ordering::Release);
                spawn_blocking(move || connection_handler(connection, abort_current_task_flag)).await
                    .map_err(Error::TaskScheduling)
            },
            else => Err(Error::TaskTimeout {reason: "task got stuck".to_string() } )
        }
    }

    pub async fn spawn_blocking_read_task<H, R>(&self, connection_handler: H) -> Result<R>
    where
        H: FnOnce(PooledConnection, Arc<AtomicBool>) -> R + Send + 'static,
        R: Send + 'static,
    {
        self.spawn_blocking_read_task_with_timeout(connection_handler, self.acquire_read_timeout)
            .await
    }

    pub async fn spawn_blocking_write_task_with_timeout<H, R>(
        &self,
        connection_handler: H,
        acquire_write_timeout: Duration,
    ) -> Result<R>
    where
        H: FnOnce(PooledConnection, Arc<AtomicBool>) -> R + Send + 'static,
        R: Send + 'static,
    {
        self.check_not_decommissioned()?;
        let _request_counter_scope = RequestCounterScope::new(
            Arc::clone(&self.request_counter_state),
            RequestCounterMode::Write,
        );
        let timeout = sleep(acquire_write_timeout);
        tokio::pin!(timeout);
        let abort_current_task_flag = Arc::clone(&self.abort_current_task_flag);
        tokio::select! {
            _ = &mut timeout => Err(Error::TaskTimeout {reason: "database is locked".to_string() }),
            guard = self.connection_pool.write() => {
                self.check_not_decommissioned()?;
                let connection = get_pooled_connection(&guard)?;
                self.check_not_decommissioned()?;
                // Every tasks gets the chance to run when ready
                abort_current_task_flag.store(false, Ordering::Release);
                spawn_blocking(move || connection_handler(connection, abort_current_task_flag)).await
                .map_err(Error::TaskScheduling)
            },
            else => Err(Error::TaskTimeout {reason: "task got stuck".to_string() } )
        }
    }

    pub async fn spawn_blocking_write_task<H, R>(&self, connection_handler: H) -> Result<R>
    where
        H: FnOnce(PooledConnection, Arc<AtomicBool>) -> R + Send + 'static,
        R: Send + 'static,
    {
        self.spawn_blocking_write_task_with_timeout(connection_handler, self.acquire_write_timeout)
            .await
    }

    pub fn pending_tasks(&self) -> PendingTasks {
        PendingTasks {
            read: self
                .request_counter_state
                .read_count
                .load(Ordering::Relaxed),
            write: self
                .request_counter_state
                .write_count
                .load(Ordering::Relaxed),
        }
    }

    pub fn abort_current_task(&self) {
        self.abort_current_task_flag.store(true, Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub acquire_read_timeout_millis: NonZeroU64,
    pub acquire_write_timeout_millis: NonZeroU64,
}
