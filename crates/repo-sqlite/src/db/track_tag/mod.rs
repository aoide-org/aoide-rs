// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::tag::*;
use aoide_repo::track::RecordId;

use crate::prelude::*;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub facet_id: Option<FacetId<'static>>,
    pub label: Option<Label<'static>>,
    pub score: Score,
}

impl From<Record> for (Option<FacetId<'static>>, PlainTag<'static>) {
    fn from(from: Record) -> Self {
        let Record {
            track_id: _,
            facet_id,
            label,
            score,
        } = from;
        let plain_tag = PlainTag { label, score };
        (facet_id, plain_tag)
    }
}
