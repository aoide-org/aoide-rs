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
use std::sync::{atomic::AtomicUsize, Arc};

use crate::prelude::send_message;

use super::{Effect, Intent, Message, MessageSender, Task};

/// Immutable environment
#[derive(Debug)]
pub struct Environment {
    api_url: Url,
    client: Client,
    pending_tasks_count: AtomicUsize,
}

impl Environment {
    pub fn new(api_url: Url) -> Self {
        Self {
            api_url,
            client: Client::new(),
            pending_tasks_count: AtomicUsize::new(0),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn join_api_url(&self, input: &str) -> anyhow::Result<Url> {
        self.api_url.join(input).map_err(Into::into)
    }
}

impl crate::prelude::Environment<Intent, Effect, Task> for Environment {
    fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_count
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }

    fn dispatch_task(&self, shared_self: Arc<Self>, message_tx: MessageSender, task: Task) {
        shared_self
            .pending_tasks_count
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        tokio::spawn(async move {
            log::debug!("Executing task: {:?}", task);
            let effect = match task {
                Task::TimedIntent { not_before, intent } => {
                    tokio::time::sleep_until(not_before.into()).await;
                    Effect::ApplyIntent(*intent)
                }
                Task::ActiveCollection(task) => task.execute_with(&shared_self).await.into(),
                Task::MediaTracker(task) => task.execute_with(&shared_self).await.into(),
            };
            log::debug!("Received effect from task: {:?}", effect);
            send_message(&message_tx, Message::Effect(effect));
            shared_self
                .pending_tasks_count
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        });
    }
}
