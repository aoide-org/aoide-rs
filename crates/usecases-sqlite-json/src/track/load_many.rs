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

use aoide_core_json::{entity::EntityUid, track::Entity};
use aoide_usecases_sqlite::SqlitePooledConnection;

use super::*;

mod uc {
    pub use aoide_usecases_sqlite::track::load::*;
}

pub type RequestBody = Vec<EntityUid>;

pub type ResponseBody = Vec<Entity>;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let mut collector = EntityCollector::with_capacity(request_body.len());
    uc::load_many(
        &pooled_connection,
        request_body.into_iter().map(Into::into),
        &mut collector,
    )?;
    Ok(collector.into())
}
