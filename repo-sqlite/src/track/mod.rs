// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

mod models;
mod schema;
mod search;
pub mod util;

use self::{
    models::*,
    schema::*,
    search::{TrackSearchBoxedExpressionBuilder, TrackSearchQueryTransform},
    util::RepositoryHelper,
};

use crate::util::*;

use aoide_core::{
    entity::{
        EntityHeader, EntityRevision, EntityRevisionUpdateResult, EntityUid, EntityVersionNumber,
    },
    tag::{Facet, Label},
    track::{
        release::{ReleaseDate, YYYYMMDD},
        *,
    },
    util::clock::{TickInstant, TickType, Ticks},
};

use aoide_repo::{
    collection::TrackStats as CollectionTrackStats,
    entity::{EntityBodyData, EntityData, Repo as EntityRepo},
    tag::{
        AvgScoreCount, CountParams as TagCountParams, FacetCount, FacetCountParams,
        Filter as TagFilter, SortField as TagSortField,
    },
    track::*,
    *,
};

use diesel::dsl::*;

///////////////////////////////////////////////////////////////////////
// Repository
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Repository<'a> {
    connection: &'a diesel::SqliteConnection,
    helper: RepositoryHelper<'a>,
}

impl<'a> Repository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self {
            connection,
            helper: RepositoryHelper::new(connection),
        }
    }
}

impl<'a> EntityRepo for Repository<'a> {
    fn resolve_repo_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>> {
        tbl_track::table
            .select(tbl_track::id)
            .filter(tbl_track::uid.eq(uid.as_ref()))
            .first::<RepoId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}

fn select_track_ids_matching_tag_filter<'a, DB>(
    tag_filter: &'a TagFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        aux_track_tag::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = aux_track_tag::table
        .select(aux_track_tag::track_id)
        .into_boxed();

    // Filter facet(s)
    if let Some(ref facets) = tag_filter.facets {
        if facets.is_empty() {
            // unfaceted tags without a facet
            select = select.filter(aux_track_tag::facet_id.is_null());
        } else {
            // tags with any of the given facets
            let subselect = aux_tag_facet::table
                .select(aux_tag_facet::id)
                .filter(aux_tag_facet::facet.eq_any(facets));
            select = select.filter(aux_track_tag::facet_id.eq_any(subselect.nullable()));
        }
    }

    // Filter labels
    if let Some(ref label) = tag_filter.label {
        let (cmp, val, dir) = label.into();
        let either_eq_or_like = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => EitherEqualOrLike::Equal(val.to_owned()),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCompare::StartsWith => EitherEqualOrLike::Like(format!(
                "{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::EndsWith => EitherEqualOrLike::Like(format!(
                "%{}",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Contains => EitherEqualOrLike::Like(format!(
                "%{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Matches => {
                EitherEqualOrLike::Like(val.replace('\\', "\\\\").replace('%', "\\%"))
            }
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => {
                if dir {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.eq(eq));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                } else {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.ne(eq));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                }
            }
            EitherEqualOrLike::Like(like) => {
                if dir {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.like(like).escape('\\'));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                } else {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.not_like(like).escape('\\'));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                }
            }
        };
    }

    // Filter tag score
    if let Some(score) = tag_filter.score {
        select = match score {
            NumericPredicate::LessThan(value) => select.filter(aux_track_tag::score.lt(value)),
            NumericPredicate::GreaterOrEqual(value) => {
                select.filter(aux_track_tag::score.ge(value))
            }
            NumericPredicate::GreaterThan(value) => select.filter(aux_track_tag::score.gt(value)),
            NumericPredicate::LessOrEqual(value) => select.filter(aux_track_tag::score.le(value)),
            NumericPredicate::Equal(value) => {
                if let Some(value) = value {
                    select.filter(aux_track_tag::score.eq(value))
                } else {
                    select.filter(aux_track_tag::score.is_null())
                }
            }
            NumericPredicate::NotEqual(value) => {
                if let Some(value) = value {
                    select.filter(aux_track_tag::score.ne(value))
                } else {
                    select.filter(aux_track_tag::score.is_not_null())
                }
            }
        };
    }

    (select, tag_filter.modifier)
}

