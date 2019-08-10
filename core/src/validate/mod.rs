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
}

#[derive(Clone, Debug)]
pub struct Error<T>
where
    T: Clone + Debug,
{
    pub aspect: T,
    pub violation: Violation,
}

impl<T> Error<T>
where
    T: Clone + Debug,
{
    pub fn new(aspect: impl Into<T>, violation: impl Into<Violation>) -> Self {
        Self {
            aspect: aspect.into(),
            violation: violation.into(),
        }
    }

    pub fn map_aspect<U>(self, aspect: U) -> Error<U>
    where
        U: Clone + Debug,
    {
        Error::new(aspect, self.violation)
    }
}

impl<T> Display for Error<T>
where
    T: Clone + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl<T> StdError for Error<T> where T: Clone + Debug {}

pub trait Validate {
    type Error;

    fn validate(&self) -> Result<(), Vec<Self::Error>>;
}
