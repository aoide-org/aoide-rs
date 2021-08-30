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

///////////////////////////////////////////////////////////////////////

use crate::{compat::is_slice_sorted_by, prelude::*};

use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    iter::once,
    str::FromStr,
};

///////////////////////////////////////////////////////////////////////
// Score
///////////////////////////////////////////////////////////////////////

pub type ScoreValue = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Score(ScoreValue);

impl Score {
    pub const fn min_value() -> ScoreValue {
        0.0
    }

    pub const fn max_value() -> ScoreValue {
        1.0
    }

    pub const fn default_value() -> ScoreValue {
        Self::max_value()
    }

    pub fn clamp_value(value: impl Into<ScoreValue>) -> ScoreValue {
        value.into().clamp(Self::min_value(), Self::max_value())
    }

    pub fn clamp_from(value: impl Into<ScoreValue>) -> Score {
        Self::clamp_value(value).into()
    }

    pub const fn min() -> Self {
        Self(Self::min_value())
    }

    pub const fn max() -> Self {
        Self(Self::max_value())
    }

    pub const fn default() -> Self {
        Self(Self::default_value())
    }

    pub const fn new(inner: ScoreValue) -> Self {
        Self(inner)
    }

    pub const fn value(self) -> ScoreValue {
        let Self(value) = self;
        value
    }

    // Convert to percentage value with a single decimal digit
    pub fn to_percentage(self) -> ScoreValue {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScoreInvalidity {
    OutOfRange,
}

impl Validate for Score {
    type Invalidity = ScoreInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::min() && *self <= Self::max()),
                ScoreInvalidity::OutOfRange,
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

/// The name of a tag.
///
/// Format: Uniccode string without leading/trailing whitespace
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Label(LabelValue);

impl Label {
    pub fn clamp_str(value: &str) -> &str {
        value.trim()
    }

    pub fn clamp_value(value: impl Into<LabelValue>) -> LabelValue {
        let value = value.into();
        let clamped = Self::clamp_str(&value);
        if clamped == value {
            value
        } else {
            clamped.into()
        }
    }

    pub fn clamp_from(value: impl Into<LabelValue>) -> Label {
        Self::clamp_value(value).into()
    }

    pub const fn new(value: LabelValue) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }

    pub fn into_value(self) -> LabelValue {
        let Self(value) = self;
        value
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LabelInvalidity {
    Empty,
    Format,
}

impl Validate for Label {
    type Invalidity = LabelInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.value().is_empty(), LabelInvalidity::Empty)
            .invalidate_if(
                Self::clamp_str(self.as_ref()) != self.value().as_str(),
                LabelInvalidity::Format,
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

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        let Self(value) = self;
        value
    }
}

impl FromStr for Label {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::clamp_str(s).to_owned().into())
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

pub type FacetValue = String;

/// Identifier for a category of tags.
///
/// See also: Spotify categories
///
/// Format: ASCII string, no uppercase characters, no whitespace
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Facet(FacetValue);

/// The alphabet of facets
///
/// All valid characters ordered by their ASCII codes.
pub const FACET_ALPHABET: &str = "+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz";

impl Facet {
    pub fn clamp_value(value: impl Into<FacetValue>) -> FacetValue {
        let mut value = value.into();
        value.retain(Self::is_valid_char);
        value
    }

    pub fn clamp_from(value: impl Into<FacetValue>) -> Label {
        Self::clamp_value(value).into()
    }

    pub const fn new(value: FacetValue) -> Self {
        Self(value)
    }

    pub fn into_value(self) -> FacetValue {
        let Self(value) = self;
        value
    }

    pub const fn value(&self) -> &FacetValue {
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

impl CanonicalOrd for Facet {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FacetInvalidity {
    Empty,
    Format,
}

impl Validate for Facet {
    type Invalidity = FacetInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.value().is_empty(), FacetInvalidity::Empty)
            .invalidate_if(
                self.value().chars().any(Facet::is_invalid_char),
                FacetInvalidity::Format,
            )
            .into()
    }
}

impl From<FacetValue> for Facet {
    fn from(from: FacetValue) -> Self {
        Self::new(from)
    }
}

impl From<Facet> for FacetValue {
    fn from(from: Facet) -> Self {
        from.into_value()
    }
}

impl AsRef<FacetValue> for Facet {
    fn as_ref(&self) -> &FacetValue {
        let Self(value) = self;
        value
    }
}

impl AsRef<str> for Facet {
    fn as_ref(&self) -> &str {
        let Self(value) = self;
        value
    }
}

impl FromStr for Facet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::clamp_value(s).into())
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value())
    }
}

pub trait Faceted {
    fn facet(&self) -> Option<&Facet>;
}

impl Faceted for Facet {
    fn facet(&self) -> Option<&Self> {
        Some(self)
    }
}

#[derive(Clone, Default, Debug)]
pub struct FacetKey(Option<Facet>);