fn select_track_ids_matching_marker_filter<'a, DB>(
    marker_label_filter: &'a StringFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        aux_track_marker::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = aux_track_marker::table
        .select(aux_track_marker::track_id)
        .into_boxed();

    // Filter labels
    if let Some(ref label) = marker_label_filter.value {
        let (cmp, val, dir) = label.into();
        let either_eq_or_like = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => EitherEqualOrLike::Equal(val.to_owned()),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCompare::StartsWith => EitherEqualOrLike::Like(format!(
                "{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::EndsWith => EitherEqualOrLike::Like(format!(
                "%{}",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Contains => EitherEqualOrLike::Like(format!(
                "%{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Matches => {
                EitherEqualOrLike::Like(val.replace('\\', "\\\\").replace('%', "\\%"))
            }
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => {
                if dir {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.eq(eq));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                } else {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.ne(eq));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                }
            }
            EitherEqualOrLike::Like(like) => {
                if dir {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.like(like).escape('\\'));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                } else {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.not_like(like).escape('\\'));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                }
            }
        };
    }

    (select, marker_label_filter.modifier)
}

enum EitherEqualOrLike {
    Equal(String),
    Like(String),
}

impl<'a> Repo for Repository<'a> {
    fn insert_track(&self, entity: Entity, body_data: EntityBodyData) -> RepoResult<()> {
        {
            let (data_fmt, data_ver, data_blob) = body_data;
            let insertable = InsertableEntity::bind(&entity.hdr, data_fmt, data_ver, &data_blob);
            let query = diesel::insert_into(tbl_track::table).values(&insertable);
            query.execute(self.connection)?;
        }
        self.helper.after_entity_inserted(&entity)?;
        Ok(())
    }

    fn update_track(
        &self,
        entity: Entity,
        body_data: EntityBodyData,
    ) -> RepoResult<EntityRevisionUpdateResult> {
        let prev_rev = entity.hdr.rev;
        let next_rev = prev_rev.next();
        {
            let (data_fmt, data_ver, data_blob) = body_data;
            let updatable = UpdatableEntity::bind(&next_rev, data_fmt, data_ver, &data_blob);
            let target = tbl_track::table.filter(
                tbl_track::uid
                    .eq(entity.hdr.uid.as_ref())
                    .and(tbl_track::rev_no.eq(prev_rev.no as i64))
                    .and(tbl_track::rev_ts.eq((prev_rev.ts.0).0)),
            );
            let repo_id = self
                .helper
                .before_entity_updated_or_removed(&entity.hdr.uid)?;
            let query = diesel::update(target).set(&updatable);
            let rows_affected: usize = query.execute(self.connection)?;
            debug_assert!(rows_affected <= 1);
            if rows_affected < 1 {
                let row = tbl_track::table
                    .select((tbl_track::rev_no, tbl_track::rev_ts))
                    .filter(tbl_track::uid.eq(entity.hdr.uid.as_ref()))
                    .first::<(i64, TickType)>(self.connection)
                    .optional()?;
                if let Some(row) = row {
                    let rev = EntityRevision {
                        no: row.0 as EntityVersionNumber,
                        ts: TickInstant(Ticks(row.1)),
                    };
                    return Ok(EntityRevisionUpdateResult::Current(rev));
                } else {
                    return Ok(EntityRevisionUpdateResult::NotFound);
                }
            }
            self.helper.after_entity_updated(repo_id, &entity.body)?;
        }
        Ok(EntityRevisionUpdateResult::Updated(prev_rev, next_rev))
    }

    fn delete_track(&self, uid: &EntityUid) -> RepoResult<Option<()>> {
        let target = tbl_track::table.filter(tbl_track::uid.eq(uid.as_ref()));
        let query = diesel::delete(target);
        self.helper.before_entity_updated_or_removed(uid)?;
        let rows_affected: usize = query.execute(self.connection)?;
        debug_assert!(rows_affected <= 1);
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }

