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

use aoide_usecases_sqlite::{collection::purge as uc, SqlitePooledConnection};

use super::*;

pub type ResponseBody = ();

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    uid: &EntityUid,
) -> Result<ResponseBody> {
    uc::purge(&pooled_connection, uid).map_err(Into::into)
}