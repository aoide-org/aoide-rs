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

use std::sync::Arc;

use crate::message::MessageSender;

pub trait TaskDispatcher<Intent, Effect, Task> {
    fn all_tasks_finished(&self) -> bool;

    fn dispatch_task(
        &self,
        shared_self: Arc<Self>,
        message_tx: MessageSender<Intent, Effect>,
        task: Task,
    );
}

use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
pub struct PendingTasksCounter {
    number_of_pending_tasks: AtomicUsize,
}

impl PendingTasksCounter {
    pub const fn new() -> Self {
        Self {
            number_of_pending_tasks: AtomicUsize::new(0),
        }
    }
}

impl PendingTasksCounter {
    pub fn start_pending_task(&self) -> usize {
        let pending_tasks = self
            .number_of_pending_tasks
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            + 1;
        debug_assert!(!self.all_pending_tasks_finished());
        pending_tasks
    }

    pub fn finish_pending_task(&self) -> usize {
        debug_assert!(!self.all_pending_tasks_finished());
        self.number_of_pending_tasks
            .fetch_sub(1, std::sync::atomic::Ordering::Release)
            - 1
    }

    pub fn all_pending_tasks_finished(&self) -> bool {
        self.number_of_pending_tasks
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }
}
