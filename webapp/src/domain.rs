pub(crate) use aoide_core::collection::{Entity as CollectionEntity, EntityHeader};

pub(crate) use aoide_core_api::collection::Summary as CollectionSummary;

#[derive(Debug, Clone)]
pub(crate) struct CollectionItem {
    pub(crate) entity: CollectionEntity,
    pub(crate) summary: Option<CollectionSummary>,
}

impl CollectionItem {
    pub(crate) const fn without_summary(entity: CollectionEntity) -> Self {
        Self {
            entity,
            summary: None,
        }
    }
}

pub(crate) type CollectionItems = Vec<CollectionItem>;
