// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    fmt,
    hash::{Hash, Hasher},
    iter::once,
    ops::Not as _,
};

use nonicle::{Canonical, CanonicalOrd, Canonicalize, CanonicalizeInto, IsCanonical};

use crate::{compat::is_sorted_by, prelude::*};

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
    #[must_use]
    pub const fn default_score() -> Score {
        Score::max()
    }

    #[must_use]
    pub fn as_borrowed(&'a self) -> Self {
        let Self { label, score } = self;
        PlainTag {
            label: label.as_ref().map(Label::as_borrowed),
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
}

impl Default for PlainTag<'_> {
    fn default() -> Self {
        Self {
            label: None,
            score: Self::default_score(),
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
    pub facet_id: FacetId<'a>,
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
    fn facet(&self) -> Option<&FacetId<'_>> {
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
        for facet in facets.iter_mut() {
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
    debug_assert!(is_sorted_by(plain_tags, |lhs, rhs| lhs
        .label
        .cmp(&rhs.label)));
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
    debug_assert!(is_sorted_by(faceted_tags, |lhs, rhs| lhs
        .facet_id
        .cmp(&rhs.facet_id)));
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
pub struct FacetKey<'a>(Option<FacetId<'a>>);

impl<'a> FacetKey<'a> {
    #[must_use]
    pub const fn new(inner: Option<FacetId<'a>>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> Option<FacetId<'a>> {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn as_borrowed(&'a self) -> Self {
        let Self(inner) = self;
        FacetKey(inner.as_ref().map(FacetId::as_borrowed))
    }

    #[must_use]
    pub fn into_owned(self) -> FacetKey<'static> {
        let Self(inner) = self;
        FacetKey(inner.map(FacetId::into_owned))
    }

    #[must_use]
    pub fn as_str(&'a self) -> &'a str {
        let Self(inner) = self;
        match inner {
            Some(facet_id) => facet_id.as_str(),
            None => "",
        }
    }
}

impl<'a> From<FacetKey<'a>> for Option<FacetId<'a>> {
    fn from(from: FacetKey<'a>) -> Self {
        from.into_inner()
    }
}

impl<'a> From<Option<FacetId<'a>>> for FacetKey<'a> {
    fn from(from: Option<FacetId<'a>>) -> Self {
        FacetKey::new(from)
    }
}

impl<'a> From<FacetId<'a>> for FacetKey<'a> {
    fn from(from: FacetId<'a>) -> Self {
        Some(from).into()
    }
}

impl<'a> From<&'a FacetId<'a>> for FacetKey<'a> {
    fn from(from: &'a FacetId<'a>) -> Self {
        Some(from).into()
    }
}

impl<'a> From<Option<&'a FacetId<'a>>> for FacetKey<'a> {
    fn from(from: Option<&'a FacetId<'a>>) -> Self {
        FacetKey::new(from.map(FacetId::as_borrowed))
    }
}

impl<'a> AsRef<Option<FacetId<'a>>> for FacetKey<'a> {
    fn as_ref(&self) -> &Option<FacetId<'a>> {
        let Self(inner) = self;
        inner
    }
}

impl Borrow<str> for FacetKey<'_> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FacetKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for FacetKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        let self_str: &str = self.as_str();
        let other_str: &str = other.as_str();
        self_str == other_str
    }
}

impl Eq for FacetKey<'_> {}

impl Ord for FacetKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_str: &str = self.as_str();
        let other_str: &str = other.as_str();
        self_str.cmp(other_str)
    }
}

impl PartialOrd for FacetKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for FacetKey<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_str: &str = self.as_str();
        self_str.hash(state);
    }
}

