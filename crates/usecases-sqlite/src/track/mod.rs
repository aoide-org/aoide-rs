// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::track::*;

use aoide_repo::{
    prelude::*,
    track::{EntityRepo as _, RecordHeader},
};

use super::*;

pub mod export_metadata;
pub mod find_unsynchronized;
pub mod import_and_replace;
pub mod load;
pub mod purge;
pub mod replace;
pub mod resolve;
pub mod search;