    fn load_track(&self, uid: &EntityUid) -> RepoResult<Option<EntityData>> {
        tbl_track::table
            .filter(tbl_track::uid.eq(uid.as_ref()))
            .first::<QueryableEntityData>(self.connection)
            .optional()
            .map(|o| o.map(Into::into))
            .map_err(Into::into)
    }

    fn load_tracks(&self, uids: &[EntityUid]) -> RepoResult<Vec<EntityData>> {
        tbl_track::table
            .filter(tbl_track::uid.eq_any(uids.iter().map(AsRef::as_ref)))
            .load::<QueryableEntityData>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn locate_tracks(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        locate_params: LocateParams,
    ) -> RepoResult<Vec<EntityData>> {
        // URI filter
        let (cmp, val, dir) = (&locate_params.media_uri).into();
        let either_eq_or_like = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => EitherEqualOrLike::Equal(val.to_owned()),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCompare::StartsWith => EitherEqualOrLike::Like(format!(
                "{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::EndsWith => EitherEqualOrLike::Like(format!(
                "%{}",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Contains => EitherEqualOrLike::Like(format!(
                "%{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Matches => {
                EitherEqualOrLike::Like(val.replace('\\', "\\\\").replace('%', "\\%"))
            }
        };

        let mut target = tbl_track::table
            .select(tbl_track::all_columns)
            .order_by(tbl_track::id) // preserve relative order of results
            .into_boxed();

        // A subselect has proven to be much more efficient than
        // joining the aux_track_location table for filtering by URI!
        let mut track_id_subselect = aux_track_location::table
            .select(aux_track_location::track_id)
            .into_boxed();
        // URI filtering
        track_id_subselect = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => {
                if dir {
                    track_id_subselect.filter(aux_track_location::uri.eq(eq))
                } else {
                    track_id_subselect.filter(aux_track_location::uri.ne(eq))
                }
            }
            EitherEqualOrLike::Like(like) => {
                if dir {
                    track_id_subselect.filter(aux_track_location::uri.like(like).escape('\\'))
                } else {
                    track_id_subselect.filter(aux_track_location::uri.not_like(like).escape('\\'))
                }
            }
        };
        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            track_id_subselect = track_id_subselect
                .filter(aux_track_location::collection_uid.eq(collection_uid.as_ref()));
        }
        target = if dir {
            target.filter(tbl_track::id.eq_any(track_id_subselect))
        } else {
            target.filter(tbl_track::id.ne_all(track_id_subselect))
        };

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableEntityData>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn search_tracks(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        search_params: SearchParams,
    ) -> RepoResult<Vec<EntityData>> {
        // TODO: Joins are very expensive and should only be used
        // when the results need to be ordered. For filtering
        // subselects have proven to be much more efficient.
        //
        // In general queries with joins are not suitable to be
        // executed efficiently as batch operations. Since search
        // operations are expected to be executed standalone the
        // joins are acceptable in this case.
        let mut target = tbl_track::table
            .select(tbl_track::all_columns)
            .distinct()
            .inner_join(aux_track_brief::table)
            .left_outer_join(aux_track_media::table)
            .left_outer_join(aux_track_collection::table)
            .into_boxed();

        if let Some(ref filter) = search_params.filter {
            target = target.filter(filter.build_expression(collection_uid));
        }

        // Collection filter
        if let Some(uid) = collection_uid {
            target = target.filter(aux_track_collection::collection_uid.eq(uid.as_ref()));
        };

        for sort_order in &search_params.ordering {
            target = sort_order.apply_to_query(target, collection_uid);
        }
        // Finally order by PK to preserve the relative order of results
        // even if no sorting was requested.
        target = target.then_order_by(tbl_track::id);

        // Pagination
        target = apply_pagination(target, pagination);

        let res = target.load::<QueryableEntityData>(self.connection)?;
        Ok(res.into_iter().map(Into::into).collect())
    }

    fn count_track_field_strings(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: Pagination,
    ) -> RepoResult<StringFieldCounts> {
        let track_id_subselect = collection_uid.map(|collection_uid| {
            aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()))
        });
        let rows = match field {
            StringField::MediaUri => {
                let mut target = aux_track_media::table
                    .select((
                        aux_track_media::uri.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_media::uri)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_media::uri)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_media::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::MediaUriDecoded => {
                let mut target = aux_track_media::table
                    .select((
                        aux_track_media::uri_decoded.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_media::uri_decoded)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_media::uri_decoded)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_media::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::MediaType => {
                let mut target = aux_track_media::table
                    .select((
                        aux_track_media::content_type.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_media::content_type)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_media::content_type)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_media::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackTitle => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::track_title.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::track_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::track_title)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackArtist => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::track_artist.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::track_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::track_artist)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackComposer => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::track_composer.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::track_composer)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::track_composer)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::AlbumTitle => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::album_title.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::album_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::album_title)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::AlbumArtist => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::album_artist.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::album_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::album_artist)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
        };
        let counts = rows
            .into_iter()
            .map(|row| {
                let value = row.0;
                debug_assert!(row.1 > 0);
                let total_count = row.1 as usize;
                StringCount { value, total_count }
            })
            .collect();
        Ok(StringFieldCounts { field, counts })
    }

    fn collect_collection_track_stats(
        &self,
        collection_uid: &EntityUid,
    ) -> RepoResult<CollectionTrackStats> {
        let total_count = aux_track_collection::table
            .select(diesel::dsl::count_star())
            .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()))
            .first::<i64>(self.connection)? as usize;

        Ok(CollectionTrackStats { total_count })
    }
}

