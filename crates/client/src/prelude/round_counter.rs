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

pub type Value = usize;

pub const INITIAL_VALUE: Value = 0;

pub const FINAL_VALUE: Value = Value::MAX;

const MAX_VALUE_DIFF: Value = Value::MAX / 2;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RoundCounter(Value);

impl RoundCounter {
    pub const INITIAL: Self = Self(INITIAL_VALUE);

    pub const FINAL: Self = Self(FINAL_VALUE);

    pub const fn is_initial(self) -> bool {
        self.0 == INITIAL_VALUE
    }

    pub const fn is_final(self) -> bool {
        self.0 == FINAL_VALUE
    }

    pub const fn default() -> Self {
        Self::INITIAL
    }

    pub fn reset(&mut self) {
        self.0 = INITIAL_VALUE;
    }

    pub fn finalize(&mut self) {
        self.0 = FINAL_VALUE;
    }

    pub const fn is_pending(self) -> bool {
        static_assertions::const_assert!(INITIAL_VALUE % 2 == 0);
        debug_assert!(!self.is_final());
        self.0 % 2 != 0
    }

    fn bump_value(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn start_next_round(&mut self) {
        debug_assert!(!self.is_final());
        while !self.is_pending() {
            self.bump_value();
            if self.is_final() {
                self.reset();
                debug_assert!(!self.is_pending());
            }
        }
    }

    pub fn finish_pending_round(&mut self, pending_round: Self) -> bool {
        debug_assert!(!self.is_final());
        debug_assert!(pending_round.is_pending());
        if *self != pending_round {
            if self.0 < pending_round.0 {
                let value_diff = pending_round.0 - self.0;
                if value_diff > MAX_VALUE_DIFF {
                    return false;
                }
            } else {
                debug_assert!(self.0 > pending_round.0);
                let value_diff = self.0 - pending_round.0;
                if value_diff <= MAX_VALUE_DIFF {
                    return false;
                }
            }
        }
        *self = pending_round;
        self.bump_value();
        debug_assert!(!self.is_pending());
        true
    }
}

impl Default for RoundCounter {
    fn default() -> Self {
        Self::default()
    }
}
