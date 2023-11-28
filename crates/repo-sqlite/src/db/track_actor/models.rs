// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{schema::*, *};
use crate::db::track::{decode_search_scope, encode_search_scope};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = track_actor, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: String,
    pub role: i16,
    pub role_notes: Option<String>,
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
            role,
            role_notes,
        } = from;
        let scope = decode_search_scope(scope)?;
        let kind = decode_kind(kind)?;
        let role = decode_role(role)?;
        let record = Record {
            track_id: track_id.into(),
            scope,
            actor: Actor {
                role,
                kind,
                name,
                role_notes,
            },
        };
        Ok((row_id.into(), record))
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = track_actor)]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub scope: i16,
    pub kind: i16,
    pub name: &'a str,
    pub role: i16,
    pub role_notes: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(track_id: RecordId, scope: Scope, actor: &'a Actor) -> Self {
        let Actor {
            kind,
            name,
            role,
            role_notes,
        } = actor;
        let scope = encode_search_scope(scope);
        let kind = encode_kind(*kind);
        let role = encode_role(*role);
        Self {
            track_id: track_id.into(),
            scope,
            kind,
            name: name.as_str(),
            role,
            role_notes: role_notes.as_ref().map(String::as_str),
        }
    }
}
