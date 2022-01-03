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

type EpochNumber = usize;

const INITIAL_EPOCH_NUMBER: EpochNumber = 0;

type SequenceNumber = usize;

const INITIAL_SEQUENCE_NUMBER: SequenceNumber = 0;

const FINAL_SEQUENCE_NUMBER: SequenceNumber = SequenceNumber::MAX;

const MAX_SEQUENCE_NUMBER_DISTANCE: SequenceNumber = SequenceNumber::MAX / 2;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Watermark {
    epoch: EpochNumber,
    sequence: SequenceNumber,
}

impl Watermark {
    pub const INITIAL: Self = Self {
        epoch: INITIAL_EPOCH_NUMBER,
        sequence: INITIAL_SEQUENCE_NUMBER,
    };

    pub const fn is_initial(self) -> bool {
        self.sequence == INITIAL_SEQUENCE_NUMBER
    }

    pub const fn is_final(self) -> bool {
        self.sequence == FINAL_SEQUENCE_NUMBER
    }

    pub const fn new() -> Self {
        Self::INITIAL
    }

    pub fn reset(&mut self) {
        self.bump_epoch()
    }

    pub fn finalize(&mut self) {
        self.sequence = FINAL_SEQUENCE_NUMBER;
    }

    const fn is_pending(self) -> bool {
        static_assertions::const_assert!(INITIAL_SEQUENCE_NUMBER % 2 == 0);
        debug_assert!(!self.is_final());
        self.sequence % 2 != 0
    }

    fn bump_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.sequence = INITIAL_SEQUENCE_NUMBER;
    }

    fn bump_sequence(&mut self) {
        self.sequence = self.sequence.wrapping_add(1);
    }
}

impl PartialOrd for Watermark {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.epoch != other.epoch {
            return None;
        }
        let ordering = match self.sequence.cmp(&other.sequence) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Less => {
                let distance = other.sequence - self.sequence;
                if distance > MAX_SEQUENCE_NUMBER_DISTANCE {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            Ordering::Greater => {
                debug_assert!(self.sequence > other.sequence);
                let distance = self.sequence - other.sequence;
                if distance > MAX_SEQUENCE_NUMBER_DISTANCE {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        };
        Some(ordering)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PendingWatermark(Watermark);

impl AsRef<Watermark> for PendingWatermark {
    fn as_ref(&self) -> &Watermark {
        &self.0
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
            this.bump_sequence();
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
        match self.partial_cmp(&pending) {
            None | Some(Ordering::Greater) => Err(self),
            Some(Ordering::Equal) | Some(Ordering::Less) => {
                let mut finished = pending;
                finished.bump_sequence();
                debug_assert!(!finished.is_pending());
                Ok(finished)
            }
        }
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
