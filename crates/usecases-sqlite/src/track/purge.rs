// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo::collection::EntityRepo as _;

use super::*;

mod uc {
    pub(super) use aoide_usecases::track::purge::*;
}

pub fn purge_by_media_source_content_path_predicates(
    connection: &mut SqliteConnection,
    collection_uid: &CollectionUid,
    path_predicates: Vec<StringPredicate>,
) -> Result<uc::PurgeByMediaContentPathPredicatesSummary> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    uc::purge_by_media_source_content_path_predicates(&mut repo, collection_id, path_predicates)
        .map_err(Into::into)
}