/// Unified map of both plain and faceted tags
pub type TagsMapInner<'a> = HashMap<FacetKey<'a>, Vec<PlainTag<'a>>>;

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

    pub fn insert(&mut self, key: impl Into<FacetKey<'a>>, tag: PlainTag<'a>) {
        use std::collections::hash_map::*;
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

    pub fn count(&mut self, facet_id: &FacetId<'a>) -> usize {
        let Self(inner) = self;
        inner.get(&FacetKey::from(facet_id)).map_or(0, Vec::len)
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        let Self(inner) = self;
        inner.values().fold(0, |sum, tags| sum + tags.len())
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
    pub fn get_plain_tags(&'a self) -> Option<&'a [PlainTag<'a>]> {
        let Self(all_tags) = self;
        all_tags.get(&FacetKey::new(None)).map(Vec::as_slice)
    }

    fn split_into_plain_and_faceted_tags(self) -> (Vec<PlainTag<'a>>, Self) {
        let Self(mut all_tags) = self;
        let plain_tags = all_tags.remove(&FacetKey::new(None)).unwrap_or_default();
        (plain_tags, Self(all_tags))
    }

    pub fn get_faceted_plain_tags(
        &'a self,
        facet_id: &'a FacetId<'_>,
    ) -> Option<&'a [PlainTag<'a>]> {
        let Self(all_tags) = self;
        all_tags.get(&FacetKey::from(facet_id)).map(Vec::as_slice)
    }

    pub fn replace_faceted_plain_tags(
        &mut self,
        facet_id: FacetId<'a>,
        plain_tags: impl Into<Vec<PlainTag<'a>>>,
    ) -> Option<Vec<PlainTag<'a>>> {
        let Self(all_tags) = self;
        match all_tags.entry(FacetKey::new(Some(facet_id))) {
            Occupied(mut entry) => Some(entry.insert(plain_tags.into())),
            Vacant(entry) => {
                entry.insert(plain_tags.into());
                None
            }
        }
    }

    /// Update faceted plain tags
    ///
    /// Update the plain tags only if the ordering of labels differs. Otherwise keep
    /// the existing plain tags with their scores.
    ///
    /// This function is useful when importing tags from text fields where the
    /// an artificial score is generated depending on the ordering. In this case
    /// the original scores should be preserved.
    ///
    /// Returns `true` if the tags have been replaced and `false` if unmodified.
    pub fn update_faceted_plain_tags_by_label_ordering<'b>(
        &mut self,
        facet_id: &'b FacetId<'b>,
        plain_tags: impl Into<Vec<PlainTag<'a>>>,
    ) -> bool {
        let plain_tags = plain_tags.into();
        if let Some(faceted_plain_tags) = self.get_faceted_plain_tags(facet_id) {
            if faceted_plain_tags.len() == plain_tags.len() {
                let mut unchanged = true;
                for (old_tag, new_tag) in faceted_plain_tags.iter().zip(plain_tags.iter()) {
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
        if plain_tags.is_empty() {
            self.remove_faceted_tags(facet_id);
        } else {
            let facet_id = facet_id.clone().into_owned();
            self.replace_faceted_plain_tags(facet_id, plain_tags);
        }
        true
    }

    pub fn take_faceted_tags<'b>(&mut self, facet_id: &'b FacetId<'b>) -> Option<FacetedTags<'a>> {
        // TODO: How to avoid this needless allocation?
        let facet_id = facet_id.as_borrowed().into_owned();
        let Self(all_tags) = self;
        all_tags
            .remove_entry(&FacetKey::from(facet_id))
            .map(|(key, tags)| {
                let FacetKey(facet_id) = key;
                debug_assert!(facet_id.is_some());
                let facet_id = facet_id.expect("facet");
                FacetedTags { facet_id, tags }
            })
    }

    pub fn remove_faceted_tags<'b>(&mut self, facet_id: &'b FacetId<'b>) -> Option<usize> {
        // TODO: How to avoid this needless allocation?
        let facet_id = facet_id.clone().into_owned();
        let Self(all_tags) = self;
        all_tags
            .remove(&FacetKey::from(facet_id))
            .map(|tags| tags.len())
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
