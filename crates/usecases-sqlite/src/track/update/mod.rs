// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, track::EntityHeader};
use aoide_repo_sqlite::DbConnection;
use aoide_usecases::track::ValidatedInput;

use crate::{RepoConnection, Result};

pub use aoide_usecases::track::update::{Conflict, ExportParams, Outcome, Params};

mod uc {
    pub(super) use aoide_usecases::track::update::update_one;
}

pub fn update_one(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    expected_entity_hdr: &EntityHeader,
    track: ValidatedInput,
    params: &Params,
) -> Result<Outcome> {
    let mut repo = RepoConnection::new(connection);
    uc::update_one(
        &mut repo,
        collection_uid,
        expected_entity_hdr,
        track,
        params,
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests;
