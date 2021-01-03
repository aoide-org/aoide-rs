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

use serde::Serialize;

use crate::usecases::json;

use aoide_core::{
    entity::EntityHeader,
    track::{Entity, Track},
};

use aoide_repo::entity::{EntityBodyData, EntityData, EntityDataVersion};

mod _serde {
    pub use aoide_core_serde::track::Track;
}

///////////////////////////////////////////////////////////////////////

const ENTITY_DATA_VERSION: EntityDataVersion = EntityDataVersion { major: 0, minor: 0 };

pub fn serialize_entity_body_data(track: &_serde::Track) -> Fallible<EntityBodyData> {
    json::serialize_entity_body_data(track, ENTITY_DATA_VERSION)
}

pub fn deserialize_entity_from_data(entity_data: EntityData) -> Fallible<Entity> {
    let (hdr, json_data) = load_entity_data(entity_data)?;
    let track: _serde::Track = serde_json::from_slice(&json_data)?;
    Ok(Entity::new(hdr, Track::from(track)))
}

pub fn load_entity_data(entity_data: EntityData) -> Fallible<(EntityHeader, Vec<u8>)> {
    json::load_entity_data(entity_data, ENTITY_DATA_VERSION)
}

pub fn load_entity_data_blob(entity_data: EntityData) -> Fallible<Vec<u8>> {
    json::load_entity_data_blob(entity_data, ENTITY_DATA_VERSION)
}

pub fn load_entity_data_array_blob(
    entity_data_iter: impl Iterator<Item = EntityData> + Clone,
) -> Fallible<Vec<u8>> {
    json::load_entity_data_array_blob(entity_data_iter, ENTITY_DATA_VERSION)
}

pub fn load_entity_data_ext_array_blob<T>(
    entity_data_ext_iter: impl Iterator<Item = EntityDataExt<Option<T>>> + Clone,
    estimated_ext_json_size_in_bytes: usize,
) -> Fallible<Vec<u8>>
where
    T: Serialize,
{
    json::load_entity_data_ext_array_blob(
        entity_data_ext_iter,
        ENTITY_DATA_VERSION,
        estimated_ext_json_size_in_bytes,
    )
}
