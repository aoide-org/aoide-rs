// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = track_tag)]
pub struct QueryableRecord {
    pub id: RowId,
    pub track_id: RowId,
    pub facet: Option<String>,
    pub label: Option<String>,
    pub score: f64,
}

impl From<QueryableRecord> for (RecordId, Record) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            track_id,
            facet,
            label,
            score,
        } = from;
        let record = Record {
            track_id: track_id.into(),
            facet_id: facet.map(Cow::Owned).map(FacetId::new),
            label: label.map(Cow::Owned).map(Label::new),
            score: score.into(),
        };
        (id.into(), record)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = track_tag)]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub facet: Option<&'a str>,
    pub label: Option<&'a str>,
    pub score: f64,
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
