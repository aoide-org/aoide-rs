// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::{entity::EntityUid, track::Entity};

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::load::*;
}

pub type RequestBody = Vec<EntityUid>;

pub type ResponseBody = Vec<Entity>;

pub fn handle_request(
    connection: &mut SqliteConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let mut collector = EntityCollector::with_capacity(request_body.len());
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection| {
    uc::load_many(
            connection,
            request_body.into_iter().map(Into::into),
            &mut collector,
        )
        //.map_err(Into::into)})
    ?;
    Ok(collector.into())
}
