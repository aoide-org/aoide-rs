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

use reqwest::{Client, Url};
use std::sync::Arc;

use crate::{
    prelude::{send_message, PendingTasksCounter, TaskDispatchEnvironment},
    WebClientEnvironment,
};

use super::{Effect, Intent, Message, MessageSender, Task};

/// Immutable environment
#[derive(Debug)]
pub struct Environment {
    service_url: Url,
    client: Client,
    pending_tasks_counter: PendingTasksCounter,
}

impl Environment {
    pub fn new(service_url: Url) -> Self {
        Self {
            service_url,
            client: Client::new(),
            pending_tasks_counter: PendingTasksCounter::new(),
        }
    }
}

impl WebClientEnvironment for Environment {
    fn client(&self) -> &Client {
        &self.client
    }

    fn join_api_url(&self, query_suffix: &str) -> anyhow::Result<Url> {
        let api_url = self.service_url.join("api/")?.join(query_suffix)?;
        log::debug!("API URL: {}", api_url);
        Ok(api_url)
    }
}

impl TaskDispatchEnvironment<Intent, Effect, Task> for Environment {
    fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_counter.all_pending_tasks_finished()
    }

    fn dispatch_task(&self, shared_self: Arc<Self>, message_tx: MessageSender, task: Task) {
        shared_self.pending_tasks_counter.start_pending_task();
        tokio::spawn(async move {
            log::debug!("Executing task: {:?}", task);
            let effect = task.execute(&*shared_self).await;
            log::debug!("Task finished with effect: {:?}", effect);
            send_message(&message_tx, Message::Effect(effect));
            shared_self.pending_tasks_counter.finish_pending_task();
        });
    }
}
