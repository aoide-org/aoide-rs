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

use std::time::Instant;

use super::round_counter::RoundCounter;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DataSnapshot<T> {
    pub value: T,
    pub since: Instant,
}

impl<T> DataSnapshot<T> {
    pub fn new_since(value: impl Into<T>, since: impl Into<Instant>) -> Self {
        Self {
            value: value.into(),
            since: since.into(),
        }
    }

    pub fn now(value: impl Into<T>) -> Self {
        Self {
            value: value.into(),
            since: Instant::now(),
        }
    }

    pub fn as_ref(&self) -> DataSnapshot<&T> {
        let Self { since, value } = self;
        DataSnapshot {
            since: *since,
            value,
        }
    }
}

#[derive(Debug)]
pub struct RemoteData<T> {
    round_counter: RoundCounter,
    pending_since: Option<Instant>,
    last_data_snapshot: Option<DataSnapshot<T>>,
}

impl<T> RemoteData<T> {
    pub const fn default() -> Self {
        Self {
            round_counter: RoundCounter::default(),
            pending_since: None,
            last_data_snapshot: None,
        }
    }

    pub fn round_counter(&self) -> RoundCounter {
        self.round_counter
    }

    pub fn last_data_snapshot(&self) -> Option<&DataSnapshot<T>> {
        self.last_data_snapshot.as_ref()
    }

    pub fn last_value(&self) -> Option<&T> {
        self.last_data_snapshot.as_ref().map(|x| &x.value)
    }

    pub fn reset(&mut self) -> Option<DataSnapshot<T>> {
        self.round_counter.reset();
        self.pending_since = None;
        self.last_data_snapshot.take()
    }

    pub fn is_pending(&self) -> bool {
        self.round_counter.is_pending()
    }

    /// Start the next round with a pending request
    ///
    /// Requests that are already pending will be discarded when finished.
    pub fn set_pending_since(&mut self, since: impl Into<Instant>) -> RoundCounter {
        self.round_counter.start_next_round();
        self.pending_since = Some(since.into());
        self.round_counter
    }

    /// Try to start the next round with a pending request
    ///
    /// Allows only a single pending request at a time.
    pub fn try_set_pending_since(&mut self, since: impl Into<Instant>) -> Option<RoundCounter> {
        (!self.is_pending()).then(|| self.set_pending_since(since))
    }

    /// Try to start the next round with a pending request
    ///
    /// Allows only a single pending request at a time.
    pub fn try_set_pending_now(&mut self) -> Option<RoundCounter> {
        (!self.is_pending()).then(|| self.set_pending_since(Instant::now()))
    }

    /// Finish a pending request
    ///
    /// Returns the last data snapshot if accepted or `None` if rejected.
    pub fn finish_pending_round_with_value_since(
        &mut self,
        pending_round: RoundCounter,
        value: impl Into<T>,
        since: impl Into<Instant>,
    ) -> Option<DataSnapshot<T>> {
        if !self.round_counter.finish_pending_round(pending_round) {
            return None;
        }
        self.pending_since = None;
        self.last_data_snapshot
            .replace(DataSnapshot::new_since(value, since))
    }

    /// Finish a pending request now
    ///
    /// Returns the last data snapshot if accepted or `None` if rejected.
    pub fn finish_pending_round_with_value_now(
        &mut self,
        pending_round: RoundCounter,
        value: impl Into<T>,
    ) -> Option<DataSnapshot<T>> {
        if !self.round_counter.finish_pending_round(pending_round) {
            return None;
        }
        self.pending_since = None;
        self.last_data_snapshot.replace(DataSnapshot::now(value))
    }
}

impl<T> Default for RemoteData<T> {
    fn default() -> Self {
        Self::default()
    }
}
