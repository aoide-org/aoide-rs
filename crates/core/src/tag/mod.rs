// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::{
        HashMap,
        hash_map::Entry::{self, Occupied, Vacant},
    },
    fmt,
    hash::{Hash, Hasher},
    iter::once,
    ops::Not as _,
};

use nonicle::{Canonical, CanonicalOrd, Canonicalize, CanonicalizeInto, IsCanonical};
use semval::prelude::*;

pub mod facet;
pub use facet::{FacetId, FacetIdInvalidity, Faceted};

pub mod label;
pub use label::{Label, LabelInvalidity, Labeled};

pub mod score;
pub use score::{Score, ScoreInvalidity, ScoreValue, Scored};

///////////////////////////////////////////////////////////////////////
// PlainTag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct PlainTag<'a> {
    pub label: Option<Label<'a>>,
    pub score: Score,
}

impl<'a> PlainTag<'a> {
    pub const DEFAULT_SCORE: Score = Score::DEFAULT;

    #[must_use]
    pub fn to_borrowed(&'a self) -> Self {
        let Self { label, score } = self;
        PlainTag {
            label: label.as_ref().map(Label::to_borrowed),
            score: *score,
        }
    }

    #[must_use]
    pub fn into_owned(self) -> PlainTag<'static> {
        let Self { label, score } = self;
        PlainTag {
            label: label.map(Label::into_owned),
            score,
        }
    }

    #[must_use]
    pub fn clone_owned(&self) -> PlainTag<'static> {
        self.to_borrowed().into_owned()
    }
}

impl Default for PlainTag<'_> {
    fn default() -> Self {
        Self {
            label: None,
            score: Self::DEFAULT_SCORE,
        }
    }
}

impl CanonicalOrd for PlainTag<'_> {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        let Self {
            label: lhs_label,
            score: _,
        } = self;
        let Self {
            label: rhs_label,
            score: _,
        } = other;
        match (lhs_label, rhs_label) {
            (Some(lhs_label), Some(rhs_label)) => lhs_label.cmp(rhs_label),
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }

    fn canonical_dedup_cmp(&self, other: &Self) -> Ordering {
        debug_assert_eq!(Ordering::Equal, self.canonical_cmp(other));
        // Reverse ordering by score, i.e. higher scores should precede lower scores
        debug_assert!(other.score.partial_cmp(&self.score).is_some());
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PlainTagInvalidity {
    Label(LabelInvalidity),
    Score(ScoreInvalidity),
}

impl Validate for PlainTag<'_> {
    type Invalidity = PlainTagInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { label, score } = self;
        ValidationContext::new()
            .validate_with(label, Self::Invalidity::Label)
            .validate_with(score, Self::Invalidity::Score)
            .into()
    }
}

impl Labeled for PlainTag<'_> {
    fn label(&self) -> Option<&Label<'_>> {
        self.label.as_ref()
    }
}

impl Scored for PlainTag<'_> {
    fn score(&self) -> Score {
        self.score
    }
}

impl IsCanonical for PlainTag<'_> {
    fn is_canonical(&self) -> bool {
        true
    }
}

impl Canonicalize for PlainTag<'_> {
    fn canonicalize(&mut self) {
        debug_assert!(self.is_canonical());
    }
}

///////////////////////////////////////////////////////////////////////
// FacetedTags
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct FacetedTags<'a> {
    pub facet_id: FacetId,
    pub tags: Vec<PlainTag<'a>>,
}

impl CanonicalOrd for FacetedTags<'_> {
    fn canonical_dedup_cmp(&self, other: &Self) -> Ordering {
        debug_assert_eq!(Ordering::Equal, self.canonical_cmp(other));
        // Conflicting tags should be resolved before, e.g. by concatenating
        // all plain tags that share the same facet_id. Here we just select the
        // faceted tag with more plain tag, i.e. reverse ordering of their
        // length.
        other.tags.len().cmp(&self.tags.len())
    }

    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.facet_id.canonical_cmp(&other.facet_id)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FacetedTagInvalidity {
    FacetId(FacetIdInvalidity),
    Tag(PlainTagInvalidity),
}

impl Validate for FacetedTags<'_> {
    type Invalidity = FacetedTagInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { facet_id, tags } = self;
        ValidationContext::new()
            .validate_with(facet_id, Self::Invalidity::FacetId)
            .merge_result(
                tags.iter()
                    .fold(ValidationContext::new(), |ctx, next| {
                        ctx.validate_with(next, Self::Invalidity::Tag)
                    })
                    .into(),
            )
            .into()
    }
}

