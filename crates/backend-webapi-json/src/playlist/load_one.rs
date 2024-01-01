// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::playlist::EntityWithEntries;
use aoide_usecases_sqlite::playlist as uc;

use super::*;

pub type ResponseBody = EntityWithEntries;

pub fn handle_request(
    connection: &mut DbConnection,
    entity_uid: &EntityUid,
) -> Result<ResponseBody> {
    connection.transaction::<_, Error, _>(|connection| {
        uc::load_one_with_entries(connection, entity_uid)
            .map(Into::into)
            .map_err(Into::into)
    })
}
