// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::track::replace::Summary;

use aoide_usecases::track::ValidatedInput;

use super::*;

mod uc {
    pub(super) use aoide_usecases::track::replace::{
        replace_many_by_media_source_content_path, Params,
    };
}

pub fn replace_many_by_media_source_content_path(
    connection: &mut SqliteConnection,
    collection_uid: &CollectionUid,
    params: &uc::Params,
    validated_track_iter: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary> {
    let mut repo = RepoConnection::new(connection);
    uc::replace_many_by_media_source_content_path(
        &mut repo,
        collection_uid,
        params,
        validated_track_iter,
    )
    .map_err(Into::into)
}
