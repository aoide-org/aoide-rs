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

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Min(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Max(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    Custom(&'static str)
}

#[derive(Clone, Debug)]
pub struct Error<T>
where
    T: Aspect,
{
    pub aspect: T,
    pub violation: Violation,
}

pub trait Aspect: Clone + Debug {}

impl<T> Aspect for T where T: Clone + Debug {}

impl<T> Error<T>
where
    T: Aspect,
{
    pub fn new(aspect: impl Into<T>, violation: impl Into<Violation>) -> Self {
        Self {
            aspect: aspect.into(),
            violation: violation.into(),
        }
    }

    pub fn map_aspect<F, U>(self, map: &F) -> Error<U>
    where
        F: Fn(T) -> U,
        U: Aspect,
    {
        Error {
            aspect: map(self.aspect),
            violation: self.violation,
        }
    }
}

impl<T> Display for Error<T>
where
    T: Aspect,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl<T> StdError for Error<T> where T: Aspect {}

#[derive(Clone, Debug)]
pub struct Errors<T>
where
    T: Aspect,
{
    errors: Vec<Error<T>>,
}

impl<T> Default for Errors<T>
where
    T: Aspect,
{
    fn default() -> Self {
        Self { errors: vec![] }
    }
}

pub type Result<T> = std::result::Result<(), Errors<T>>;

impl<T> Errors<T>
where
    T: Aspect,
{
    pub fn error(aspect: impl Into<T>, violation: impl Into<Violation>) -> Self {
        Self {
            errors: vec![Error::new(aspect, violation)],
        }
    }

    pub fn add_error(&mut self, aspect: impl Into<T>, violation: impl Into<Violation>) {
        self.errors.push(Error::new(aspect, violation));
    }

    pub fn merge_errors(&mut self, other: &mut Self) {
        self.errors.append(&mut other.errors);
    }

    pub fn merge_result(&mut self, res: Result<T>)
    {
        if let Err(mut other) = res {
            self.merge_errors(&mut other);
        }
    }

    pub fn map_and_merge_result<F, U>(&mut self, res: Result<U>, map: F)
    where
        F: Fn(U) -> T,
        U: Aspect,
    {
        if let Err(other) = res {
            self.errors.reserve(other.errors.len());
            for e in other.errors.into_iter() {
                self.errors.push(e.map_aspect(&map))
            }
        }
    }

    pub fn into_result(self) -> Result<T> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<T> AsRef<Vec<Error<T>>> for Errors<T>
where
    T: Aspect,
{
    fn as_ref(&self) -> &Vec<Error<T>> {
        &self.errors
    }
}

pub trait Validate {
    type Aspect: Aspect;

    fn validate(&self) -> Result<Self::Aspect>;

    fn error(
        aspect: impl Into<Self::Aspect>,
        violation: impl Into<Violation>,
    ) -> Error<Self::Aspect> {
        Error::new(aspect, violation)
    }
}
