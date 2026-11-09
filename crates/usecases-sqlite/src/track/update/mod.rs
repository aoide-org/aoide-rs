// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, track::EntityHeader};
use aoide_repo_sqlite::DbConnection;
use aoide_usecases::track::ValidatedInput;

use crate::{RepoConnection, Result};

use aoide_media_file::io::import::ImportTrackConfig;

pub use aoide_usecases::track::update::{Conflict, ExportParams, Outcome, Params, RevertOutcome};

mod uc {
    pub(super) use aoide_usecases::track::update::{export_one, revert_one, update_one};
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

pub fn export_one(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    expected_entity_hdr: &EntityHeader,
    export: ExportParams,
) -> Result<Outcome> {
    let mut repo = RepoConnection::new(connection);
    uc::export_one(&mut repo, collection_uid, expected_entity_hdr, export).map_err(Into::into)
}

pub fn revert_one(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    expected_entity_hdr: &EntityHeader,
    import_config: &ImportTrackConfig,
) -> Result<RevertOutcome> {
    let mut repo = RepoConnection::new(connection);
    uc::revert_one(
        &mut repo,
        collection_uid,
        expected_entity_hdr,
        import_config,
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests;
