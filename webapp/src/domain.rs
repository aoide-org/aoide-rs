pub use aoide_core_ext_serde::collection::CollectionWithSummary;
pub use aoide_core_serde::{
    collection::{Collection, MediaSourceConfig},
    util::color::*,
};

pub type Collections = Vec<((String, u64), Collection)>;
pub type CollectionId = String;

pub enum CollectionData {
    Overview(Collection),
    WithSummary(CollectionWithSummary),
}
