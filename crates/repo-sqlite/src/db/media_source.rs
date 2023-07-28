// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::media::{artwork::ApicType, content::ContentPathKind};
use aoide_repo::{collection::RecordId as CollectionId, media::source::RecordHeader};
use diesel::sql_types::BigInt;
use strum::FromRepr;

use self::schema::*;
use crate::prelude::*;

pub(crate) fn encode_content_path_kind(value: ContentPathKind) -> i16 {
    value as _
}

pub(crate) fn decode_content_path_kind(value: i16) -> RepoResult<ContentPathKind> {
    u8::try_from(value)
        .ok()
        .and_then(ContentPathKind::from_repr)
        .ok_or_else(|| anyhow::anyhow!("invalid ContentPathKind value: {value}").into())
}

pub(crate) fn encode_apic_type(value: ApicType) -> i16 {
    value as _
}

pub(crate) fn decode_apic_type(value: i16) -> RepoResult<ApicType> {
    u8::try_from(value)
        .ok()
        .and_then(ApicType::from_repr)
        .ok_or_else(|| anyhow::anyhow!("invalid ApicType value: {value}").into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(i8)]
enum ArtworkSource {
    Irregular = -2,
    Unsupported = -1,
    Missing = 0,
    Embedded = 1,
    Linked = 2,
}

impl ArtworkSource {
    fn decode(value: i16) -> RepoResult<Self> {
        value
            .try_into()
            .ok()
            .and_then(Self::from_repr)
            .ok_or_else(|| anyhow::anyhow!("invalid ArtworkSource value: {value}").into())
    }

    const fn encode(self) -> i16 {
        self as _
    }
}

pub(crate) fn select_row_id_filtered_by_collection_id<'db>(
    collection_id: CollectionId,
) -> media_source::BoxedQuery<'db, DbBackend, BigInt> {
    media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed()
}

/// Filter by a path predicate.
///
/// URIs are only unambiguous within a collection. Therefore
/// filtering is restricted to a single collection.
pub(crate) fn select_row_id_filtered_by_content_path_predicate(
    collection_id: CollectionId,
    content_path_predicate: StringPredicate<'_>,
) -> media_source::BoxedQuery<'_, DbBackend, BigInt> {
    // Source URI filtering
    let statement = media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed();
    match content_path_predicate {
        StringPredicate::StartsWith(path_prefix_nocase) => {
            if path_prefix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_starts_with(&path_prefix_nocase)),
            )
        }
        StringPredicate::StartsNotWith(path_prefix_nocase) => {
            if path_prefix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path
                    .not_like(escape_like_starts_with(&path_prefix_nocase)),
            )
        }
        StringPredicate::EndsWith(path_suffix_nocase) => {
            if path_suffix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_ends_with(&path_suffix_nocase)),
            )
        }
        StringPredicate::EndsNotWith(path_suffix_nocase) => {
            if path_suffix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path
                    .not_like(escape_like_ends_with(&path_suffix_nocase)),
            )
        }
        StringPredicate::Contains(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_contains(&path_fragment_nocase)),
            )
        }
        StringPredicate::ContainsNot(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path
                    .not_like(escape_like_contains(&path_fragment_nocase)),
            )
        }
        StringPredicate::Matches(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_matches(&path_fragment_nocase)),
            )
        }
        StringPredicate::MatchesNot(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path
                    .not_like(escape_like_matches(&path_fragment_nocase)),
            )
        }
        StringPredicate::Prefix(path_prefix) => {
            if path_prefix.is_empty() {
                return statement;
            }
            statement.filter(sql_column_substr_prefix_eq(
                "media_source.content_link_path",
                &path_prefix,
            ))
        }
        StringPredicate::Equals(path) => statement.filter(media_source::content_link_path.eq(path)),
        StringPredicate::EqualsNot(path) => {
            statement.filter(media_source::content_link_path.ne(path))
        }
    }
}
