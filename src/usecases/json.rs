// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub use aoide_core::entity::EntityHeader;

use aoide_repo::entity::{EntityBodyData, EntityData, EntityDataFormat, EntityDataVersion};

mod _serde {
    pub use aoide_core_serde::entity::EntityHeader;
}

use serde::Serialize;
use std::io::Write;

///////////////////////////////////////////////////////////////////////

const ENTITY_DATA_FORMAT: EntityDataFormat = EntityDataFormat::JSON;

pub fn serialize_entity_body_data<E: Serialize>(
    entity: &E,
    data_ver: EntityDataVersion,
) -> Fallible<EntityBodyData> {
    Ok((ENTITY_DATA_FORMAT, data_ver, serde_json::to_vec(entity)?))
}

pub fn load_entity_data(
    entity_data: EntityData,
    expected_data_ver: EntityDataVersion,
) -> Fallible<(EntityHeader, Vec<u8>)> {
    let (hdr, (data_fmt, data_ver, json_data)) = entity_data;
    if data_fmt != ENTITY_DATA_FORMAT {
        let e = anyhow!(
            "Unsupported data format when loading entity {}: expected = {:?}, actual = {:?}",
            hdr.uid,
            ENTITY_DATA_FORMAT,
            data_fmt
        );
        return Err(e);
    }
    if data_ver < expected_data_ver {
        // TODO: Data migration from an older version
        unimplemented!();
    }
    if data_ver == expected_data_ver {
        return Ok((hdr, json_data));
    }
    let e = anyhow!(
        "Unsupported data version when loading entity {}: expected = {:?}, actual = {:?}",
        hdr.uid,
        expected_data_ver,
        data_ver
    );
    Err(e)
}

fn load_and_write_entity_data(
    mut json_writer: &mut impl Write,
    entity_data: EntityData,
    expected_data_ver: EntityDataVersion,
) -> Fallible<()> {
    let (hdr, json_data) = load_entity_data(entity_data, expected_data_ver)?;
    json_writer.write_all(b"[")?;
    serde_json::to_writer(&mut json_writer, &_serde::EntityHeader::from(hdr))?;
    json_writer.write_all(b",")?;
    json_writer.write_all(&json_data)?;
    json_writer.write_all(b"]")?;
    Ok(())
}

fn entity_data_blob_size(entity_data: &EntityData) -> usize {
    let uid_bytes = 33;
    let rev_ver_bytes = ((entity_data.0).rev.ver as f64).log10().ceil() as usize;
    let rev_ts_bytes = 16;
    // ["<uid>",[<rev.ver>,<rev.ts>]]
    (entity_data.1).2.len() + uid_bytes + rev_ver_bytes + rev_ts_bytes + 8
}

pub fn load_entity_data_blob(
    entity_data: EntityData,
    expected_data_ver: EntityDataVersion,
) -> Fallible<Vec<u8>> {
    let mut json_writer = Vec::with_capacity(entity_data_blob_size(&entity_data));
    load_and_write_entity_data(&mut json_writer, entity_data, expected_data_ver)?;
    Ok(json_writer)
}

pub fn load_entity_data_array_blob(
    entity_data_iter: impl Iterator<Item = EntityData> + Clone,
    expected_data_ver: EntityDataVersion,
) -> Fallible<Vec<u8>> {
    let mut json_writer = Vec::with_capacity(entity_data_iter.clone().fold(
        /*closing bracket*/ 1,
        |acc, ref entity_data| {
            acc + entity_data_blob_size(&entity_data) + /*opening bracket or comma*/ 1
        },
    ));
    json_writer.write_all(b"[")?;
    for (i, entity_data) in entity_data_iter.enumerate() {
        if i > 0 {
            json_writer.write_all(b",")?;
        }
        load_and_write_entity_data(&mut json_writer, entity_data, expected_data_ver)?;
    }
    json_writer.write_all(b"]")?;
    json_writer.flush()?;
    Ok(json_writer)
}
