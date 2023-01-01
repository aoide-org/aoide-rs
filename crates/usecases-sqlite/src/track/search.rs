// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

mod uc {
    pub(super) use aoide_core_api::track::search::*;
    pub(super) use aoide_usecases::track::search::*;
}

pub fn search(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: uc::Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize> {
    let mut repo = RepoConnection::new(connection);
    uc::search_with_params(&mut repo, collection_uid, params, pagination, collector)
        .map_err(Into::into)
}
