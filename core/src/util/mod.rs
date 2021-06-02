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

use std::{borrow::Borrow, cmp::Ordering, ops::Deref};

pub mod clock;
pub mod color;
pub mod url;

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

/// Check if a slice is sorted and does not contain duplicates
pub fn is_iter_sorted_strictly_by<'a, T, F>(
    mut iter: impl Iterator<Item = &'a T>,
    mut cmp: F,
) -> bool
where
    F: FnMut(&'a T, &'a T) -> Ordering,
    T: 'a,
{
    if let Some(first) = iter.next() {
        let mut prev = first;
        for next in iter {
            if cmp(prev, next) != Ordering::Less {
                return false;
            }
            prev = next;
        }
    }
    true
}

/// Check if a slice is sorted and does not contain duplicates
pub fn is_slice_sorted_strictly_by<T, F>(slice: &[T], cmp: F) -> bool
where
    F: FnMut(&T, &T) -> Ordering,
{
    is_iter_sorted_strictly_by(slice.iter(), cmp)
}

pub trait CanonicalOrd {
    fn canonical_cmp(&self, other: &Self) -> Ordering;
}

impl<T> CanonicalOrd for [T]
where
    T: CanonicalOrd,
{
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        if self.len() != other.len() {
            return self.len().cmp(&other.len());
        }
        for (lhs, rhs) in self.iter().zip(other.iter()) {
            let ord = lhs.canonical_cmp(rhs);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    }
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
            && is_slice_sorted_strictly_by(self, |lhs, rhs| lhs.canonical_cmp(rhs))
    }
}

impl<T> IsCanonical for &[T]
where
    T: IsCanonical + CanonicalOrd,
{
    fn is_canonical(&self) -> bool {
        (**self).is_canonical()
    }
}

impl<T> IsCanonical for &mut [T]
where
    T: IsCanonical + CanonicalOrd,
{
    fn is_canonical(&self) -> bool {
        (&**self).is_canonical()
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

pub trait CanonicalizeInto: Canonicalize {
    // The return type Canonical<Self> would be more appropriate,
    // but is not permitted.
    fn canonicalize_into(self) -> Self;
}

impl<T> CanonicalizeInto for T
where
    T: Canonicalize,
{
    fn canonicalize_into(mut self) -> Self {
        self.canonicalize();
        self
    }
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

impl<T> Canonicalize for Vec<T>
where
    T: Canonicalize + CanonicalOrd,
{
    fn canonicalize(&mut self) {
        for elem in self.iter_mut() {
            elem.canonicalize();
        }
        self.sort_unstable_by(|lhs, rhs| lhs.canonical_cmp(rhs));
        self.dedup_by(|lhs, rhs| lhs.canonical_cmp(rhs) == Ordering::Equal);
        debug_assert!(self.is_canonical());
    }
}

/// Type-safe wrapper for immutable, canonical data.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Canonical<T>(T);

impl<T> Canonical<T>
where
    T: IsCanonical + std::fmt::Debug,
{
    pub fn tie(canonical: T) -> Self {
        debug_assert!(canonical.is_canonical());
        Self(canonical)
    }

    pub fn untie(self) -> T {
        let Canonical(canonical) = self;
        canonical
    }
}

impl<T> Canonical<Vec<T>>
where
    T: IsCanonical,
{
    pub fn as_slice(&self) -> Canonical<&[T]> {
        Canonical(self.as_ref().as_slice())
    }
}

impl<T> IsCanonical for Canonical<T>
where
    T: IsCanonical,
{
    fn is_canonical(&self) -> bool {
        true
    }
}

impl<T> AsRef<T> for Canonical<T> {
    fn as_ref(&self) -> &T {
        let Canonical(canonical) = self;
        canonical
    }
}

impl<T> Deref for Canonical<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> Borrow<T> for Canonical<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
