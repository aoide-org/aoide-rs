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

use crate::util::roundtrip::{
    PendingWatermark, Watermark, WatermarkFinishPending as _, WatermarkStartPending as _,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DataSnapshot<T> {
    pub value: T,
    pub since: Instant,
}

impl<T> DataSnapshot<T> {
    pub fn new(value: impl Into<T>, since: impl Into<Instant>) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundtripState {
    Idle {
        watermark: Watermark,
    },
    Pending {
        watermark: PendingWatermark,
        since: Instant,
    },
}

impl RoundtripState {
    pub const fn new() -> Self {
        Self::Idle {
            watermark: Watermark::INITIAL,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn start_pending(&mut self, since: impl Into<Instant>) -> PendingWatermark {
        let since = since.into();
        let watermark = match self {
            Self::Idle { watermark } => watermark.start_pending(),
            Self::Pending {
                watermark,
                since: _since,
            } => {
                debug_assert!(*_since <= since);
                watermark.start_pending()
            }
        };
        *self = Self::Pending { watermark, since };
        watermark
    }

    pub fn finish_pending(&mut self, watermark: PendingWatermark) -> bool {
        match *self {
            Self::Idle { .. } => false,
            Self::Pending {
                watermark: self_watermark,
                since: _,
            } => match self_watermark.finish_pending(watermark) {
                Ok(watermark) => {
                    *self = Self::Idle { watermark };
                    true
                }
                Err(_) => false,
            },
        }
    }
}

#[derive(Debug)]
pub struct RemoteData<T> {
    roundtrip_state: RoundtripState,
    last_snapshot: Option<DataSnapshot<T>>,
}

impl<T> RemoteData<T> {
    pub const fn default() -> Self {
        Self {
            roundtrip_state: RoundtripState::new(),
            last_snapshot: None,
        }
    }

    pub fn last_snapshot(&self) -> Option<&DataSnapshot<T>> {
        self.last_snapshot.as_ref()
    }

    pub fn last_value(&self) -> Option<&T> {
        self.last_snapshot.as_ref().map(|x| &x.value)
    }

    pub fn reset(&mut self) -> Option<DataSnapshot<T>> {
        self.roundtrip_state.reset();
        self.last_snapshot.take()
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.roundtrip_state, RoundtripState::Pending { .. })
    }

    /// Start the next round with a pending request
    ///
    /// Requests that are already pending will be discarded when finished.
    pub fn start_pending(&mut self, since: impl Into<Instant>) -> PendingWatermark {
        self.roundtrip_state.start_pending(since)
    }

    /// Start the next round with a pending request new
    ///
    /// Requests that are already pending will be discarded when finished.
    pub fn start_pending_now(&mut self) -> PendingWatermark {
        self.start_pending(Instant::now())
    }

    /// Try to start the next round with a pending request
    ///
    /// Allows only a single pending request at a time.
    pub fn try_start_pending(&mut self, since: impl Into<Instant>) -> Option<PendingWatermark> {
        (!self.is_pending()).then(|| self.start_pending(since))
    }

    /// Try to start the next round with a pending request
    ///
    /// Allows only a single pending request at a time.
    pub fn try_start_pending_now(&mut self) -> Option<PendingWatermark> {
        (!self.is_pending()).then(|| self.start_pending(Instant::now()))
    }

    /// Finish a pending request without touching or updating any data
    pub fn finish_pending(&mut self, token: PendingWatermark) -> bool {
        self.roundtrip_state.finish_pending(token)
    }

    /// Finish a pending request
    ///
    /// Returns the last data snapshot if accepted or the given value if rejected.
    pub fn finish_pending_with_value(
        &mut self,
        token: PendingWatermark,
        value: impl Into<T>,
        since: impl Into<Instant>,
    ) -> Result<Option<DataSnapshot<T>>, T> {
        if !self.finish_pending(token) {
            return Err(value.into());
        }
        let last_snapshot = self.last_snapshot.replace(DataSnapshot::new(value, since));
        Ok(last_snapshot)
    }

    /// Finish a pending request now
    ///
    /// Returns the last data snapshot if accepted or the given value if rejected.
    pub fn finish_pending_with_value_now(
        &mut self,
        token: PendingWatermark,
        value: impl Into<T>,
    ) -> Result<Option<DataSnapshot<T>>, T> {
        if !self.finish_pending(token) {
            return Err(value.into());
        }
        let last_snapshot = self.last_snapshot.replace(DataSnapshot::now(value));
        Ok(last_snapshot)
    }
}

impl<T> Default for RemoteData<T> {
    fn default() -> Self {
        Self::default()
    }
}