impl<'a> Albums for Repository<'a> {
    fn count_tracks_by_album(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByAlbumParams,
        pagination: Pagination,
    ) -> RepoResult<Vec<AlbumCountResults>> {
        let mut target = aux_track_brief::table
            .select((
                aux_track_brief::album_title,
                aux_track_brief::album_artist,
                aux_track_brief::release_date,
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS count"),
            ))
            .group_by((
                aux_track_brief::album_title,
                aux_track_brief::album_artist,
                aux_track_brief::release_date,
            ))
            .into_boxed();

        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
        }

        target = target.filter(aux_track_brief::album_title.is_not_null());
        target = target.filter(aux_track_brief::album_artist.is_not_null());
        if let Some(min_release_date) = params.min_release_date {
            target =
                target.filter(aux_track_brief::release_date.ge(YYYYMMDD::from(min_release_date)));
        }
        if let Some(max_release_date) = params.max_release_date {
            target =
                target.filter(aux_track_brief::release_date.le(YYYYMMDD::from(max_release_date)));
        }

        for sort_order in &params.ordering {
            let direction = sort_order.direction;
            use SortDirection::*;
            use SortField::*;
            match sort_order.field {
                AlbumTitle => match direction {
                    Ascending => {
                        target = target.then_order_by(aux_track_brief::album_title.asc());
                    }
                    Descending => {
                        target = target.then_order_by(aux_track_brief::album_title.desc())
                    }
                },
                AlbumArtist => match direction {
                    Ascending => {
                        target = target.then_order_by(aux_track_brief::track_artist.asc());
                    }
                    Descending => {
                        target = target.then_order_by(aux_track_brief::album_artist.desc());
                    }
                },
                ReleaseDate => match direction {
                    Ascending => {
                        target = target.then_order_by(aux_track_brief::release_date.asc());
                    }
                    Descending => {
                        target = target.then_order_by(aux_track_brief::release_date.desc());
                    }
                },
                field => log::warn!(
                    "Ignoring sort order by field {:?} when counting tracks by albums",
                    field
                ),
            }
        }
        target = target.then_order_by(sql::<diesel::sql_types::BigInt>("count").desc());

        // Pagination
        target = apply_pagination(target, pagination);