impl FacetKey {
    pub const fn new(inner: Option<Facet>) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> Option<Facet> {
        let Self(inner) = self;
        inner
    }
}

impl From<Option<Facet>> for FacetKey {
    fn from(inner: Option<Facet>) -> Self {
        FacetKey::new(inner)
    }
}

impl From<FacetKey> for Option<Facet> {
    fn from(from: FacetKey) -> Self {
        from.into_inner()
    }
}

impl From<Facet> for FacetKey {
    fn from(from: Facet) -> Self {
        FacetKey(Some(from))
    }
}

impl AsRef<Option<Facet>> for FacetKey {
    fn as_ref(&self) -> &Option<Facet> {
        let Self(inner) = self;
        inner
    }
}

impl AsRef<str> for FacetKey {
    fn as_ref(&self) -> &str {
        let Self(inner) = self;
        match inner {
            Some(facet) => facet.as_ref(),
            None => "",
        }
    }
}

impl Borrow<str> for FacetKey {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl FromStr for FacetKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = Facet::clamp_value(s);
        let inner = if value.is_empty() {
            None
        } else {
            Some(Facet::new(value))
        };
        Ok(inner.into())
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    pub facet: Facet,
    pub tags: Vec<PlainTag>,
}

impl CanonicalOrd for FacetedTags {
    fn canonical_dedup_cmp(&self, other: &Self) -> Ordering {
        debug_assert_eq!(Ordering::Equal, self.canonical_cmp(other));
        // Conflicting tags should be resolved before, e.g. by concatenating
        // all plain tags that share the same facet. Here we just select the
        // faceted tag with more plain tag, i.e. reverse ordering of their
        // length.
        other.tags.len().cmp(&self.tags.len())
    }

    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.facet.canonical_cmp(&other.facet)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FacetedTagInvalidity {
    Facet(FacetInvalidity),
    Tag(PlainTagInvalidity),
}

impl Validate for FacetedTags {
    type Invalidity = FacetedTagInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { facet, tags } = self;
        ValidationContext::new()
            .validate_with(facet, Self::Invalidity::Facet)
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
    fn facet(&self) -> Option<&Facet> {
        Some(&self.facet)
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
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }

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
        facets.sort_unstable_by(|lhs, rhs| lhs.facet.canonical_cmp(&rhs.facet));
        facets.dedup_by(|next, prev| {
            if prev
                .facet
                .canonical_cmp(&next.facet)
                .then_with(|| prev.facet.canonical_dedup_cmp(&next.facet))
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TagsInvalidity {
    Facet(FacetInvalidity),
    PlainTag(PlainTagInvalidity),
    DuplicateFacets,
    DuplicateLabels,
}

fn check_for_duplicates_in_sorted_plain_tags_slice(
    plain_tags: &[PlainTag],
) -> Option<TagsInvalidity> {
    debug_assert!(is_slice_sorted_by(plain_tags, |lhs, rhs| lhs
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
    debug_assert!(is_slice_sorted_by(faceted_tags, |lhs, rhs| lhs
        .facet
        .cmp(&rhs.facet)));
    let mut iter = faceted_tags.iter();
    if let Some(mut prev) = iter.next() {
        let duplicate_labels = check_for_duplicates_in_sorted_plain_tags_slice(&prev.tags);
        if duplicate_labels.is_some() {
            return duplicate_labels;
        }
        for next in iter {
            if prev.facet == next.facet {
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
                        let FacetedTags { facet, tags } = faceted_tags;
                        ctx.validate_with(facet, Self::Invalidity::Facet)
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
    pub const fn new(inner: TagsMapInner) -> Self {
        Self(inner)
    }

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

    pub fn count(&mut self, facet: &Facet) -> usize {
        let Self(inner) = self;
        inner
            .get(facet.value().as_str())
            .map(|val| val.len())
            .unwrap_or(0)
    }

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
    fn take_plain_tags(&mut self) -> Vec<PlainTag> {
        let Self(all_tags) = self;
        all_tags.remove(&FacetKey::new(None)).unwrap_or_default()
    }

    pub fn take_faceted_tags(&mut self, facet: &Facet) -> Option<FacetedTags> {
        let Self(all_tags) = self;
        all_tags
            .remove_entry(facet.value().as_str())
            .map(|(key, tags)| {
                let FacetKey(facet) = key;
                debug_assert!(facet.is_some());
                let facet = facet.expect("facet");
                FacetedTags { facet, tags }
            })
    }

    pub fn remove_faceted_tags(&mut self, facet: &Facet) -> usize {
        let Self(all_tags) = self;
        all_tags
            .remove(facet.value().as_str())
            .map(|tags| tags.len())
            .unwrap_or(0)
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
            let FacetedTags { facet, tags } = faceted_tags;
            (facet.into(), tags)
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
                let FacetKey(facet) = key;
                debug_assert!(facet.is_some());
                let facet = facet.expect("facet");
                FacetedTags { facet, tags }
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
