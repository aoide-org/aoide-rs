// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::FetchResultPage;

#[derive(Debug)]
pub enum Task {
    FetchResultPage(FetchResultPage),
}

#[cfg(feature = "webapi-backend")]
mod webapi;
