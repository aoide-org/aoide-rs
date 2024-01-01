// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::tag::{FACET_ID_GENRE, FACET_ID_MOOD};
use aoide_media_file::util::tag::{
    FacetedTagMappingConfig, FacetedTagMappingConfigInner, TagMappingConfig,
};

pub mod source;
pub mod tracker;

/// Default separator for genre/mood tags on import/export.
///
/// Multiple genre/mood tags are concatenated with this
/// separator and exported into a single tag field.
///
/// On import the tag field contents are split by this
/// separator and ordered by applying
/// [`DEFAULT_GENRE_MOOD_SCORE_ATTENUATION`] for deriving
/// the score.
pub const DEFAULT_GENRE_MOOD_LABEL_SEPARATOR: &str = ";";

/// Exponential attenuation for ordering multiple genre/mood
/// tags by score on import.
///
/// Used when importing tags from composite tag field, see
/// also [`DEFAULT_GENRE_MOOD_LABEL_SEPARATOR`]. The score
/// value of the first tag is [`aoide_core::tag::Score::MAX`].
pub const DEFAULT_GENRE_MOOD_SCORE_ATTENUATION: f64 = 0.75;

/// An opinionated [`FacetedTagMappingConfig`] that supports
/// multi-valued genre/mood file tags split by a commonly used
/// separator.
///
/// See also:
/// - [`DEFAULT_GENRE_MOOD_LABEL_SEPARATOR`]
/// - [`DEFAULT_GENRE_MOOD_SCORE_ATTENUATION`]
#[must_use]
pub fn predefined_faceted_tag_mapping_config() -> FacetedTagMappingConfig {
    [
        (
            FACET_ID_GENRE.to_borrowed().into(),
            TagMappingConfig {
                label_separator: DEFAULT_GENRE_MOOD_LABEL_SEPARATOR.to_owned(),
                split_score_attenuation: DEFAULT_GENRE_MOOD_SCORE_ATTENUATION,
            },
        ),
        (
            FACET_ID_MOOD.to_borrowed().into(),
            TagMappingConfig {
                label_separator: DEFAULT_GENRE_MOOD_LABEL_SEPARATOR.to_owned(),
                split_score_attenuation: DEFAULT_GENRE_MOOD_SCORE_ATTENUATION,
            },
        ),
    ]
    .into_iter()
    .collect::<FacetedTagMappingConfigInner>()
    .into()
}
