// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

use super::FetchResultPageRequest;

#[derive(Debug)]
pub enum Task {
    FetchResultPage {
        collection_uid: CollectionUid,
        request: FetchResultPageRequest,
    },
}

#[cfg(feature = "webapi-backend")]
mod webapi;
