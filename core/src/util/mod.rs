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

pub trait IsCanonical {
    fn is_canonical(&self) -> bool;
}

impl<T> IsCanonical for Option<T>
where
    T: IsCanonical,
{
    fn is_canonical(&self) -> bool {
        self.as_ref().map(T::is_canonical).unwrap_or(true)
    }
}

impl<T> IsCanonical for [T]
where
    T: IsCanonical + CanonicalOrd,
{
    fn is_canonical(&self) -> bool {
        self.iter().all(T::is_canonical)
            && is_slice_sorted_by(self, |lhs, rhs| lhs.canonical_cmp(rhs))
    }
}

impl<T> IsCanonical for &mut [T]
where
    T: IsCanonical + CanonicalOrd,
{
    fn is_canonical(&self) -> bool {
        let immutable_slice: &[T] = self;
        immutable_slice.is_canonical()
    }
}

impl<T> IsCanonical for Vec<T>
where
    T: IsCanonical + CanonicalOrd,
{
    fn is_canonical(&self) -> bool {
        self.as_slice().is_canonical()
    }
}

pub trait Canonicalize: IsCanonical {
    fn canonicalize(&mut self);
}

impl<T> Canonicalize for Option<T>
where
    T: Canonicalize,
{
    fn canonicalize(&mut self) {
        self.as_mut().map(Canonicalize::canonicalize);
        debug_assert!(self.is_canonical());
    }
}

impl<T> Canonicalize for &mut [T]
where
    T: Canonicalize + CanonicalOrd,
{
    fn canonicalize(&mut self) {
        for elem in self.iter_mut() {
            elem.canonicalize();
        }
        self.sort_unstable_by(|lhs, rhs| lhs.canonical_cmp(rhs));
        debug_assert!(self.is_canonical());
    }
}

impl<T> Canonicalize for Vec<T>
where
    T: Canonicalize + CanonicalOrd,
{
    fn canonicalize(&mut self) {
        self.as_mut_slice().canonicalize();
        debug_assert!(self.is_canonical());
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

// TODO
