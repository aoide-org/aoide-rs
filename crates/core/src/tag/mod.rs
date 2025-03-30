// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    hash::{BuildHasher as _, Hash, Hasher},
    ops::Not as _,
};

use hashbrown::{DefaultHashBuilder, HashTable, hash_table::Entry};
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

#[derive(Clone, Debug)]
pub struct FacetKey(Option<FacetId>);

impl FacetKey {
    /// Without a facet.
    #[must_use]
    pub const fn unfaceted() -> Self {
        Self(None)
    }

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

impl From<&FacetKey> for FacetKey {
    fn from(from: &FacetKey) -> Self {
        // Cloning a FacetKey is cheap.
        from.clone()
    }
}

impl From<Option<&FacetId>> for FacetKey {
    fn from(from: Option<&FacetId>) -> Self {
        // Cloning a FacetId is cheap.
        from.cloned().into()
    }
}

impl From<&FacetId> for FacetKey {
    fn from(from: &FacetId) -> Self {
        // Cloning a FacetId is cheap.
        from.clone().into()
    }
}

impl AsRef<Option<FacetId>> for FacetKey {
    fn as_ref(&self) -> &Option<FacetId> {
        let Self(inner) = self;
        inner
    }
}

impl AsRef<str> for FacetKey {
    fn as_ref(&self) -> &str {
        self.as_str()
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
        self.as_str().eq(other.as_str())
    }
}

impl Eq for FacetKey {}

impl Ord for FacetKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for FacetKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for FacetKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

#[derive(Debug, Clone)]
enum InsertOrReplaceTags<'a> {
    Insert(Vec<PlainTag<'a>>),
    Replace(Vec<PlainTag<'a>>),
}

/// Unified map of both plain and faceted tags.
///
/// May contain duplicate tags if not _canonicalized_.
#[derive(Debug, Default, Clone)]
pub struct TagsMap<'a> {
    hash_builder: DefaultHashBuilder,
    hash_table: HashTable<(FacetKey, Vec<PlainTag<'a>>)>,
}

impl<'a> TagsMap<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hash_builder: Default::default(),
            hash_table: HashTable::new(),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            hash_builder: Default::default(),
            hash_table: HashTable::with_capacity(capacity),
        }
    }

    #[must_use]
    pub fn find<'b>(
        &'b self,
        facet_key: &impl AsRef<str>,
    ) -> Option<(&'b FacetKey, &'b [PlainTag<'a>])> {
        let Self {
            hash_builder,
            hash_table,
        } = self;
        let hash = hash_builder.hash_one(facet_key.as_ref());
        hash_table
            .find(hash, |(occupied_key, _)| {
                occupied_key.as_str().eq(facet_key.as_ref())
            })
            .map(|(facet_key, tags)| (facet_key, tags.as_slice()))
    }

    #[must_use]
    pub fn count(&self, facet_key: &impl AsRef<str>) -> Option<usize> {
        self.find(facet_key).map(|(_, tags)| tags.len())
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        self.hash_table
            .iter()
            .fold(0, |total_count, (_, tags)| total_count + tags.len())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }

    /// The [`FacetId`]s of all faceted tags.
    pub fn facet_ids(&self) -> impl Iterator<Item = &FacetId> {
        self.hash_table
            .iter()
            .filter_map(|(FacetKey(facet_id), _)| facet_id.as_ref())
    }

    /// Insert a single tag for the given facet.
    pub fn insert_one(
        &mut self,
        facet_key: impl AsRef<str> + Into<FacetKey>,
        one_tag: PlainTag<'a>,
    ) -> &FacetKey {
        self.insert(facet_key, vec![one_tag])
    }

    /// Insert multiple tags for the given facet.
    ///
    /// Returns the [`FacetKey`].
    pub fn insert(
        &mut self,
        facet_key: impl AsRef<str> + Into<FacetKey>,
        insert_tags: Vec<PlainTag<'a>>,
    ) -> &FacetKey {
        let (facet_key, replaced_tags) =
            self.insert_or_replace(facet_key, InsertOrReplaceTags::Insert(insert_tags));
        debug_assert!(replaced_tags.is_none());
        facet_key
    }

    /// Replace all tags of the given facet.
    ///
    /// Returns the [`FacetKey`] and the replaced tags. The replaced tags are `None`
    /// if no entry existed yet.
    pub fn replace(
        &mut self,
        facet_key: impl AsRef<str> + Into<FacetKey>,
        replace_tags: Vec<PlainTag<'a>>,
    ) -> (&FacetKey, Option<Vec<PlainTag<'a>>>) {
        self.insert_or_replace(facet_key, InsertOrReplaceTags::Replace(replace_tags))
    }

    /// Insert or replace tags.
    ///
    /// Returns the [`FacetKey`] and the corresponding replaced tags.
    fn insert_or_replace(
        &mut self,
        facet_key: impl AsRef<str> + Into<FacetKey>,
        insert_or_replace_tags: InsertOrReplaceTags<'a>,
    ) -> (&FacetKey, Option<Vec<PlainTag<'a>>>) {
        let Self {
            hash_builder,
            hash_table,
        } = self;
        let hash = hash_builder.hash_one(facet_key.as_ref());
        match hash_table.entry(
            hash,
            |(occupied_key, _)| occupied_key.as_str().eq(facet_key.as_ref()),
            |(facet_key, _)| hash_builder.hash_one(facet_key),
        ) {
            Entry::Occupied(mut occupied_entry) => {
                let tags = &mut occupied_entry.get_mut().1;
                let replaced_tags = match insert_or_replace_tags {
                    InsertOrReplaceTags::Insert(mut insert_tags) => {
                        tags.append(&mut insert_tags);
                        None
                    }
                    InsertOrReplaceTags::Replace(replace_tags) => {
                        let replaced_tags = std::mem::replace(tags, replace_tags);
                        Some(replaced_tags)
                    }
                };
                let occupied_key = &occupied_entry.into_mut().0;
                debug_assert_eq!(
                    *occupied_key,
                    facet_key.into(),
                    "converting the unused argument must result in the same key"
                );
                (occupied_key, replaced_tags)
            }
            Entry::Vacant(vacant) => {
                let facet_key = facet_key.into();
                let tags = match insert_or_replace_tags {
                    InsertOrReplaceTags::Insert(tags) | InsertOrReplaceTags::Replace(tags) => tags,
                };
                let occupied_entry = vacant.insert((facet_key, tags));
                let occupied_key = &occupied_entry.into_mut().0;
                (occupied_key, None)
            }
        }
    }

    /// Remove tags.
    ///
    /// Returns the [`FacetKey`] and the corresponding removed tags.
    pub fn remove(&mut self, facet_key: &impl AsRef<str>) -> Option<(FacetKey, Vec<PlainTag<'a>>)> {
        let Self {
            hash_builder,
            hash_table,
        } = self;
        let hash = hash_builder.hash_one(facet_key.as_ref());
        match hash_table.find_entry(hash, |(occupied_key, _)| {
            occupied_key.as_str().eq(facet_key.as_ref())
        }) {
            Ok(occupied) => Some(occupied.remove().0),
            Err(_absent) => None,
        }
    }

    pub fn merge(&mut self, other: Self) {
        if self.is_empty() {
            // Replace the whole instance.
            *self = other;
        } else {
            // TODO: Optimize?
            for (facet_key, tags) in other.hash_table {
                self.insert(facet_key, tags);
            }
        }
    }

    #[must_use]
    fn split_into_plain_and_faceted_tags(self) -> (Vec<PlainTag<'a>>, Self) {
        let Self {
            hash_builder,
            mut hash_table,
        } = self;
        let plain_key = FacetKey::unfaceted();
        let plain_hash = hash_builder.hash_one(&plain_key);
        let plain_tags = hash_table
            .find_entry(plain_hash, |(facet_key, _)| facet_key.eq(&plain_key))
            .map(|entry| entry.remove().0.1)
            .ok()
            .unwrap_or_default();
        (
            plain_tags,
            Self {
                hash_builder,
                hash_table,
            },
        )
    }

    #[must_use]
    pub fn all_tag_labels_equal<'b>(
        &'b self,
        facet_key: &impl AsRef<str>,
        new_plain_tags: &[PlainTag<'a>],
    ) -> Option<(&'b FacetKey, bool)> {
        let (facet_key, old_plain_tags) = self.find(&facet_key)?;
        let all_tag_labels_equal = {
            if old_plain_tags.len() == new_plain_tags.len() {
                old_plain_tags
                    .iter()
                    .zip(new_plain_tags.iter())
                    .all(|(old_tag, new_tag)| old_tag.label == new_tag.label)
            } else {
                false
            }
        };
        Some((facet_key, all_tag_labels_equal))
    }
}

