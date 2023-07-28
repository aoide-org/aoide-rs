// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use aoide_core::{
    prelude::*,
    tag::{FacetId, FacetKey, FacetedTags, PlainTag, Score, ScoreValue, Tags, TagsMap},
};
use compact_str::{format_compact, CompactString};
use gigtag::facet::{has_date_like_suffix, Facet as _};

pub type Facet = gigtag::facet::CompactFacet;

pub type Label = gigtag::label::CompactLabel;

pub type PropName = gigtag::props::CompactName;
pub type PropValue = CompactString;
pub type Property = gigtag::props::Property<PropName, PropValue>;

pub type Tag = gigtag::Tag<Facet, Label, PropName, PropValue>;
pub type DecodedTags = gigtag::DecodedTags<Facet, Label, PropName, PropValue>;

pub(crate) const SCORE_PROP_NAME: &str = "s";

fn export_valid_label(label: &aoide_core::tag::Label<'_>) -> Option<Label> {
    let label = label.as_str();
    (gigtag::label::is_valid(label)).then(|| gigtag::label::Label::from_str(label))
}

fn export_facet(facet_id: &FacetId<'_>) -> Facet {
    let facet = Facet::from_str(facet_id.as_str());
    debug_assert!(facet.is_valid());
    facet
}

#[must_use]
fn try_export_plain_tag(facet: Facet, plain_tag: &PlainTag<'_>) -> Option<Tag> {
    debug_assert!((facet.is_valid()));
    let label = if let Some(label) = plain_tag.label.as_ref() {
        export_valid_label(label)?
    } else {
        Default::default()
    };
    // A default score could only be omitted if the tag has a label or a date-like facet!
    let score = (plain_tag.score != Default::default()
        || (label.is_empty() && !has_date_like_suffix(&facet)))
    .then(|| Property {
        name: gigtag::props::Name::from_str(SCORE_PROP_NAME),
        value: format_compact!("{score}", score = plain_tag.score.value()),
    });
    let tag = Tag {
        facet,
        label,
        props: score.into_iter().collect(),
    };
    debug_assert!(tag.has_facet() || tag.has_label());
    debug_assert!(tag.props().len() <= 1);
    log::debug!(
        "Exported {encoded_tag} from {plain_tag:?}",
        encoded_tag = tag.encode()
    );
    tag.is_valid().then_some(tag)
}

fn export_plain_tags<'item>(
    facet: Facet,
    iter: impl Iterator<Item = &'item PlainTag<'item>> + 'item,
) -> impl Iterator<Item = Tag> + 'item {
    iter.filter_map(move |plain_tag| {
        try_export_plain_tag(facet.clone(), plain_tag).or_else(|| {
            log::warn!("Failed to export {facet:?} {plain_tag:?}");
            None
        })
    })
}

fn export_tags(tags: Canonical<&Tags<'_>>) -> Vec<Tag> {
    let mut exported_tags = Vec::with_capacity(tags.total_count());
    exported_tags.extend(export_plain_tags(Default::default(), tags.plain.iter()));
    tags.facets
        .iter()
        .fold(exported_tags, |mut exported_tags, faceted_tags| {
            let facet = export_facet(&faceted_tags.facet_id);
            exported_tags.extend(export_plain_tags(facet, faceted_tags.tags.iter()));
            exported_tags
        })
}

pub fn update_tags_in_encoded(
    tags: Canonical<&Tags<'_>>,
    encoded: &mut Cow<'_, str>,
) -> std::fmt::Result {
    let mut exported_tags = export_tags(tags);
    if exported_tags.is_empty() {
        return Ok(());
    }
    // Preserve all gig tags that could not be imported as aoide tags,
    // thereby essentially replacing the old aoide tags (that are simply
    // discarded after decoding) with the new exported aoide tags.
    let (mut decoded_tags, _num_imported) = decode_tags_eagerly_into(encoded, None);
    decoded_tags.tags.append(&mut exported_tags);
    decoded_tags.reorder_and_dedup();
    let encoded = encoded.to_mut();
    encoded.clear();
    decoded_tags.encode_into(encoded)
}

#[allow(clippy::needless_pass_by_value)] // consume remaining_tags
pub fn export_and_encode_tags_into(
    tags: Canonical<&Tags<'_>>,
    encoded_tags: &mut Vec<PlainTag<'_>>,
) -> std::fmt::Result {
    if encoded_tags.len() == 1 {
        let PlainTag { label, score } = encoded_tags.drain(..).next().expect("exactly one item");
        let mut encoded = label.unwrap_or_default().into();
        crate::util::gigtag::update_tags_in_encoded(tags, &mut encoded)?;
        let tag = PlainTag {
            label: aoide_core::tag::Label::clamp_from(encoded),
            score,
        };
        *encoded_tags = vec![tag];
    } else {
        let mut encoded = Cow::Owned(String::new());
        crate::util::gigtag::update_tags_in_encoded(tags, &mut encoded)?;
        let tag = PlainTag {
            label: aoide_core::tag::Label::clamp_from(encoded),
            ..Default::default()
        };
        encoded_tags.push(tag);
    }
    Ok(())
}

