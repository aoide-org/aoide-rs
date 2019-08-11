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

///////////////////////////////////////////////////////////////////////

use std::fmt::{self, Debug, Display};
use std::{any::Any, error::Error as StdError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Min(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Max(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
//#[non_exhaustive]
pub enum Violation {
    // Cardinality
    Missing,
    TooFew(Min),
    TooMany(Max),

    // Length
    Empty,
    TooShort(Min),
    TooLong(Max),

    OutOfBounds,

    Invalid,

    // Custom code for extensibility
    Custom(&'static str),
}

#[derive(Clone, Debug)]
pub struct Error<T>
where
    T: Validation,
{
    /// The validation context.
    pub validation: T,

    /// The actual cause of this error.
    pub violation: Violation,
}

/// A `Validation` defines the context for validating certain objectives.
/// These types are typically an `enum`s with one variant per objective.
/// Some of these variants may recursively wrap dependent validations to
/// trace back the root cause of a validation error.
///
/// For an anonymous or innermost context use the unit type `()`,
/// e.g. when validating non-composite types without the need for
/// any distinctive objectives.
pub trait Validation: Any + Debug {}

impl<T> Validation for T where T: Any + Debug {}

impl<T> Error<T>
where
    T: Validation,
{
    pub fn new(validation: impl Into<T>, violation: impl Into<Violation>) -> Self {
        Self {
            validation: validation.into(),
            violation: violation.into(),
        }
    }

    pub fn map_validation<F, U>(self, map: &F) -> Error<U>
    where
        F: Fn(T) -> U,
        U: Validation,
    {
        Error {
            validation: map(self.validation),
            violation: self.violation,
        }
    }
}

impl<T> Display for Error<T>
where
    T: Validation,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl<T> StdError for Error<T> where T: Validation {}

/// A collection of validation errors.
#[derive(Clone, Debug)]
pub struct Errors<T>
where
    T: Validation,
{
    errors: Vec<Error<T>>,
}

impl<T> Default for Errors<T>
where
    T: Validation,
{
    fn default() -> Self {
        Self { errors: vec![] }
    }
}

impl<T> StdError for Errors<T> where T: Validation {}

impl<T> Display for Errors<T>
where
    T: Validation,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<(), Errors<T>>;

impl<T> Errors<T>
where
    T: Validation,
{
    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Count the number of errors in this collection.
    pub fn count(&self) -> usize {
        self.errors.len()
    }

    /// Create a new collection with a single error.
    pub fn error(validation: impl Into<T>, violation: impl Into<Violation>) -> Self {
        Self {
            errors: vec![Error::new(validation, violation)],
        }
    }

    /// Add a new error to this collection.
    pub fn add_error(&mut self, validation: impl Into<T>, violation: impl Into<Violation>) {
        self.errors.push(Error::new(validation, violation));
    }

    /// Merge and clear another collection of errors into this collection.
    pub fn merge_errors(&mut self, other: &mut Self) {
        self.errors.append(&mut other.errors);
    }

    /// Merge a validation result into this collection.
    pub fn merge_result(&mut self, res: Result<T>) {
        if let Err(mut other) = res {
            self.merge_errors(&mut other);
        }
    }

    /// Merge an incompatible validation result into this collection,
    /// thereby mapping the validation types.
    pub fn map_and_merge_result<F, U>(&mut self, res: Result<U>, map: F)
    where
        F: Fn(U) -> T,
        U: Validation,
    {
        if let Err(other) = res {
            self.errors.reserve(other.errors.len());
            for e in other.errors.into_iter() {
                self.errors.push(e.map_validation(&map))
            }
        }
    }

    /// Convert this collection into a validation result.
    pub fn into_result(self) -> Result<T> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<T> IntoIterator for Errors<T>
where
    T: Validation,
{
    type Item = Error<T>;
    type IntoIter = std::vec::IntoIter<Error<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<T> AsRef<Vec<Error<T>>> for Errors<T>
where
    T: Validation,
{
    fn as_ref(&self) -> &Vec<Error<T>> {
        &self.errors
    }
}

/// A trait for validating types. Validation is expected to be an expensive
/// operation that should only be invoked when crossing boundaries.
pub trait Validate<T>
where
    T: Validation,
{
    /// Validate this instance.
    fn validate(&self) -> Result<T>;

    fn error(validation: impl Into<T>, violation: impl Into<Violation>) -> Error<T> {
        Error::new(validation, violation)
    }
}
