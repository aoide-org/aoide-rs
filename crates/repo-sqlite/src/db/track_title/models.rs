// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{schema::*, *};
use crate::db::track::{decode_search_scope, encode_search_scope};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = track_title, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: String,
}

impl TryFrom<QueryableRecord> for (RecordId, Record) {
    type Error = anyhow::Error;

    fn try_from(from: QueryableRecord) -> anyhow::Result<Self> {
        let QueryableRecord {
            row_id,
            track_id,
            scope,
            kind,
            name,
        } = from;
        let scope = decode_search_scope(scope)?;
        let kind = decode_kind(kind)?;
        let record = Record {
            track_id: track_id.into(),
            scope,
            title: Title { kind, name },
        };
        Ok((row_id.into(), record))
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = track_title)]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: &'a str,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(track_id: RecordId, scope: Scope, title: &'a Title) -> Self {
        let Title { kind, name } = title;
        let scope = encode_search_scope(scope);
        let kind = encode_kind(*kind);
        Self {
            track_id: track_id.into(),
            scope,
            kind,
            name: name.as_str(),
        }
    }
}