        let res = target
            .load::<(Option<String>, Option<String>, Option<YYYYMMDD>, i64)>(self.connection)?;

        Ok(res
            .into_iter()
            .map(|row| AlbumCountResults {
                title: row.0,
                artist: row.1,
                release_date: row.2.map(ReleaseDate::new),
                total_count: row.3 as usize,
            })
            .collect())
    }
}

impl<'a> Tags for Repository<'a> {
    fn count_tracks_by_tag_facet(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &FacetCountParams,
        pagination: Pagination,
    ) -> RepoResult<Vec<FacetCount>> {
        let mut target = aux_track_tag::table
            .inner_join(aux_tag_facet::table)
            .select((
                aux_tag_facet::facet,
                sql::<diesel::sql_types::BigInt>("count(*) AS count"),
            ))
            .group_by(aux_track_tag::facet_id)
            .into_boxed();

        // Facet filtering
        if let Some(ref facets) = params.facets {
            target = target.filter(
                aux_tag_facet::facet.eq_any(facets.iter().map(AsRef::as_ref).map(String::as_str)),
            );
        }

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Ordering
        if params.ordering.is_empty() {
            target = target.then_order_by(sql::<diesel::sql_types::BigInt>("count").desc());
        } else {
            for sort_order in &params.ordering {
                let direction = sort_order.direction;
                use SortDirection::*;
                use TagSortField::*;
                match sort_order.field {
                    Facet => {
                        let col = aux_tag_facet::facet;
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.asc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Count => {
                        let col = sql::<diesel::sql_types::BigInt>("count");
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.desc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    field => log::warn!(
                        "Ignoring sort order by field {:?} when counting tracks by tag facet",
                        field
                    ),
                }
            }
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(String, i64)>(self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| FacetCount {
                facet: Facet::new(row.0),
                total_count: row.1 as usize,
            })
            .collect())
    }

    fn count_tracks_by_tag(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &TagCountParams,
        pagination: Pagination,
    ) -> RepoResult<Vec<AvgScoreCount>> {
        let mut target = aux_track_tag::table
            .left_outer_join(aux_tag_facet::table)
            .left_outer_join(aux_tag_label::table)
            .select((
                aux_tag_facet::facet.nullable(),
                aux_tag_label::label.nullable(),
                sql::<diesel::sql_types::Double>("AVG(score) AS avg_score"),
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS count"),
            ))
            .group_by((aux_track_tag::facet_id, aux_track_tag::label_id))
            .into_boxed();

        // Facet filtering
        if let Some(ref facets) = params.facets {
            let facets = facets.iter().map(AsRef::as_ref).map(String::as_str);
            target = if params.include_non_faceted_tags() {
                target.filter(
                    aux_tag_facet::facet
                        .eq_any(facets)
                        .or(aux_tag_facet::facet.is_null()),
                )
            } else {
                target.filter(aux_tag_facet::facet.eq_any(facets))
            };
        } else {
            // Include all faceted tags
            if !params.include_non_faceted_tags() {
                target = target.filter(aux_track_tag::facet_id.is_not_null());
            }
        }

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Ordering
        if params.ordering.is_empty() {
            target = target.then_order_by(sql::<diesel::sql_types::BigInt>("count").desc());
        } else {
            for sort_order in &params.ordering {
                let direction = sort_order.direction;
                use SortDirection::*;
                use TagSortField::*;
                match sort_order.field {
                    Facet => {
                        let col = aux_tag_facet::facet;
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.asc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Label => {
                        let col = aux_tag_label::label;
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.asc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Score => {
                        let col = sql::<diesel::sql_types::Double>("avg_score");
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.desc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Count => {
                        let col = sql::<diesel::sql_types::BigInt>("count");
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.desc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                }
            }
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(Option<String>, Option<String>, f64, i64)>(self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| AvgScoreCount {
                facet: row.0.map(Facet::new),
                label: row.1.map(Label::new),
                avg_score: row.2.into(),
                total_count: row.3 as usize,
            })
            .collect())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
