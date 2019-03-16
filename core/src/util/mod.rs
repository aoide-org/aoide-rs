// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

pub mod clock;
pub mod color;

pub trait IsValid {
    fn is_valid(&self) -> bool;
}

pub trait IsInteger {
    fn is_integer(&self) -> bool;
}

impl IsInteger for f64 {
    fn is_integer(&self) -> bool {
        (self.trunc() - self).abs() == 0f64
    }
}

impl IsInteger for f32 {
    fn is_integer(&self) -> bool {
        (self.trunc() - self).abs() == 0f32
    }
}

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for std::time::Duration {
    fn is_empty(&self) -> bool {
        *self == std::time::Duration::from_secs(0)
    }
}

impl IsEmpty for chrono::Duration {
    fn is_empty(&self) -> bool {
        self.is_zero()
    }
}

pub trait IsDefault {
    fn is_default(&self) -> bool;
}

impl<T> IsDefault for T
where
    T: Default + PartialEq,
{
    fn is_default(&self) -> bool {
        self == &Default::default()
    }
}
