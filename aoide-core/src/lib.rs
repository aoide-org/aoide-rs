// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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
//
extern crate base64;

extern crate chrono;

extern crate log;

extern crate mime;

extern crate rand;

extern crate ring;

#[macro_use]
extern crate serde;

#[cfg(test)]
extern crate mime_guess;

#[cfg(test)]
extern crate serde_json;

///////////////////////////////////////////////////////////////////////
/// Public Modules
///////////////////////////////////////////////////////////////////////
//
pub mod audio;

pub mod domain;