impl Faceted for FacetedTags<'_> {
    fn facet(&self) -> Option<&FacetId> {
        Some(&self.facet_id)
    }
}

impl IsCanonical for FacetedTags<'_> {
    fn is_canonical(&self) -> bool {
        !self.tags.is_empty() && self.tags.is_canonical()
    }
}

impl Canonicalize for FacetedTags<'_> {
    fn canonicalize(&mut self) {
        self.tags.canonicalize();
    }
}

///////////////////////////////////////////////////////////////////////
// Tags
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Tags<'a> {
    pub plain: Vec<PlainTag<'a>>,
    pub facets: Vec<FacetedTags<'a>>,
}

impl<'a> Tags<'a> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        let Self { plain, facets } = self;
        facets
            .iter()
            .fold(plain.len(), |sum, faceted| sum + faceted.tags.len())
    }

    pub fn split_off_faceted_tags<'b, I>(
        &mut self,
        facet_ids: &I,
        facets_size_hint: usize,
    ) -> Vec<FacetedTags<'a>>
    where
        I: Iterator<Item = &'b FacetId> + Clone,
    {
        let mut facets = Vec::with_capacity(facets_size_hint);
        self.facets.retain_mut(|faceted_tags| {
            for facet_id in facet_ids.clone() {
                if *facet_id != faceted_tags.facet_id {
                    continue;
                }
                let tombstone = FacetedTags {
                    facet_id: FacetId::new_unchecked("".into()),
                    tags: Default::default(),
                };
                let faceted_tags = std::mem::replace(faceted_tags, tombstone);
                facets.push(faceted_tags);
                return false;
            }
            true
        });
        facets
    }
}

impl IsCanonical for Tags<'_> {
    fn is_canonical(&self) -> bool {
        let Self {
            plain: plain_tags,
            facets,
        } = self;
        plain_tags.is_canonical() && facets.is_canonical()
    }
}

impl Canonicalize for Tags<'_> {
    fn canonicalize(&mut self) {
        let Self {
            plain: plain_tags,
            facets,
        } = self;
        plain_tags.canonicalize();
        facets.retain(|f| f.tags.is_empty().not());
        facets.sort_unstable_by(|lhs, rhs| lhs.facet_id.canonical_cmp(&rhs.facet_id));
        facets.dedup_by(|next, prev| {
            if prev
                .facet_id
                .canonical_cmp(&next.facet_id)
                .then_with(|| prev.facet_id.canonical_dedup_cmp(&next.facet_id))
                != Ordering::Equal
            {
                return false;
            }
            // Join their tags
            prev.tags.append(&mut next.tags);
            true
        });
        for facet in &mut *facets {
            facet.canonicalize();
        }
        debug_assert!(facets.is_canonical());
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TagsInvalidity {
    FacetId(FacetIdInvalidity),
    PlainTag(PlainTagInvalidity),
    DuplicateFacets,
    DuplicateLabels,
}

fn check_for_duplicates_in_sorted_plain_tags_slice(
    plain_tags: &[PlainTag<'_>],
) -> Option<TagsInvalidity> {
    debug_assert!(plain_tags.is_sorted_by(|lhs, rhs| lhs.label.cmp(&rhs.label).is_le()));
    let mut iter = plain_tags.iter();
    if let Some(mut prev) = iter.next() {
        for next in iter {
            if prev.label == next.label {
                return Some(TagsInvalidity::DuplicateLabels);
            }
            prev = next;
        }
    }
    None
}

fn check_for_duplicates_in_sorted_faceted_tags_slice(
    faceted_tags: &[FacetedTags<'_>],
) -> Option<TagsInvalidity> {
    debug_assert!(faceted_tags.is_sorted_by(|lhs, rhs| lhs.facet_id.cmp(&rhs.facet_id).is_le()));
    let mut iter = faceted_tags.iter();
    if let Some(mut prev) = iter.next() {
        let duplicate_labels = check_for_duplicates_in_sorted_plain_tags_slice(&prev.tags);
        if duplicate_labels.is_some() {
            return duplicate_labels;
        }
        for next in iter {
            if prev.facet_id == next.facet_id {
                return Some(TagsInvalidity::DuplicateFacets);
            }
            prev = next;
            let duplicate_labels = check_for_duplicates_in_sorted_plain_tags_slice(&next.tags);
            if duplicate_labels.is_some() {
                return duplicate_labels;
            }
        }
    }
    None
}

impl Validate for Tags<'_> {
    type Invalidity = TagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            plain: plain_tags,
            facets,
        } = self;
        debug_assert!(self.is_canonical() || (plain_tags.is_empty() && facets.is_empty()));
        let mut context = ValidationContext::new()
            .validate_with(&plain_tags, Self::Invalidity::PlainTag)
            .merge_result(
                facets
                    .iter()
                    .fold(ValidationContext::new(), |ctx, faceted_tags| {
                        let FacetedTags { facet_id, tags } = faceted_tags;
                        ctx.validate_with(facet_id, Self::Invalidity::FacetId)
                            .validate_with(tags, Self::Invalidity::PlainTag)
                    })
                    .into(),
            );
        if let Some(duplicates) = check_for_duplicates_in_sorted_plain_tags_slice(plain_tags) {
            context = context.invalidate(duplicates);
        }
        if let Some(duplicates) = check_for_duplicates_in_sorted_faceted_tags_slice(facets) {
            context = context.invalidate(duplicates);
        }
        context.into()
    }
}

