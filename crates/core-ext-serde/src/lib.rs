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

#![deny(missing_debug_implementations)]
#![deny(clippy::clone_on_ref_ptr)]
#![warn(rust_2018_idioms)]

pub use aoide_core_ext as _inner;

// Common imports
mod prelude {
    pub use serde::{Deserialize, Serialize};
}

pub mod collection;
pub mod filtering;
pub mod media;
pub mod sorting;
pub mod tag;
pub mod track;
