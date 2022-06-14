// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    fmt,
    hash::{Hash, Hasher},
    iter::once,
    ops::Deref,
};

use crate::{
    compat::is_sorted_by,
    prelude::*,
    util::{
        canonical::{CanonicalOrd, Canonicalize, IsCanonical},
        string::trimmed_non_empty_from,
    },
};

///////////////////////////////////////////////////////////////////////
// Score
///////////////////////////////////////////////////////////////////////

pub type ScoreValue = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Score(ScoreValue);

impl Score {
    #[must_use]
    pub const fn min_value() -> ScoreValue {
        0.0
    }

    #[must_use]
    pub const fn max_value() -> ScoreValue {
        1.0
    }

    #[must_use]
    pub const fn default_value() -> ScoreValue {
        Self::max_value()
    }

    pub fn clamp_value(value: impl Into<ScoreValue>) -> ScoreValue {
        value.into().clamp(Self::min_value(), Self::max_value())
    }

    pub fn clamp_from(value: impl Into<ScoreValue>) -> Self {
        Self::clamp_value(value).into()
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(Self::min_value())
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(Self::max_value())
    }

    #[must_use]
    pub const fn default() -> Self {
        Self(Self::default_value())
    }

    #[must_use]
    pub const fn new(inner: ScoreValue) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn value(self) -> ScoreValue {
        let Self(value) = self;
        value
    }

    // Convert to percentage value with a single decimal digit
    #[must_use]
    pub fn to_percentage(self) -> ScoreValue {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
    #[must_use]
    pub fn to_permille(self) -> u16 {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() as u16
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScoreInvalidity {
    OutOfRange,
}

impl Validate for Score {
    type Invalidity = ScoreInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::min() && *self <= Self::max()),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl From<Score> for ScoreValue {
    fn from(from: Score) -> Self {
        from.value()
    }
}

impl From<ScoreValue> for Score {
    fn from(value: ScoreValue) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_assert!(self.validate().is_ok());
        write!(f, "{:.1}%", self.to_percentage())
    }
}

pub trait Scored {
    fn score(&self) -> Score;
}

impl Scored for Score {
    fn score(&self) -> Self {
        *self
    }
}

///////////////////////////////////////////////////////////////////////
// Label
///////////////////////////////////////////////////////////////////////

pub type LabelValue = String;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CowLabel<'a>(Cow<'a, str>);

impl<'a> From<CowLabel<'a>> for Label {
    fn from(from: CowLabel<'a>) -> Self {
        Self(from.0.into())
    }
}

impl<'a> From<&'a Label> for CowLabel<'a> {
    fn from(from: &'a Label) -> Self {
        Self(from.0.as_str().into())
    }
}

impl<'a> AsRef<Cow<'a, str>> for CowLabel<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl<'a> Deref for CowLabel<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<str> for CowLabel<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

/// The name of a tag.
///
/// Format: Unicode string without leading/trailing whitespace
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(LabelValue);

impl Label {
    pub fn clamp_value<'a>(value: impl Into<Cow<'a, str>>) -> Option<CowLabel<'a>> {
        trimmed_non_empty_from(value).map(CowLabel)
    }

    pub fn clamp_from<'a>(value: impl Into<Cow<'a, str>>) -> Option<Self> {
        Self::clamp_value(value).map(Into::into)
    }

    #[must_use]
    pub const fn new(value: LabelValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub fn into_value(self) -> LabelValue {
        let Self(value) = self;
        value
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LabelInvalidity {
    Empty,
    Format,
}

impl Validate for Label {
    type Invalidity = LabelInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.value().is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                Self::clamp_value(self.value().as_str()) != Some(self.into()),
                Self::Invalidity::Format,
            )
            .into()
    }
}

impl From<LabelValue> for Label {
    fn from(value: LabelValue) -> Self {
        Self::new(value)
    }
}

impl From<Label> for LabelValue {
    fn from(from: Label) -> Self {
        from.into_value()
    }
}

impl AsRef<LabelValue> for Label {
    fn as_ref(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }
}

