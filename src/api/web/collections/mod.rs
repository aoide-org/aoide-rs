// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod _repo {
    pub use aoide_repo::collection::{MediaSourceSummary, PlaylistSummary, Summary, TrackSummary};
}

mod _core {
    pub use aoide_core::{collection::Entity, entity::EntityHeader};
}

use aoide_core::entity::EntityUid;

use aoide_repo::{
    collection::RecordHeader,
    prelude::{RecordCollector, ReservableRecordCollector},
};

use aoide_core_serde::{
    collection::{Collection, Entity},
    entity::Entity as GenericEntity,
};

///////////////////////////////////////////////////////////////////////

pub mod create;
pub mod delete;
pub mod load_all;
pub mod load_one;
pub mod update;

#[derive(Debug, Clone, Default)]
pub struct EntityCollector(Vec<EntityWithSummary>);

impl EntityCollector {
    pub const fn new(inner: Vec<EntityWithSummary>) -> Self {
        Self(inner)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let inner = Vec::with_capacity(capacity);
        Self(inner)
    }
}

impl From<EntityCollector> for Vec<EntityWithSummary> {
    fn from(from: EntityCollector) -> Self {
        let EntityCollector(inner) = from;
        inner
    }
}

impl RecordCollector for EntityCollector {
    type Header = RecordHeader;
    type Record = (_core::Entity, Option<_repo::Summary>);

    fn collect(
        &mut self,
        _header: RecordHeader,
        (entity, summary): (_core::Entity, Option<_repo::Summary>),
    ) {
        let Self(inner) = self;
        inner.push(merge_entity_with_summary(
            entity.into(),
            summary.map(Into::into),
        ));
    }
}

impl ReservableRecordCollector for EntityCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MediaSourceSummary {
    pub total_count: u64,
}

impl From<_repo::MediaSourceSummary> for MediaSourceSummary {
    fn from(from: _repo::MediaSourceSummary) -> Self {
        let _repo::MediaSourceSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistSummary {
    pub total_count: u64,
}

impl From<_repo::PlaylistSummary> for PlaylistSummary {
    fn from(from: _repo::PlaylistSummary) -> Self {
        let _repo::PlaylistSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSummary {
    pub total_count: u64,
}

impl From<_repo::TrackSummary> for TrackSummary {
    fn from(from: _repo::TrackSummary) -> Self {
        let _repo::TrackSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_sources: Option<MediaSourceSummary>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracks: Option<TrackSummary>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlists: Option<PlaylistSummary>,
}

impl From<_repo::Summary> for Summary {
    fn from(from: _repo::Summary) -> Self {
        let _repo::Summary {
            tracks,
            playlists,
            media_sources,
        } = from;
        Self {
            tracks: tracks.map(Into::into),
            playlists: playlists.map(Into::into),
            media_sources: media_sources.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionWithSummary {
    #[serde(flatten)]
    collection: Collection,

    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<Summary>,
}

pub type EntityWithSummary = GenericEntity<CollectionWithSummary>;

fn merge_entity_with_summary(entity: Entity, summary: Option<Summary>) -> EntityWithSummary {
    let GenericEntity(hdr, body) = entity;
    let body = CollectionWithSummary {
        collection: body,
        summary,
    };
    GenericEntity(hdr, body)
}
