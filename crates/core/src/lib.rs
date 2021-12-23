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

///////////////////////////////////////////////////////////////////////

#![deny(missing_debug_implementations)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(rust_2018_idioms)]

// TODO: Move into `domain` submodule
pub mod audio;
pub mod collection;
pub mod entity;
pub mod media;
pub mod music;
pub mod playlist;
pub mod tag;
pub mod track;
pub mod util;

pub mod prelude {

    pub(crate) use crate::{
        entity::*,
        util::{clock::*, color::*, *},
    };

    pub(crate) use semval::prelude::*;
}

mod compat {
    use std::cmp::Ordering;

    // TODO: Remove after https://github.com/rust-lang/rust/issues/53485
    // has been stabilized.
    pub fn is_sorted_by<'a, T, F>(iterable: impl IntoIterator<Item = &'a T>, mut cmp: F) -> bool
    where
        F: FnMut(&'a T, &'a T) -> Ordering,
        T: 'a,
    {
        let mut iter = iterable.into_iter();
        if let Some(first) = iter.next() {
            let mut prev = first;
            for next in iter {
                if cmp(prev, next) == Ordering::Greater {
                    return false;
                }
                prev = next;
            }
        }
        true
    }
}
