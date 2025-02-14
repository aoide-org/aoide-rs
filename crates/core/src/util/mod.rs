// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod clock;
pub mod color;
pub mod fs;
pub mod random;
pub mod string;
pub mod url;

pub trait IsInteger {
    fn is_integer(&self) -> bool;
}

impl IsInteger for f64 {
    fn is_integer(&self) -> bool {
        (self.trunc() - self).abs() == 0_f64
    }
}

impl IsInteger for f32 {
    fn is_integer(&self) -> bool {
        (self.trunc() - self).abs() == 0_f32
    }
}
