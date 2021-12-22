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

use aoide_core::entity::EntityUid;

use aoide_core_json::track::Entity;

use super::*;

mod uc {
    pub use aoide_usecases_sqlite::track::load::*;
}

pub type ResponseBody = Entity;

pub fn handle_request(connection: &SqliteConnection, uid: &EntityUid) -> Result<ResponseBody> {
    uc::load_one(connection, uid)
        .map(Into::into)
        .map_err(Into::into)
}
