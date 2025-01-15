// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::collection::LoadScope;
use aoide_core_api_json::collection::export_entity_with_summary;
use aoide_usecases_sqlite::collection as uc;

use super::*;

pub type ResponseBody = EntityWithSummary;

#[allow(clippy::needless_pass_by_value)] // consume arguments
pub fn handle_request(
    connection: &mut DbConnection,
    entity_uid: &CollectionUid,
) -> Result<EntityWithSummary> {
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::load_one(connection, entity_uid, LoadScope::EntityWithSummary).map_err(Into::into)
        })
        .map(|(_record_id, entity_with_summary)| export_entity_with_summary(entity_with_summary))
}
