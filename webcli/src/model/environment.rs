// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::sync::{atomic::AtomicUsize, Arc};

use reqwest::{Client, Url};

use aoide_client::{
    messaging::{send_message, TaskDispatcher},
    webapi::ClientEnvironment,
};

use super::{Effect, Intent, Message, MessageSender, Task};

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
}

impl ClientEnvironment for Environment {
    fn client(&self) -> &Client {
        &self.client
    }

    fn join_api_url(&self, query_suffix: &str) -> anyhow::Result<Url> {
        let api_url = self.service_url.join("api/")?.join(query_suffix)?;
        log::debug!("API URL: {}", api_url);
        Ok(api_url)
    }
}

impl TaskDispatcher<Intent, Effect, Task> for Environment {
    fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_counter.all_pending_tasks_finished()
    }

    fn dispatch_task(&self, shared_self: Arc<Self>, message_tx: MessageSender, task: Task) {
        let started_pending_task = shared_self.pending_tasks_counter.start_pending_task();
        debug_assert!(started_pending_task > 0);
        if started_pending_task == 1 {
            log::debug!("Started first pending task");
        }
        tokio::spawn(async move {
            log::debug!("Executing task: {:?}", task);
            let effect = task.execute(&*shared_self).await;
            log::debug!("Task finished with effect: {:?}", effect);
            send_message(&message_tx, Message::Effect(effect));
            if shared_self.pending_tasks_counter.finish_pending_task() == 0 {
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
    fn start_pending_task(&self) -> usize {
        let pending_tasks = self
            .number_of_pending_tasks
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            + 1;
        debug_assert!(!self.all_pending_tasks_finished());
        pending_tasks
    }

    fn finish_pending_task(&self) -> usize {
        debug_assert!(!self.all_pending_tasks_finished());
        self.number_of_pending_tasks
            .fetch_sub(1, std::sync::atomic::Ordering::Release)
            - 1
    }

    fn all_pending_tasks_finished(&self) -> bool {
        self.number_of_pending_tasks
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }
}
