// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use egui::{Context, TextureHandle, TextureOptions};

use aoide::{
    TrackUid,
    music::{key::KeySignature, tempo::TempoBpm},
    tag::FacetId,
    track::{
        AdvisoryRating,
        tag::{FACET_ID_COMMENT, FACET_ID_GENRE, FACET_ID_GROUPING},
    },
    util::clock::DateOrDateTime,
};
use itertools::Itertools as _;
use url::Url;

use super::{artwork_thumbnail_image, artwork_thumbnail_image_placeholder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackYear {
    pub min: i16,
    pub max: i16,
}

/// Simplified, pre-rendered track data
pub struct TrackListItem {
    pub entity_uid: TrackUid,
    pub content_url: Option<Url>,

    pub artist: Option<String>,
    pub title: Option<String>,
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub album_subtitle: Option<String>,
    pub copyright: Option<String>,
    pub advisory_rating: Option<AdvisoryRating>,
    pub grouping: Option<String>,
    pub comment: Option<String>,
    pub genres: Vec<String>,
    pub year: Option<TrackYear>,
    pub bpm: Option<TempoBpm>,
    pub key: Option<KeySignature>,

    pub artwork_thumbnail_texture: TextureHandle,
}

const MULTI_VALUED_TAG_LABEL_SEPARATOR: &str = "\n";

impl TrackListItem {
    #[must_use]
    pub fn new(
        ctx: &Context,
        entity_uid: aoide::TrackUid,
        content_url: Option<Url>,
        track: &aoide::Track,
    ) -> Self {
        let artist = track.track_artist().map(ToOwned::to_owned);
        let title = track.track_title().map(ToOwned::to_owned);
        let album_artist = track.album_artist().map(ToOwned::to_owned);
        let album_title = track.album_title().map(ToOwned::to_owned);
        let album_subtitle = track.album_subtitle().map(ToOwned::to_owned);
        let copyright = track.copyright.clone();
        let advisory_rating = track.advisory_rating;
        let genres = filter_faceted_track_tag_labels(track, FACET_ID_GENRE)
            .map(ToString::to_string)
            .collect();
        let grouping = concat_faceted_track_tag_labels(
            track,
            FACET_ID_GROUPING,
            MULTI_VALUED_TAG_LABEL_SEPARATOR,
        );
        let comment = concat_faceted_track_tag_labels(
            track,
            FACET_ID_COMMENT,
            MULTI_VALUED_TAG_LABEL_SEPARATOR,
        );
        let dates = track
            .recorded_at
            .as_ref()
            .into_iter()
            .chain(track.released_at.as_ref())
            .chain(track.released_orig_at.as_ref());
        let year_min = dates.clone().map(DateOrDateTime::year).min();
        let year_max = dates.map(DateOrDateTime::year).max();
        let year = match (year_min, year_max) {
            (Some(min), Some(max)) => Some(TrackYear { min, max }),
            (None, None) => None,
            _ => unreachable!(),
        };
        let bpm = track.metrics.tempo_bpm;
        let key = track.metrics.key_signature;
        let artwork_thumbnail_image = track
            .media_source
            .artwork
            .as_ref()
            .and_then(|artwork| {
                let default_color = match track.color {
                    Some(aoide::util::color::Color::Rgb(rgb_color)) => Some(rgb_color),
                    _ => None,
                };
                artwork_thumbnail_image(artwork, default_color)
            })
            .unwrap_or_else(|| {
                // TODO: Use a single, shared, transparent texture for all tracks without artwork.
                artwork_thumbnail_image_placeholder()
            });
        // TODO: Only load the texture once for each distinct image -> hash the image data.
        let artwork_thumbnail_texture = ctx.load_texture(
            "", // anonymous
            artwork_thumbnail_image,
            TextureOptions::LINEAR,
        );
        Self {
            entity_uid,
            content_url,
            artist,
            title,
            album_artist,
            album_title,
            album_subtitle,
            copyright,
            advisory_rating,
            grouping,
            comment,
            genres,
            year,
            bpm,
            key,
            artwork_thumbnail_texture,
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for TrackListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrackListItem")
            .field("entity_uid", &self.entity_uid)
            .field(
                "artwork_thumbnail_texture",
                &self.artwork_thumbnail_texture.id(),
            )
            .finish()
    }
}

fn filter_faceted_track_tag_labels<'a>(
    track: &'a aoide::Track,
    facet_id: &'a FacetId,
) -> impl Iterator<Item = &'a aoide::tag::Label<'a>> {
    track
        .tags
        .facets
        .iter()
        .filter_map(|faceted_tags| {
            if faceted_tags.facet_id == *facet_id {
                Some(faceted_tags.tags.iter())
            } else {
                None
            }
        })
        .flatten()
        .filter_map(|tag| tag.label.as_ref())
}

#[must_use]
#[allow(unstable_name_collisions)] // Itertools::intersperse()
fn concat_faceted_track_tag_labels(
    track: &aoide::Track,
    facet_id: &FacetId,
    separator: &str,
) -> Option<String> {
    let concat = filter_faceted_track_tag_labels(track, facet_id)
        .map(aoide::tag::Label::as_str)
        .intersperse(separator)
        .collect::<String>();
    if concat.is_empty()
        && filter_faceted_track_tag_labels(track, facet_id)
            .next()
            .is_none()
    {
        None
    } else {
        Some(concat)
    }
}
