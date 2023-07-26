// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::clock::YyyyMmDdDateValue;

use crate::prelude::*;

#[derive(Debug, Queryable)]
#[diesel(table_name = view_album)]
#[allow(unreachable_pub)] // TODO: Remove when used
pub struct QueryableRecord {
    pub phantom_id: RowId,
    pub artist: String,
    pub title: String,
    pub track_count: i64,
    pub track_id_concat: String,
    pub kind: Option<i16>,
    pub publisher: Option<String>,
    pub min_recorded_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub max_recorded_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub min_released_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub max_released_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub min_released_orig_at_yyyymmdd: Option<YyyyMmDdDateValue>,
    pub max_released_orig_at_yyyymmdd: Option<YyyyMmDdDateValue>,
}
