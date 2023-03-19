// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{atomic::AtomicUsize, Arc};

use infect::{TaskContext, TaskExecutor};
use reqwest::{Client, Url};

use aoide_client::webapi::ClientEnvironment;

use super::{Effect, Intent, Task};

/// Immutable environment
#[derive(Debug)]
pub(crate) struct Environment {
    service_url: Url,
    client: Client,
    pending_tasks_counter: PendingTasksCounter,
}

impl Environment {
    #[must_use]
    pub(crate) fn new(service_url: Url) -> Self {
        Self {
            service_url,
            client: Client::new(),
            pending_tasks_counter: PendingTasksCounter::new(),
        }
    }

    #[must_use]
    pub(crate) fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_counter.all_tasks_finished()
    }
}

impl ClientEnvironment for Environment {
    fn client(&self) -> &Client {
        &self.client
    }

    fn join_api_url(&self, query_suffix: &str) -> anyhow::Result<Url> {
        let api_url = self.service_url.join("api/")?.join(query_suffix)?;
        log::debug!("API URL: {api_url}");
        Ok(api_url)
    }
}

impl TaskExecutor<Arc<Environment>> for Environment {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn spawn_task(
        &self,
        mut context: TaskContext<Arc<Environment>, Self::Intent, Self::Effect>,
        task: Self::Task,
    ) {
        let started_pending_task = self.pending_tasks_counter.start_task();
        debug_assert!(started_pending_task > 0);
        if started_pending_task == 1 {
            log::debug!("Started first pending task");
        }
        tokio::spawn(async move {
            log::debug!("Executing task {task:?}");
            let effect = task.execute(&*context.task_executor).await;
            log::debug!("Task finished with effect: {effect:?}");
            context.submit_effect(effect);
            if context.task_executor.pending_tasks_counter.finish_task() == 0 {
                log::debug!("Finished last pending task");
            }
        });
    }
}

#[derive(Debug)]
struct PendingTasksCounter {
    number_of_pending_tasks: AtomicUsize,
}

impl PendingTasksCounter {
    const fn new() -> Self {
        Self {
            number_of_pending_tasks: AtomicUsize::new(0),
        }
    }
}

impl PendingTasksCounter {
    fn start_task(&self) -> usize {
        let pending_tasks = self
            .number_of_pending_tasks
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            + 1;
        debug_assert!(!self.all_tasks_finished());
        pending_tasks
    }

    fn finish_task(&self) -> usize {
        debug_assert!(!self.all_tasks_finished());
        self.number_of_pending_tasks
            .fetch_sub(1, std::sync::atomic::Ordering::Release)
            - 1
    }

    fn all_tasks_finished(&self) -> bool {
        self.number_of_pending_tasks
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }
}
