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

#![deny(missing_debug_implementations, missing_copy_implementations)]

use serde::{Deserialize, Serialize};

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////
pub mod prelude {
    pub use super::audio::{sample::*, signal::*, *};
    pub use super::entity::*;
    pub use super::time::*;
    pub use super::util::*;
}

pub mod audio;

pub mod collection;

pub mod entity;

pub mod metadata;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub mod music;

pub mod time;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub mod track;

pub mod util;

pub(crate) use util::*;
