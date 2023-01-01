// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::url::BaseUrl;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
}