impl<'a> From<Tags<'a>> for TagsMap<'a> {
    fn from(from: Tags<'a>) -> Self {
        let Tags {
            plain: plain_tags,
            facets,
        } = from;
        let mut into = Self::with_capacity(1 + facets.len());
        into.insert(FacetKey::unfaceted(), plain_tags);
        for FacetedTags { facet_id, tags } in facets {
            into.insert(facet_id, tags);
        }
        into
    }
}

impl<'a> FromIterator<(FacetKey, Vec<PlainTag<'a>>)> for TagsMap<'a> {
    fn from_iter<T: IntoIterator<Item = (FacetKey, Vec<PlainTag<'a>>)>>(iter: T) -> Self {
        let mut into = Self::new();
        for (facet_key, tags) in iter {
            into.insert(facet_key, tags);
        }
        into
    }
}

impl<'a> CanonicalizeInto<Tags<'a>> for TagsMap<'a> {
    fn canonicalize_into(self) -> Canonical<Tags<'a>> {
        let (plain_tags, faceted_tags) = self.split_into_plain_and_faceted_tags();
        let TagsMap {
            hash_builder: _,
            hash_table,
        } = faceted_tags;
        let facets = hash_table
            .into_iter()
            .map(|(facet_key, tags)| {
                let FacetKey(facet_id) = facet_key;
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