impl Deref for Label {
    type Target = LabelValue;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

pub trait Labeled {
    fn label(&self) -> Option<&Label>;
}

impl Labeled for Label {
    fn label(&self) -> Option<&Self> {
        Some(self)
    }
}

///////////////////////////////////////////////////////////////////////
// Facet
///////////////////////////////////////////////////////////////////////

pub type FacetIdValue = String;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CowFacetId<'a>(Cow<'a, str>);

impl<'a> From<CowFacetId<'a>> for FacetId {
    fn from(from: CowFacetId<'a>) -> Self {
        Self(from.0.into())
    }
}

impl<'a> From<&'a FacetId> for CowFacetId<'a> {
    fn from(from: &'a FacetId) -> Self {
        Self(from.0.as_str().into())
    }
}

impl<'a> AsRef<Cow<'a, str>> for CowFacetId<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl<'a> Deref for CowFacetId<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<str> for CowFacetId<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

/// An identifier for referencing tag categories.
///
/// Facets are used for grouping/categorizing and providing context or meaning.
///
/// Serves as a symbolic, internal identifier that is not intended to be displayed
/// literally in the UI. The restrictive naming constraints ensure that they are
/// not used for storing arbitrary text. Instead facet identifiers should be mapped
/// to translated display strings, e.g. the facet "genre" could be mapped to "Genre"
/// in English and the facet "venue" could be mapped to "Veranstaltungsort" in German.
///
/// Value constraints:
///   - charset/alphabet: `+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz`
///   - no leading/trailing/inner whitespace
///
/// Rationale for the value constraints:
///   - Facet identifiers are intended to be created, shared, and parsed worldwide
///   - The Lingua franca of IT is English
///   - ASCII characters can be encoded by a single byte in UTF-8
///
/// References:
///   - <https://en.wikipedia.org/wiki/Faceted_classification>
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FacetId(FacetIdValue);

/// The alphabet of facet identifiers
///
/// All valid characters, ordered by their ASCII codes.
pub const FACET_ID_ALPHABET: &str = "+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz";

impl FacetId {
    pub fn clamp_value<'a>(value: impl Into<Cow<'a, str>>) -> Option<CowFacetId<'a>> {
        let mut value: String = value.into().into();
        value.retain(Self::is_valid_char);
        if value.is_empty() {
            None
        } else {
            Some(CowFacetId(Cow::Owned(value)))
        }
    }

    pub fn clamp_from<'a>(value: impl Into<Cow<'a, str>>) -> Option<Self> {
        Self::clamp_value(value).map(Into::into)
    }

    #[must_use]
    pub const fn new(value: FacetIdValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn into_value(self) -> FacetIdValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub const fn value(&self) -> &FacetIdValue {
        let Self(value) = self;
        value
    }

    fn is_valid_char(c: char) -> bool {
        // TODO: Use regex?
        if !c.is_ascii() || c.is_ascii_whitespace() || c.is_ascii_uppercase() {
            return false;
        }
        if c.is_ascii_alphanumeric() {
            return true;
        }
        "+-./@[]_".contains(c)
    }

    fn is_invalid_char(c: char) -> bool {
        !Self::is_valid_char(c)
    }
}

impl CanonicalOrd for FacetId {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FacetIdInvalidity {
    Empty,
    Format,
}

impl Validate for FacetId {
    type Invalidity = FacetIdInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.value().is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                self.value()
                    .chars()
                    .next()
                    .map_or(false, |c| !c.is_ascii_lowercase()),
                Self::Invalidity::Format,
            )
            .invalidate_if(
                self.value().chars().any(FacetId::is_invalid_char),
                Self::Invalidity::Format,
            )
            .into()
    }
}

impl From<FacetIdValue> for FacetId {
    fn from(from: FacetIdValue) -> Self {
        Self::new(from)
    }
}

impl From<FacetId> for FacetIdValue {
    fn from(from: FacetId) -> Self {
        from.into_value()
    }
}

impl AsRef<FacetIdValue> for FacetId {
    fn as_ref(&self) -> &FacetIdValue {
        let Self(value) = self;
        value
    }
}

