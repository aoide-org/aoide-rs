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

use super::Intent;

use crate::models::{active_collection, media_tracker};

use std::time::Instant;

#[derive(Debug)]
pub enum Task {
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    ActiveCollection(active_collection::Task),
    MediaTracker(media_tracker::Task),
}

impl From<active_collection::Task> for Task {
    fn from(task: active_collection::Task) -> Self {
        Self::ActiveCollection(task)
    }
}

impl From<media_tracker::Task> for Task {
    fn from(task: media_tracker::Task) -> Self {
        Self::MediaTracker(task)
    }
}
