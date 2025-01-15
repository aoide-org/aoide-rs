// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::url::BaseUrl;

use crate::Pagination;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filtering {
    pub min_count: Option<usize>,
    pub max_count: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordering {
    CountAscending,
    CountDescending,
    ContentPathAscending,
    ContentPathDescending,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub filtering: Filtering,
    pub ordering: Option<Ordering>,
    pub pagination: Pagination,
}
