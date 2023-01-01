// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Borrow, cmp::Ordering, ops::Deref};

/// Check if an iterable is sorted and does not contain duplicates
fn is_sorted_strictly_by<'a, T, F>(iterable: impl IntoIterator<Item = &'a T>, mut cmp: F) -> bool
where
    F: FnMut(&'a T, &'a T) -> Ordering,
    T: 'a,
{
    let mut iter = iterable.into_iter();
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

pub trait CanonicalOrd {
    fn canonical_cmp(&self, other: &Self) -> Ordering;

    /// Ordering fore deduplication.
    ///
    /// Only used for disambiguation, i.e. Will be chained with
    /// `canonical_cmp()`. Should return `Ordering::Less` for items
    /// that should take precedence during deduplication.
    fn canonical_dedup_cmp(&self, other: &Self) -> Ordering {
        debug_assert_eq!(Ordering::Equal, self.canonical_cmp(other));
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
        self.as_ref().map_or(true, T::is_canonical)
    }
}

impl<T> IsCanonical for [T]
where
    T: IsCanonical + CanonicalOrd,
{
    fn is_canonical(&self) -> bool {
        self.iter().all(T::is_canonical) && is_sorted_strictly_by(self, CanonicalOrd::canonical_cmp)
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
        (**self).is_canonical()
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
    #[must_use]
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
        self.sort_unstable_by(|lhs, rhs| {
            lhs.canonical_cmp(rhs)
                .then_with(|| lhs.canonical_dedup_cmp(rhs))
        });
        self.dedup_by(|lhs, rhs| lhs.canonical_cmp(rhs) == Ordering::Equal);
        debug_assert!(self.is_canonical());
    }
}

/// Type-safe wrapper for immutable, canonical data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    pub fn untie_replace(&mut self, canonical_src: T) -> T {
        debug_assert!(canonical_src.is_canonical());
        let Self(canonical_dest) = self;
        std::mem::replace(canonical_dest, canonical_src)
    }
}

impl<T> Canonical<Vec<T>>
where
    T: IsCanonical,
{
    #[must_use]
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
