// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, TrackEntity};
use aoide_core_api::Pagination;
use aoide_repo::{track::RecordHeader, ReservableRecordCollector};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

mod uc {
    pub(super) use aoide_core_api::track::search::*;
    pub(super) use aoide_usecases::track::search::*;
}

pub fn search(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &uc::Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = TrackEntity>,
) -> Result<usize> {
    let mut repo = RepoConnection::new(connection);
    uc::search_with_params(&mut repo, collection_uid, params, pagination, collector)
        .map_err(Into::into)
}
