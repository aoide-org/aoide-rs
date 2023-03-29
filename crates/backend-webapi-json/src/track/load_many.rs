// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::EntityUidTyped;
use aoide_core_json::{entity::EntityUid, track::Entity};

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::load::*;
}

pub type RequestBody = Vec<EntityUid>;

pub type ResponseBody = Vec<Entity>;

pub fn handle_request(
    connection: &mut DbConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let collector_config = EntityCollectorConfig {
        capacity: Some(request_body.len()),
        encode_gigtags: false,
    };
    let mut collector = EntityCollector::new(collector_config);
    connection.transaction::<_, Error, _>(|connection| {
        uc::load_many(
            connection,
            request_body.into_iter().map(EntityUidTyped::from_untyped),
            &mut collector,
        )
        .map_err(Into::into)
    })?;
    Ok(collector.into())
}