///////////////////////////////////////////////////////////////////////
// TagsMap
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug)]
pub struct FacetKey(Option<FacetId>);

impl FacetKey {
    #[must_use]
    pub const fn new(inner: Option<FacetId>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> Option<FacetId> {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        if let Self(Some(facet_id)) = self {
            facet_id.as_str()
        } else {
            debug_assert!(FacetId::clamp_from("").is_none());
            ""
        }
    }
}

impl From<FacetKey> for Option<FacetId> {
    fn from(from: FacetKey) -> Self {
        from.into_inner()
    }
}

impl From<Option<FacetId>> for FacetKey {
    fn from(from: Option<FacetId>) -> Self {
        FacetKey::new(from)
    }
}

impl From<FacetId> for FacetKey {
    fn from(from: FacetId) -> Self {
        Some(from).into()
    }
}

impl AsRef<Option<FacetId>> for FacetKey {
    fn as_ref(&self) -> &Option<FacetId> {
        let Self(inner) = self;
        inner
    }
}

impl Borrow<str> for FacetKey {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FacetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq for FacetKey {
    fn eq(&self, other: &Self) -> bool {
        let self_str: &str = self.as_str();
        let other_str: &str = other.as_str();
        self_str == other_str
    }
}

impl Eq for FacetKey {}

impl Ord for FacetKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_str: &str = self.as_str();
        let other_str: &str = other.as_str();
        self_str.cmp(other_str)
    }
}

impl PartialOrd for FacetKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for FacetKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_str: &str = self.as_str();
        self_str.hash(state);
    }
}

/// Unified map of both plain and faceted tags
pub type TagsMapInner<'a> = HashMap<FacetKey, Vec<PlainTag<'a>>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagsMap<'a>(TagsMapInner<'a>);

impl<'a> TagsMap<'a> {
    #[must_use]
    pub const fn new(inner: TagsMapInner<'a>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> TagsMapInner<'a> {
        let Self(inner) = self;
        inner
    }

    pub fn insert(&mut self, key: impl Into<FacetKey>, tag: PlainTag<'a>) {
        let Self(inner) = self;
        match inner.entry(key.into()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(tag);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![tag]);
            }
        }
    }

    pub fn insert_many(&mut self, key: impl Into<FacetKey>, mut tags: Vec<PlainTag<'a>>) {
        let Self(inner) = self;
        match inner.entry(key.into()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().append(&mut tags);
            }
            Entry::Vacant(entry) => {
                entry.insert(tags);
            }
        }
    }

    pub fn merge(&mut self, other: Self) {
        if self.is_empty() {
            // Replace the whole instance.
            *self = other;
            return;
        }
        for (key, tags) in other.into_inner() {
            self.insert_many(key, tags);
        }
    }

    pub fn count(&mut self, facet_key: &FacetKey) -> usize {
        let Self(inner) = self;
        inner.get(facet_key).map_or(0, Vec::len)
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        let Self(inner) = self;
        inner.values().fold(0, |sum, tags| sum + tags.len())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }
}

impl<'a> From<TagsMapInner<'a>> for TagsMap<'a> {
    fn from(from: TagsMapInner<'a>) -> Self {
        Self::new(from)
    }
}

impl<'a> From<TagsMap<'a>> for TagsMapInner<'a> {
    fn from(from: TagsMap<'a>) -> Self {
        from.into_inner()
    }
}

impl<'a> AsRef<TagsMapInner<'a>> for TagsMap<'a> {
    fn as_ref(&self) -> &TagsMapInner<'a> {
        let Self(inner) = self;
        inner
    }
}

