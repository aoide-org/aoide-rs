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
#![warn(rust_2018_idioms)]
// TODO: Remove after clippy fix has been released.
// https://github.com/rust-lang/rust-clippy/pull/6553
#![allow(clippy::field_reassign_with_default)]

pub mod prelude {
    pub use serde::{Deserialize, Serialize};

    pub(crate) use serde_repr::*;

    pub(crate) use crate::util::{clock::*, color::*};

    pub(crate) use schemars::JsonSchema;
}

pub mod audio;
pub mod collection;
pub mod entity;
pub mod media;
pub mod music;
pub mod playlist;
pub mod tag;
pub mod track;
pub mod util;
