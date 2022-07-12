// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use num_traits::FromPrimitive as _;

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track_actor"]
pub struct QueryableRecord {
    pub id: RowId,
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
            id,
            track_id,
            scope,
            kind,
            name,
            role,
            role_notes,
        } = from;
        let kind = ActorKind::from_i16(kind)
            .ok_or_else(|| anyhow::anyhow!("Invalid actor kind value: {kind}"))?;
        let role = ActorRole::from_i16(role)
            .ok_or_else(|| anyhow::anyhow!("Invalid actor role value: {role}"))?;
        let scope = Scope::from_i16(scope)
            .ok_or_else(|| anyhow::anyhow!("Invalid scope value: {scope}"))?;
        let record = Record {
            track_id: track_id.into(),
            scope,
            actor: Actor {
                kind,
                name,
                role,
                role_notes,
            },
        };
        Ok((id.into(), record))
    }
}

#[derive(Debug, Insertable)]
#[table_name = "track_actor"]
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
        Self {
            track_id: track_id.into(),
            scope: scope as i16,
            kind: *kind as i16,
            name: name.as_str(),
            role: *role as i16,
            role_notes: role_notes.as_ref().map(String::as_str),
        }
    }
}
