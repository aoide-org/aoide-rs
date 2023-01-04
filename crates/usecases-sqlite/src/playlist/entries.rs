// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::playlist::EntityHeader;

use aoide_core_api::playlist::EntityWithEntriesSummary;

use aoide_repo::playlist::RecordHeader;

use super::*;

pub fn patch(
    connection: &mut DbConnection,
    entity_header: &EntityHeader,
    operations: impl IntoIterator<Item = uc::playlist::entries::PatchOperation>,
) -> Result<(RecordHeader, EntityWithEntriesSummary)> {
    let mut repo = RepoConnection::new(connection);
    uc::playlist::entries::patch(&mut repo, entity_header, operations).map_err(Into::into)
}
