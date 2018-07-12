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

///////////////////////////////////////////////////////////////////////
/// External Crates
///////////////////////////////////////////////////////////////////////
///
extern crate aoide_core;

extern crate aoide_storage;

extern crate actix;

extern crate actix_web;

extern crate clap;

extern crate diesel;

#[macro_use]
extern crate failure;

extern crate futures;

#[macro_use]
extern crate log;

extern crate mime;

extern crate r2d2;

#[macro_use]
extern crate serde;

extern crate serde_json;

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////
///
pub mod api;
