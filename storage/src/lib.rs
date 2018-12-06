// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

// missing_debug_implementations
#![deny(missing_copy_implementations)]

///////////////////////////////////////////////////////////////////////
/// External Crates
///////////////////////////////////////////////////////////////////////
//
extern crate aoide_core;

#[macro_use]
extern crate diesel;

extern crate chrono;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;

extern crate mime;

extern crate percent_encoding;

extern crate rmp_serde;

#[macro_use]
extern crate serde;

extern crate serde_cbor;

extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate diesel_migrations;

///////////////////////////////////////////////////////////////////////
/// Public Modules
///////////////////////////////////////////////////////////////////////
//
#[cfg_attr(feature = "cargo-clippy", allow(proc_macro_derive_resolution_fallback))]
pub mod storage;

pub mod api;