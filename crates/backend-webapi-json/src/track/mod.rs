// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    tag::FacetedTags,
    track::{tag::FACET_ID_GROUPING, EntityUid},
};
use aoide_core_json::track::Entity;
use aoide_media::fmt::encode_gig_tags;
use aoide_repo::{
    prelude::{RecordCollector, ReservableRecordCollector},
    track::RecordHeader,
};
use nonicle::Canonical;

use super::*;

mod _core {
    pub(super) use aoide_core::track::Entity;
}

pub mod export_metadata;
pub mod find_unsynchronized;
pub mod import_and_replace;
pub mod load_many;
pub mod load_one;
pub mod replace;
pub mod resolve;
pub mod search;

const DEFAULT_PAGINATION: Pagination = Pagination {
    limit: Some(100),
    offset: None,
};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TrackQueryParams {
    pub encode_gigtags: Option<bool>,
}

fn new_request_id() -> Uuid {
    Uuid::new_v4()
}

#[derive(Debug, Clone)]
pub struct EntityCollectorConfig {
    pub capacity: Option<usize>,

    pub encode_gigtags: bool,
}

#[derive(Debug)]
pub struct EntityCollector {
    encode_gigtags: bool,

    collected: Vec<Entity>,
}

impl EntityCollector {
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(config: EntityCollectorConfig) -> Self {
        let EntityCollectorConfig {
            capacity,
            encode_gigtags,
        } = config;
        let collected = if let Some(capacity) = capacity {
            Vec::with_capacity(capacity)
        } else {
            Vec::new()
        };
        Self {
            encode_gigtags,
            collected,
        }
    }
}

impl From<EntityCollector> for Vec<Entity> {
    fn from(from: EntityCollector) -> Self {
        let EntityCollector { collected, .. } = from;
        collected
    }
}

impl RecordCollector for EntityCollector {
    type Header = RecordHeader;
    type Record = _core::Entity;

    fn collect(&mut self, _record_header: RecordHeader, mut entity: _core::Entity) {
        let Self {
            collected,
            encode_gigtags,
        } = self;
        if *encode_gigtags {
            let mut tags = std::mem::take(&mut entity.body.track.tags).untie();
            let mut grouping_tags = tags
                .facets
                .iter()
                .enumerate()
                .find_map(|(index, faceted_tags)| {
                    (faceted_tags.facet_id == *FACET_ID_GROUPING).then_some(index)
                })
                .map(|index| tags.facets.remove(index).tags)
                .unwrap_or_default();
            let mut tags = Canonical::tie(tags);
            encode_gig_tags(&mut tags, &mut grouping_tags).expect("no error");
            let mut tags = tags.untie();
            tags.facets.push(FacetedTags {
                facet_id: FACET_ID_GROUPING.clone(),
                tags: grouping_tags,
            });
            entity.body.track.tags = Canonical::tie(tags);
        }
        collected.push(entity.into());
    }
}

impl ReservableRecordCollector for EntityCollector {
    fn reserve(&mut self, additional: usize) {
        let Self { collected, .. } = self;
        collected.reserve(additional);
    }
}
