use std::cmp::Ordering;

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

type SequenceNumber = usize;

const INITIAL_SEQUENCE_NUMBER: SequenceNumber = 0;

const FINAL_SEQUENCE_NUMBER: SequenceNumber = SequenceNumber::MAX;

const MAX_SEQUENCE_NUMBER_DISTANCE: SequenceNumber = SequenceNumber::MAX / 2;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Watermark(SequenceNumber);

impl Watermark {
    pub const INITIAL: Self = Self(INITIAL_SEQUENCE_NUMBER);

    pub const FINAL: Self = Self(FINAL_SEQUENCE_NUMBER);

    pub const fn is_initial(self) -> bool {
        self.0 == INITIAL_SEQUENCE_NUMBER
    }

    pub const fn is_final(self) -> bool {
        self.0 == FINAL_SEQUENCE_NUMBER
    }

    pub const fn new() -> Self {
        Self::INITIAL
    }

    pub fn reset(&mut self) {
        self.0 = INITIAL_SEQUENCE_NUMBER;
    }

    pub fn finalize(&mut self) {
        self.0 = FINAL_SEQUENCE_NUMBER;
    }

    const fn is_pending(self) -> bool {
        static_assertions::const_assert!(INITIAL_SEQUENCE_NUMBER % 2 == 0);
        debug_assert!(!self.is_final());
        self.0 % 2 != 0
    }

    fn bump_value(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

impl Ord for Watermark {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Less => {
                let distance = other.0 - self.0;
                if distance > MAX_SEQUENCE_NUMBER_DISTANCE {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            Ordering::Greater => {
                debug_assert!(self.0 > other.0);
                let distance = self.0 - other.0;
                if distance > MAX_SEQUENCE_NUMBER_DISTANCE {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        }
    }
}

impl PartialOrd for Watermark {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PendingWatermark(Watermark);

impl AsRef<Watermark> for PendingWatermark {
    fn as_ref(&self) -> &Watermark {
        &self.0
    }
}

impl PendingWatermark {
    pub fn finish_pending(self, other: Self) -> Result<Watermark, Self> {
        let Self(self_pending) = self;
        debug_assert!(self_pending.is_pending());
        let Self(other_pending) = other;
        debug_assert!(other_pending.is_pending());
        if self_pending > other_pending {
            return Err(self);
        }
        debug_assert!(self_pending <= other_pending);
        let mut finished = other_pending;
        finished.bump_value();
        debug_assert!(!finished.is_pending());
        Ok(finished)
    }
}

pub trait WatermarkStartPending {
    fn start_pending(self) -> PendingWatermark;
}

impl WatermarkStartPending for Watermark {
    fn start_pending(self) -> PendingWatermark {
        let mut this = self;
        debug_assert!(!this.is_final());
        while !this.is_pending() {
            this.bump_value();
            if this.is_final() {
                this.reset();
                debug_assert!(!this.is_pending());
            }
        }
        PendingWatermark(this)
    }
}

impl WatermarkStartPending for PendingWatermark {
    fn start_pending(self) -> Self {
        let Self(this) = self;
        debug_assert!(this.is_pending());
        let token = this.start_pending();
        debug_assert_ne!(token, self);
        token
    }
}

pub trait WatermarkFinishPending: Sized {
    fn finish_pending(self, pending: PendingWatermark) -> Result<Watermark, Self>;
}

impl WatermarkFinishPending for Watermark {
    fn finish_pending(self, token: PendingWatermark) -> Result<Self, Self> {
        let PendingWatermark(pending) = token;
        debug_assert!(pending.is_pending());
        if self > pending {
            return Err(self);
        }
        debug_assert!(self <= pending);
        let mut finished = pending;
        finished.bump_value();
        debug_assert!(!finished.is_pending());
        Ok(finished)
    }
}

impl WatermarkFinishPending for PendingWatermark {
    fn finish_pending(self, token: PendingWatermark) -> Result<Watermark, Self> {
        let PendingWatermark(this) = self;
        this.finish_pending(token).map_err(Self)
    }
}

impl Default for Watermark {
    fn default() -> Self {
        Self::new()
    }
}
