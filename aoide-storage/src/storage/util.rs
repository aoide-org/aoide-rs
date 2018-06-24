// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use diesel;
use diesel::prelude::*;

use api::Pagination;

pub(crate) fn apply_pagination<'a, ST, QS, DB>(
    source: diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>,
    pagination: &Pagination,
) -> diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>
where
    QS: diesel::query_source::QuerySource,
    DB: diesel::backend::Backend + diesel::sql_types::HasSqlType<ST> + 'a,
{
    let mut target = source;
    if let Some(offset) = pagination.offset {
        target = target.offset(offset as i64);
    };
    if let Some(limit) = pagination.limit {
        target = target.limit(limit as i64);
    };
    target
}