fn try_import_tag(tag: &Tag) -> Option<(FacetKey<'_>, PlainTag<'_>)> {
    let score = match &tag.props() {
        [] => Default::default(),
        [prop] => {
            // Skip non-aoide tags with unknown property names
            if prop.name().as_ref() != SCORE_PROP_NAME {
                return None;
            }
            // Skip non-aoide tag if property value fails to parse
            let score_value = prop.value().parse::<ScoreValue>().ok()?;
            let score = Score::clamp_from(score_value);
            // Skip non-aoide tag if property value fails is not a valid score value
            #[allow(clippy::float_cmp)]
            if score_value != score.value() {
                return None;
            }
            score
        }
        [_, ..] => {
            // Skip non-aoide tag with multiple properties
            return None;
        }
    };
    if !(tag.has_facet() || tag.has_label()) {
        return None;
    }
    let facet_key = if tag.has_facet() {
        let facet_id_clamped = FacetId::clamp_from(tag.facet().as_ref());
        let facet_id_unclamped = Some(FacetId::new_unchecked(Cow::Borrowed(tag.facet())));
        if facet_id_clamped != facet_id_unclamped {
            // Skip non-aoide tag
            return None;
        }
        facet_id_clamped.into()
    } else {
        Default::default()
    };
    let label = if tag.has_label() {
        let label_str = tag.label().as_ref();
        let label = aoide_core::tag::Label::clamp_from(label_str);
        if label.as_ref().map_or("", aoide_core::tag::Label::as_str) != label_str {
            // Skip non-aoide tag
            return None;
        }
        label
    } else {
        None
    };
    let plain_tag = PlainTag { label, score };
    if !plain_tag.is_valid() {
        return None;
    }
    (facet_key, plain_tag).into()
}

fn decode_tags_eagerly_into(
    encoded: &str,
    mut tags_map: Option<&mut TagsMap<'static>>,
) -> (DecodedTags, usize) {
    let mut num_imported = 0;
    let mut decoded_tags = DecodedTags::decode_str(encoded);
    decoded_tags.tags.retain(|tag| {
        if let Some((facet_key, plain_tag)) = try_import_tag(tag) {
            log::debug!("Imported {facet_key:?} {plain_tag:?} from {tag:?}");
            if let Some(tags_map) = tags_map.as_mut() {
                tags_map.insert(facet_key.into_owned(), plain_tag.into_owned());
            }
            num_imported += 1;
            // Discard the imported tag
            false
        } else {
            log::debug!("Skipped import of {tag:?}");
            // Preserve the unknown tag
            true
        }
    });
    (decoded_tags, num_imported)
}

fn import_and_extract_tags_from_label_eagerly_into(
    label: &mut aoide_core::tag::Label<'_>,
    tags_map: Option<&mut TagsMap<'static>>,
) -> (bool, usize) {
    let (decoded_tags, num_imported) = decode_tags_eagerly_into(label.as_str(), tags_map);
    if num_imported == 0 {
        // Preserve as is
        return (true, num_imported);
    }
    // Re-encode undecoded prefix and remaining tags
    let reencoded = match decoded_tags.reencode() {
        Ok(reencoded) => reencoded,
        Err(err) => {
            // This is unexpected and should never happen
            log::error!("Failed to re-encode undecoded prefix and remaining tags: {err}");
            // Preserve everything as is (even though some tags have already been imported)
            return (true, num_imported);
        }
    };
    if let Some(remaining_label) = aoide_core::tag::Label::clamp_from(reencoded) {
        *label = remaining_label;
    } else {
        // Nothing remaining that needs to be preserved
        return (false, num_imported);
    }
    (true, num_imported)
}

#[must_use]
pub fn import_from_faceted_tags(mut faceted_tags: FacetedTags<'static>) -> TagsMap<'static> {
    let mut tags_map = TagsMap::default();
    faceted_tags.tags.retain_mut(|plain_tag| {
        if let Some(label) = plain_tag.label.as_mut() {
            let (retain, num_imported) =
                import_and_extract_tags_from_label_eagerly_into(label, Some(&mut tags_map));
            if retain {
                log::debug!("Imported {num_imported} tag(s) retaining {plain_tag:?}");
            } else {
                log::debug!("Imported {num_imported} tag(s)");
            }
            retain
        } else {
            true
        }
    });
    if !faceted_tags.tags.is_empty() {
        let FacetedTags { facet_id, mut tags } = faceted_tags;
        let ingested_tags = tags_map.take_faceted_tags(&facet_id);
        if let Some(mut ingested_tags) = ingested_tags {
            if !ingested_tags.tags.is_empty() {
                log::warn!(
                    "Joining {num_undecoded} undecoded with {num_ingested} ingested tag(s) for \
                     facet '{facet_id}'",
                    num_undecoded = tags.len(),
                    num_ingested = ingested_tags.tags.len()
                );
                tags.append(&mut ingested_tags.tags);
            }
        }
        tags_map.replace_faceted_plain_tags(facet_id, tags);
    }
    tags_map
}

#[cfg(test)]
mod tests;