impl Deref for FacetId {
    type Target = FacetIdValue;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<str> for FacetId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FacetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value())
    }
}

pub trait Faceted {
    fn facet(&self) -> Option<&FacetId>;
}

impl Faceted for FacetId {
    fn facet(&self) -> Option<&Self> {
        Some(self)
    }
}

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
}

impl From<Option<FacetId>> for FacetKey {
    fn from(inner: Option<FacetId>) -> Self {
        FacetKey::new(inner)
    }
}

impl From<FacetKey> for Option<FacetId> {
    fn from(from: FacetKey) -> Self {
        from.into_inner()
    }
}

impl From<FacetId> for FacetKey {
    fn from(from: FacetId) -> Self {
        FacetKey(Some(from))
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
        let Self(inner) = self;
        match inner {
            Some(facet_id) => facet_id.as_ref(),
            None => "",
        }
    }
}

impl Borrow<str> for FacetKey {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for FacetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl PartialEq for FacetKey {
    fn eq(&self, other: &Self) -> bool {
        let self_str: &str = self.as_ref();
        let other_str: &str = other.as_ref();
        self_str == other_str
    }
}

impl Eq for FacetKey {}

impl Ord for FacetKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_str: &str = self.as_ref();
        let other_str: &str = other.as_ref();
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
        let self_str: &str = self.as_ref();
        self_str.hash(state);
    }
}

///////////////////////////////////////////////////////////////////////
// PlainTag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct PlainTag {
    pub label: Option<Label>,
    pub score: Score,
}

impl CanonicalOrd for PlainTag {
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

impl Validate for PlainTag {
    type Invalidity = PlainTagInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { label, score } = self;
        ValidationContext::new()
            .validate_with(label, Self::Invalidity::Label)
            .validate_with(score, Self::Invalidity::Score)
            .into()
    }
}

impl PlainTag {
    #[must_use]
    pub const fn default_score() -> Score {
        Score::max()
    }
}

impl Default for PlainTag {
    fn default() -> Self {
        Self {
            label: None,
            score: Self::default_score(),
        }
    }
}

impl Labeled for PlainTag {
    fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }
}

impl Scored for PlainTag {
    fn score(&self) -> Score {
        self.score
    }
}

impl IsCanonical for PlainTag {
    fn is_canonical(&self) -> bool {
        true
    }
}

impl Canonicalize for PlainTag {
    fn canonicalize(&mut self) {
        debug_assert!(self.is_canonical());
    }
}

///////////////////////////////////////////////////////////////////////
// FacetedTags
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct FacetedTags {
    pub facet_id: FacetId,
    pub tags: Vec<PlainTag>,
}

impl CanonicalOrd for FacetedTags {
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

impl Validate for FacetedTags {
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

impl Faceted for FacetedTags {
    fn facet(&self) -> Option<&FacetId> {
        Some(&self.facet_id)
    }
}

impl IsCanonical for FacetedTags {
    fn is_canonical(&self) -> bool {
        !self.tags.is_empty() && self.tags.is_canonical()
    }
}

impl Canonicalize for FacetedTags {
    fn canonicalize(&mut self) {
        self.tags.canonicalize();
    }
}

///////////////////////////////////////////////////////////////////////
// Tags
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Tags {
    pub plain: Vec<PlainTag>,
    pub facets: Vec<FacetedTags>,
}

impl Tags {
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

impl IsCanonical for Tags {
    fn is_canonical(&self) -> bool {
        let Self {
            plain: plain_tags,
            facets,
        } = self;
        plain_tags.is_canonical() && facets.is_canonical()
    }
}

impl Canonicalize for Tags {
    fn canonicalize(&mut self) {
        let Self {
            plain: plain_tags,
            facets,
        } = self;
        plain_tags.canonicalize();
        facets.retain(|f| !f.tags.is_empty());
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
    plain_tags: &[PlainTag],
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
    faceted_tags: &[FacetedTags],
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

impl Validate for Tags {
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

/// Unified map of both plain and faceted tags
pub type TagsMapInner = HashMap<FacetKey, Vec<PlainTag>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagsMap(TagsMapInner);

impl TagsMap {
    #[must_use]
    pub const fn new(inner: TagsMapInner) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> TagsMapInner {
        let Self(inner) = self;
        inner
    }

    pub fn insert(&mut self, key: FacetKey, tag: PlainTag) {
        use std::collections::hash_map::*;
        let Self(inner) = self;
        match inner.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(tag);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![tag]);
            }
        }
    }