impl<'a> TagsMap<'a> {
    pub fn get_tags(&'a self, facet_key: &FacetKey) -> Option<&'a [PlainTag<'a>]> {
        let Self(all_tags) = self;
        all_tags.get(facet_key).map(Vec::as_slice)
    }

    pub fn replace_tags(
        &mut self,
        facet_key: FacetKey,
        plain_tags: impl Into<Vec<PlainTag<'a>>>,
    ) -> Option<Vec<PlainTag<'a>>> {
        let Self(all_tags) = self;
        match all_tags.entry(facet_key) {
            Occupied(mut entry) => Some(entry.insert(plain_tags.into())),
            Vacant(entry) => {
                entry.insert(plain_tags.into());
                None
            }
        }
    }

    fn split_into_plain_and_faceted_tags(self) -> (Vec<PlainTag<'a>>, Self) {
        let Self(mut all_tags) = self;
        let plain_tags = all_tags.remove(&FacetKey::new(None)).unwrap_or_default();
        (plain_tags, Self(all_tags))
    }

    /// Update tags.
    ///
    /// Update the plain tags only if the ordering of labels differs. Otherwise keep
    /// the existing plain tags with their scores.
    ///
    /// This function is useful when importing tags from text fields where the
    /// an artificial score is generated depending on the ordering. In this case
    /// the original scores should be preserved.
    ///
    /// Returns `true` if the tags have been replaced and `false` if unmodified.
    pub fn update_tags_by_label_ordering(
        &mut self,
        facet_key: &FacetKey,
        new_plain_tags: impl Into<Vec<PlainTag<'a>>>,
    ) -> bool {
        let new_plain_tags = new_plain_tags.into();
        if let Some(old_plain_tags) = self.get_tags(facet_key) {
            if old_plain_tags.len() == new_plain_tags.len() {
                let mut unchanged = true;
                for (old_tag, new_tag) in old_plain_tags.iter().zip(new_plain_tags.iter()) {
                    if old_tag.label != new_tag.label {
                        unchanged = false;
                        break;
                    }
                }
                if !unchanged {
                    // No update desired if ordering of labels didn't change
                    return false;
                }
            }
        }
        if new_plain_tags.is_empty() {
            self.remove_tags(facet_key);
        } else {
            self.replace_tags(facet_key.clone(), new_plain_tags);
        }
        true
    }

    #[expect(clippy::missing_panics_doc)] // Never panics
    pub fn take_faceted_tags(&mut self, facet_key: &FacetKey) -> Option<FacetedTags<'a>> {
        if matches!(facet_key, FacetKey(None)) {
            return None;
        };
        let Self(all_tags) = self;
        all_tags.remove_entry(facet_key).map(|(key, tags)| {
            let FacetKey(facet_id) = key;
            debug_assert!(facet_id.is_some());
            let facet_id = facet_id.expect("facet");
            FacetedTags { facet_id, tags }
        })
    }

    pub fn remove_tags(&mut self, facet_key: &FacetKey) -> Option<usize> {
        let Self(all_tags) = self;
        all_tags.remove(facet_key).map(|tags| tags.len())
    }

    pub fn facet_keys(&self) -> impl Iterator<Item = &FacetKey> {
        self.0.keys()
    }
}

impl<'a> From<Tags<'a>> for TagsMap<'a> {
    fn from(from: Tags<'a>) -> Self {
        let Tags {
            plain: plain_tags,
            facets,
        } = from;
        let plain_iter = once((FacetKey::new(None), plain_tags));
        let faceted_iter = facets.into_iter().map(|faceted_tags| {
            let FacetedTags { facet_id, tags } = faceted_tags;
            (facet_id.into(), tags)
        });
        Self::new(plain_iter.chain(faceted_iter).collect())
    }
}

impl<'a> CanonicalizeInto<Tags<'a>> for TagsMap<'a> {
    fn canonicalize_into(self) -> Canonical<Tags<'a>> {
        let (plain_tags, faceted_tags) = self.split_into_plain_and_faceted_tags();
        let TagsMap(faceted_tags) = faceted_tags;
        let facets = faceted_tags
            .into_iter()
            .map(|(key, tags)| {
                let FacetKey(facet_id) = key;
                debug_assert!(facet_id.is_some());
                let facet_id = facet_id.expect("facet");
                FacetedTags { facet_id, tags }
            })
            .collect();
        let tags = Tags {
            plain: plain_tags,
            facets,
        };
        tags.canonicalize_into()
    }
}

#[cfg(test)]
mod tests;
