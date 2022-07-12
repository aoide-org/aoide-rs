// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

mod uc {
    pub(super) use aoide_core_api::track::search::*;
    pub(super) use aoide_usecases::track::search::*;
}

pub fn search(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: uc::Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize> {
    let repo = RepoConnection::new(connection);
    uc::search_with_params(&repo, collection_uid, params, pagination, collector).map_err(Into::into)
}
