use std::cmp::Ordering;

use crate::compat::is_slice_sorted_by;

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

pub mod clock;
pub mod color;

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

impl<T> IsEmpty for [T] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
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

pub trait CanonicalOrd {
    fn canonical_cmp(&self, other: &Self) -> Ordering;
}

pub fn sort_slice_canonically<T: CanonicalOrd>(slice: &mut [T]) {
    slice.sort_unstable_by(|lhs, rhs| lhs.canonical_cmp(rhs));
    debug_assert!(is_slice_sorted_canonically(slice));
}

pub fn is_slice_sorted_canonically<T: CanonicalOrd>(slice: &[T]) -> bool {
    is_slice_sorted_by(slice, |lhs, rhs| lhs.canonical_cmp(rhs))
}

pub trait Canonicalize {
    fn canonicalize(&mut self);

    fn is_canonicalized(&self) -> bool;
}

pub fn is_slice_canonicalized(slice: &[impl Canonicalize]) -> bool {
    slice.iter().all(Canonicalize::is_canonicalized)
}

impl<T> Canonicalize for Option<T>
where
    T: Canonicalize,
{
    fn canonicalize(&mut self) {
        self.as_mut().map(Canonicalize::canonicalize);
    }

    fn is_canonicalized(&self) -> bool {
        self.as_ref()
            .map(Canonicalize::is_canonicalized)
            .unwrap_or(true)
    }
}

impl<T> Canonicalize for &mut [T]
where
    T: Canonicalize,
{
    fn canonicalize(&mut self) {
        for elem in self.iter_mut() {
            elem.canonicalize();
        }
    }

    fn is_canonicalized(&self) -> bool {
        is_slice_canonicalized(self)
    }
}

impl<T> Canonicalize for Vec<T>
where
    T: Canonicalize,
{
    fn canonicalize(&mut self) {
        self.as_mut_slice().canonicalize();
    }

    fn is_canonicalized(&self) -> bool {
        is_slice_canonicalized(self)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

// TODO