    pub fn count(&mut self, facet_id: &FacetId) -> usize {
        let Self(inner) = self;
        inner.get(facet_id.value().as_str()).map_or(0, Vec::len)
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        let Self(inner) = self;
        inner.values().fold(0, |sum, tags| sum + tags.len())
    }
}

impl From<TagsMapInner> for TagsMap {
    fn from(from: TagsMapInner) -> Self {
        Self::new(from)
    }
}

impl From<TagsMap> for TagsMapInner {
    fn from(from: TagsMap) -> Self {
        from.into_inner()
    }
}

impl AsRef<TagsMapInner> for TagsMap {
    fn as_ref(&self) -> &TagsMapInner {
        let Self(inner) = self;
        inner
    }
}

impl TagsMap {
    pub fn get_plain_tags(&self) -> Option<&[PlainTag]> {
        let Self(all_tags) = self;
        all_tags.get(&FacetKey::new(None)).map(Vec::as_slice)
    }

    fn take_plain_tags(&mut self) -> Vec<PlainTag> {
        let Self(all_tags) = self;
        all_tags.remove(&FacetKey::new(None)).unwrap_or_default()
    }

    pub fn get_faceted_plain_tags(&self, facet_id: &FacetId) -> Option<&[PlainTag]> {
        let Self(all_tags) = self;
        all_tags.get(facet_id.value().as_str()).map(Vec::as_slice)
    }

    pub fn replace_faceted_plain_tags(
        &mut self,
        facet_id: FacetId,
        plain_tags: impl Into<Vec<PlainTag>>,
    ) -> Option<Vec<PlainTag>> {
        let Self(all_tags) = self;
        match all_tags.entry(Some(facet_id).into()) {
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
    pub fn update_faceted_plain_tags_by_label_ordering(
        &mut self,
        facet_id: &FacetId,
        plain_tags: impl Into<Vec<PlainTag>>,
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
            self.replace_faceted_plain_tags(facet_id.clone(), plain_tags);
        }
        true
    }

    pub fn take_faceted_tags(&mut self, facet_id: &FacetId) -> Option<FacetedTags> {
        let Self(all_tags) = self;
        all_tags
            .remove_entry(facet_id.value().as_str())
            .map(|(key, tags)| {
                let FacetKey(facet_id) = key;
                debug_assert!(facet_id.is_some());
                let facet_id = facet_id.expect("facet");
                FacetedTags { facet_id, tags }
            })
    }

    pub fn remove_faceted_tags(&mut self, facet_id: &FacetId) -> Option<usize> {
        let Self(all_tags) = self;
        all_tags
            .remove(facet_id.value().as_str())
            .map(|tags| tags.len())
    }
}

impl From<Tags> for TagsMap {
    fn from(from: Tags) -> Self {
        let Tags {
            plain: plain_tags,
            facets,
        } = from;
        let plain_iter = once((None.into(), plain_tags));
        let faceted_iter = facets.into_iter().map(|faceted_tags| {
            let FacetedTags { facet_id, tags } = faceted_tags;
            (facet_id.into(), tags)
        });
        Self::new(plain_iter.chain(faceted_iter).collect())
    }
}

impl From<TagsMap> for Tags {
    fn from(mut from: TagsMap) -> Self {
        let plain_tags = from.take_plain_tags();
        let TagsMap(faceted_tags) = from;
        let facets = faceted_tags
            .into_iter()
            .map(|(key, tags)| {
                let FacetKey(facet_id) = key;
                debug_assert!(facet_id.is_some());
                let facet_id = facet_id.expect("facet");
                FacetedTags { facet_id, tags }
            })
            .collect();
        let mut tags = Self {
            plain: plain_tags,
            facets,
        };
        tags.canonicalize();
        tags
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
