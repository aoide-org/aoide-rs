// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::prelude::*;
use semval::prelude::*;

use aoide_core::tag::{FacetId, Label, PlainTag, Score, ScoreValue};

use crate::RowId;

use super::{schema::*, Record, RecordId};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = track_tag, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub track_id: RowId,
    pub facet: Option<String>,
    pub label: Option<String>,
    pub score: ScoreValue,
}

impl From<QueryableRecord> for (RecordId, Record) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            row_id,
            track_id,
            facet,
            label,
            score,
        } = from;
        let facet_id = facet.map(FacetId::from_unchecked);
        debug_assert!(facet_id.as_ref().map_or(true, FacetId::is_valid));
        let label = label.map(Label::from_unchecked);
        debug_assert!(label.as_ref().map_or(true, Label::is_valid));
        let score = Score::new_unchecked(score);
        debug_assert!(score.is_valid());
        let record = Record {
            track_id: track_id.into(),
            facet_id,
            label,
            score,
        };
        (row_id.into(), record)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = track_tag)]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub facet: Option<&'a str>,
    pub label: Option<&'a str>,
    pub score: ScoreValue,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(
        track_id: RecordId,
        facet_id: Option<&'a FacetId<'a>>,
        plain_tag: &'a PlainTag<'a>,
    ) -> Self {
        let PlainTag { label, score } = plain_tag;
        Self {
            track_id: track_id.into(),
            facet: facet_id.map(FacetId::as_str),
            label: label.as_ref().map(Label::as_str),
            score: score.value(),
        }
    }
}
