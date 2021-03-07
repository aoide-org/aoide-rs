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

use super::*;

use aoide_core::util::clock::DateTime;

use semval::Validate as _;

///////////////////////////////////////////////////////////////////////

pub fn update(
    connection: &SqliteConnection,
    updated_entity_with_current_rev: Entity,
) -> Result<Entity> {
    let (hdr, body) = updated_entity_with_current_rev.into();
    if let Err(err) = body.validate() {
        return Err(anyhow::anyhow!("Invalid collection: {:?}", err).into());
    }
    let EntityHeader {
        uid,
        rev: current_rev,
    } = hdr;
    let next_rev = current_rev.next();
    let next_hdr = EntityHeader { uid, rev: next_rev };
    let updated_entity_with_next_rev = Entity::new(next_hdr, body);
    let updated_at = DateTime::now_utc();
    let db = RepoConnection::new(connection);
    Ok(
        db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
            db.update_collection_entity_revision(
                &current_rev,
                updated_at,
                &updated_entity_with_next_rev,
            )?;
            Ok(updated_entity_with_next_rev)
        })?,
    )
}
