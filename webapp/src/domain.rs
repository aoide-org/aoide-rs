pub use aoide_core::{
    collection::{Entity as CollectionEntity, MediaSourceConfig},
    entity::EntityHeader,
    util::color::*,
};

pub use aoide_core_api::collection::Summary as CollectionSummary;

#[derive(Debug, Clone)]
pub struct CollectionItem {
    pub entity: CollectionEntity,
    pub summary: Option<CollectionSummary>,
}

impl CollectionItem {
    pub const fn without_summary(entity: CollectionEntity) -> Self {
        Self {
            entity,
            summary: None,
        }
    }
}

pub type CollectionItems = Vec<CollectionItem>;
