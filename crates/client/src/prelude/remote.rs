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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DataSnapshot<T> {
    pub since: Instant,
    pub value: T,
}

impl<T> DataSnapshot<T> {
    pub fn new(since: impl Into<Instant>, value: impl Into<T>) -> Self {
        Self {
            since: since.into(),
            value: value.into(),
        }
    }

    pub fn now(value: impl Into<T>) -> Self {
        Self {
            since: Instant::now(),
            value: value.into(),
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RemoteData<T> {
    Unknown,
    Pending {
        stale_snapshot: DataSnapshot<Option<DataSnapshot<T>>>,
    },
    Ready {
        snapshot: Option<DataSnapshot<T>>,
    },
}

impl<T> Default for RemoteData<T> {
    fn default() -> Self {
        Self::Unknown
    }
}

impl<T> RemoteData<T> {
    pub fn ready_since(since: impl Into<Instant>, value: impl Into<T>) -> Self {
        Self::Ready {
            snapshot: Some(DataSnapshot::new(since, value)),
        }
    }

    pub fn ready_now(value: impl Into<T>) -> Self {
        Self::Ready {
            snapshot: Some(DataSnapshot::now(value)),
        }
    }

    pub fn get(&self) -> Option<&DataSnapshot<T>> {
        match self {
            Self::Unknown => None,
            Self::Pending { stale_snapshot } => stale_snapshot.value.as_ref(),
            Self::Ready { snapshot } => snapshot.as_ref(),
        }
    }

    pub fn get_ready(&self) -> Option<&DataSnapshot<T>> {
        match self {
            Self::Unknown | Self::Pending { .. } => None,
            Self::Ready { snapshot } => snapshot.as_ref(),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut DataSnapshot<T>> {
        match self {
            Self::Unknown | Self::Pending { .. } => None,
            Self::Ready { snapshot } => snapshot.as_mut(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Unknown;
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn take_ready(&mut self) -> Option<DataSnapshot<T>> {
        if let Self::Ready { snapshot } = self {
            let snapshot = snapshot.take();
            debug_assert!(snapshot.is_some());
            *self = Self::Unknown;
            snapshot
        } else {
            None
        }
    }

    pub fn try_set_pending_since(&mut self, since: impl Into<Instant>) -> bool {
        if self.is_pending() {
            return false;
        }
        let stale_snapshot = DataSnapshot {
            since: since.into(),
            value: self.take_ready(),
        };
        debug_assert!(
            stale_snapshot.since
                >= stale_snapshot
                    .value
                    .as_ref()
                    .map(|x| x.since)
                    .unwrap_or(stale_snapshot.since)
        );
        *self = Self::Pending { stale_snapshot };
        true
    }

    pub fn try_set_pending_now(&mut self) -> bool {
        self.try_set_pending_since(Instant::now())
    }
}
