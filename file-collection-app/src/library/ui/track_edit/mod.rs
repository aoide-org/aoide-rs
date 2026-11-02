// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Editable track properties.

use std::fmt;

use itertools::Itertools as _;
use nonicle::{Canonical, CanonicalizeInto as _};

use aoide::{
    PlainTag, TagFacetId, TagLabel, Tags, Track,
    media_file::util::gigtag,
    music::{
        key::{KeyCode, KeySignature},
        tempo::TempoBpm,
    },
    tag::FacetedTags,
    track::{
        AdvisoryRating,
        metric::MetricsFlags,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_LANGUAGE, FACET_ID_MBID_ARTIST, FACET_ID_MBID_RECORDING,
            FACET_ID_MBID_RELEASE, FACET_ID_MBID_RELEASE_ARTIST, FACET_ID_MBID_RELEASE_GROUP,
            FACET_ID_MBID_TRACK, FACET_ID_MBID_WORK, FACET_ID_MOOD, FACET_ID_XID,
        },
        title::{Kind as TitleKind, Title, Titles},
    },
    util::{
        clock::{DateOrDateTime, OffsetDateTimeMs, YyyyMmDdDate},
        color::{Color, ColorIndex, RgbColor},
    },
};

/// Facets that are not editable as custom tags.
///
/// Either edited by a dedicated field or stored in a dedicated file tag.
/// Tags with these facets are preserved when editing custom tags.
const NON_CUSTOM_TAG_FACETS: &[&TagFacetId] = &[
    FACET_ID_COMMENT,
    FACET_ID_DESCRIPTION,
    FACET_ID_GENRE,
    FACET_ID_GROUPING,
    FACET_ID_ISRC,
    FACET_ID_LANGUAGE,
    FACET_ID_MBID_ARTIST,
    FACET_ID_MBID_RECORDING,
    FACET_ID_MBID_RELEASE,
    FACET_ID_MBID_RELEASE_ARTIST,
    FACET_ID_MBID_RELEASE_GROUP,
    FACET_ID_MBID_TRACK,
    FACET_ID_MBID_WORK,
    FACET_ID_MOOD,
    FACET_ID_XID,
];

#[must_use]
fn is_custom_tag_facet(facet_id: &TagFacetId) -> bool {
    !NON_CUSTOM_TAG_FACETS.contains(&facet_id)
}

const LINE_SEPARATOR: &str = "\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Title,
    Artist,
    AlbumTitle,
    AlbumArtist,
    AlbumSubtitle,
    RecordedAt,
    Bpm,
    Key,
    Genres,
    Grouping,
    Comment,
    Publisher,
    Copyright,
    AdvisoryRating,
    Color,
    CustomTags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    pub field: Field,
    pub message: String,
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// Editable track properties as text.
///
/// Only modified fields are applied to the track. All other properties
/// are preserved as is, even if they could not be represented losslessly
/// by the corresponding field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackEditDraft {
    pub title: String,
    pub artist: String,
    pub album_title: String,
    pub album_artist: String,
    pub album_subtitle: String,
    /// `YYYY`, `YYYY-MM`, `YYYY-MM-DD`, or an RFC 3339 date/time.
    pub recorded_at: String,
    pub bpm: String,
    /// Lancelot key code, e.g. `8A`.
    pub key: String,
    /// One genre per line.
    pub genres: String,
    pub grouping: String,
    pub comment: String,
    pub publisher: String,
    pub copyright: String,
    pub advisory_rating: Option<AdvisoryRating>,
    /// `#RRGGBB` or a color index.
    pub color: String,
    /// Custom tags as gig tags, separated by whitespace.
    pub custom_tags: String,
}

impl TrackEditDraft {
    #[must_use]
    pub fn from_track(track: &Track) -> Self {
        Self {
            title: track.track_title().unwrap_or_default().to_owned(),
            artist: track.track_artist().unwrap_or_default().to_owned(),
            album_title: track.album_title().unwrap_or_default().to_owned(),
            album_artist: track.album_artist().unwrap_or_default().to_owned(),
            album_subtitle: track.album_subtitle().unwrap_or_default().to_owned(),
            recorded_at: track
                .recorded_at
                .as_ref()
                .map(format_date_or_date_time)
                .unwrap_or_default(),
            bpm: track
                .metrics
                .tempo_bpm
                .map(|bpm| bpm.value().to_string())
                .unwrap_or_default(),
            key: track
                .metrics
                .key_signature
                .map(|key| key.code().as_lancelot_str().to_owned())
                .unwrap_or_default(),
            genres: join_faceted_tag_labels(&track.tags, FACET_ID_GENRE),
            grouping: join_faceted_tag_labels(&track.tags, FACET_ID_GROUPING),
            comment: join_faceted_tag_labels(&track.tags, FACET_ID_COMMENT),
            publisher: track.publisher.clone().unwrap_or_default(),
            copyright: track.copyright.clone().unwrap_or_default(),
            advisory_rating: track.advisory_rating,
            color: track.color.map(format_color).unwrap_or_default(),
            custom_tags: encode_custom_tags(&track.tags),
        }
    }

    /// Check if any field differs from the original track.
    #[must_use]
    pub fn is_modified(&self, original: &Track) -> bool {
        *self != Self::from_track(original)
    }

    /// Apply all modified fields to the original track.
    ///
    /// Returns the edited track or the errors of all invalid fields.
    pub fn to_track(&self, original: &Track) -> Result<Track, Vec<FieldError>> {
        let unmodified = Self::from_track(original);
        let mut track = original.clone();
        let mut errors = Vec::new();
        let mut tags = std::mem::take(&mut track.tags).untie();

        let Self {
            title,
            artist,
            album_title,
            album_artist,
            album_subtitle,
            recorded_at,
            bpm,
            key,
            // Tags are applied separately
            genres: _,
            grouping: _,
            comment: _,
            publisher,
            copyright,
            advisory_rating,
            color,
            custom_tags: _,
        } = self;
        if *title != unmodified.title {
            set_main_title(&mut track.titles, non_empty_trimmed(title));
        }
        if *artist != unmodified.artist {
            track.set_track_artist(non_empty_trimmed(artist));
        }
        if *album_title != unmodified.album_title {
            let mut album = std::mem::take(&mut track.album).untie();
            set_main_title(&mut album.titles, non_empty_trimmed(album_title));
            track.album = Canonical::tie(album);
        }
        if *album_artist != unmodified.album_artist {
            track.set_album_artist(non_empty_trimmed(album_artist));
        }
        if *album_subtitle != unmodified.album_subtitle {
            track.set_album_subtitle(non_empty_trimmed(album_subtitle));
        }
        if *recorded_at != unmodified.recorded_at {
            match parse_date_or_date_time(recorded_at) {
                Ok(recorded_at) => track.recorded_at = recorded_at,
                Err(message) => errors.push(FieldError {
                    field: Field::RecordedAt,
                    message,
                }),
            }
        }
        if *bpm != unmodified.bpm {
            match parse_bpm(bpm) {
                Ok(bpm) => {
                    // Same as when importing the BPM from file tags.
                    let is_integer = bpm.is_some_and(|bpm| bpm.value().fract() == 0.0);
                    track
                        .metrics
                        .flags
                        .set(MetricsFlags::TEMPO_BPM_INTEGER, is_integer);
                    track.metrics.tempo_bpm = bpm;
                }
                Err(message) => errors.push(FieldError {
                    field: Field::Bpm,
                    message,
                }),
            }
        }
        if *key != unmodified.key {
            match parse_key(key) {
                Ok(key) => track.metrics.key_signature = key,
                Err(message) => errors.push(FieldError {
                    field: Field::Key,
                    message,
                }),
            }
        }
        if *publisher != unmodified.publisher {
            track.publisher = non_empty_trimmed(publisher);
        }
        if *copyright != unmodified.copyright {
            track.copyright = non_empty_trimmed(copyright);
        }
        if *advisory_rating != unmodified.advisory_rating {
            track.advisory_rating = *advisory_rating;
        }
        if *color != unmodified.color {
            match parse_color(color) {
                Ok(color) => track.color = color,
                Err(message) => errors.push(FieldError {
                    field: Field::Color,
                    message,
                }),
            }
        }
        errors.extend(self.apply_tags(&unmodified, &mut tags));

        if !errors.is_empty() {
            return Err(errors);
        }
        track.tags = tags.canonicalize_into();
        Ok(track)
    }

    fn apply_tags(&self, unmodified: &Self, tags: &mut Tags<'static>) -> Option<FieldError> {
        let Self {
            genres,
            grouping,
            comment,
            custom_tags,
            ..
        } = self;
        if *genres != unmodified.genres {
            replace_faceted_tags(tags, FACET_ID_GENRE, genres.lines());
        }
        if *grouping != unmodified.grouping {
            replace_faceted_tags(tags, FACET_ID_GROUPING, [grouping.as_str()]);
        }
        if *comment != unmodified.comment {
            replace_faceted_tags(tags, FACET_ID_COMMENT, [comment.as_str()]);
        }
        if *custom_tags != unmodified.custom_tags {
            match decode_custom_tags(custom_tags) {
                Ok(custom_tags) => {
                    let Tags { plain, facets } = custom_tags;
                    tags.plain = plain;
                    tags.facets
                        .retain(|faceted_tags| !is_custom_tag_facet(&faceted_tags.facet_id));
                    tags.facets.extend(facets);
                }
                Err(message) => {
                    return Some(FieldError {
                        field: Field::CustomTags,
                        message,
                    });
                }
            }
        }
        None
    }
}

#[must_use]
fn non_empty_trimmed(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn set_main_title(titles: &mut Canonical<Vec<Title>>, name: Option<String>) {
    let mut untied = std::mem::take(titles).untie();
    if Titles::set_kind_title(&mut untied, TitleKind::Main, name) {
        *titles = untied.canonicalize_into();
    } else {
        *titles = Canonical::tie(untied);
    }
}

fn filter_faceted_tags<'a>(
    tags: &'a Tags<'static>,
    facet_id: &'a TagFacetId,
) -> impl Iterator<Item = &'a PlainTag<'static>> {
    tags.facets
        .iter()
        .filter(move |faceted_tags| faceted_tags.facet_id == *facet_id)
        .flat_map(|faceted_tags| faceted_tags.tags.iter())
}

#[must_use]
fn join_faceted_tag_labels(tags: &Tags<'static>, facet_id: &TagFacetId) -> String {
    filter_faceted_tags(tags, facet_id)
        .filter_map(|tag| tag.label.as_ref().map(TagLabel::as_str))
        .join(LINE_SEPARATOR)
}

/// Replace the tags of a facet with labels.
///
/// The scores of existing tags with the same label are preserved.
fn replace_faceted_tags<'a>(
    tags: &mut Tags<'static>,
    facet_id: &TagFacetId,
    labels: impl IntoIterator<Item = &'a str>,
) {
    let new_tags: Vec<_> = labels
        .into_iter()
        .filter_map(|label| TagLabel::clamp_from(label.to_owned()))
        .map(|label| {
            filter_faceted_tags(tags, facet_id)
                .find(|tag| tag.label.as_ref() == Some(&label))
                .cloned()
                .unwrap_or_else(|| PlainTag {
                    label: Some(label),
                    ..Default::default()
                })
        })
        .collect();
    tags.facets
        .retain(|faceted_tags| faceted_tags.facet_id != *facet_id);
    tags.facets.push(FacetedTags {
        facet_id: facet_id.clone(),
        tags: new_tags,
    });
}

#[must_use]
fn encode_custom_tags(tags: &Tags<'static>) -> String {
    let custom_tags = Tags {
        plain: tags.plain.clone(),
        facets: tags
            .facets
            .iter()
            .filter(|faceted_tags| is_custom_tag_facet(&faceted_tags.facet_id))
            .cloned()
            .collect(),
    };
    // Already canonical, because the tags of the track are canonical.
    let custom_tags = Canonical::tie(custom_tags);
    gigtag::encode_tags(custom_tags.as_canonical_ref()).unwrap_or_else(|err| {
        log::error!("Failed to encode custom tags: {err}");
        String::new()
    })
}

fn decode_custom_tags(input: &str) -> Result<Tags<'static>, String> {
    // Gig tags must be encoded on a single line.
    let input = input.split_whitespace().join(" ");
    let decoded = gigtag::decode_tags(&input).map_err(|err| {
        format!(
            "Invalid custom tags: {invalid}",
            invalid = err.invalid.trim()
        )
    })?;
    let tags = decoded.canonicalize_into().untie();
    let non_custom_facets = tags
        .facets
        .iter()
        .map(|faceted_tags| &faceted_tags.facet_id)
        .filter(|facet_id| !is_custom_tag_facet(facet_id))
        .join(", ");
    if !non_custom_facets.is_empty() {
        return Err(format!("Not editable as custom tags: {non_custom_facets}"));
    }
    Ok(tags)
}

#[must_use]
fn format_date_or_date_time(date_or_date_time: &DateOrDateTime) -> String {
    match date_or_date_time {
        DateOrDateTime::Date(date) => {
            if date.is_year() {
                format!("{year:04}", year = date.year())
            } else if date.day_of_month() == 0 {
                format!(
                    "{year:04}-{month:02}",
                    year = date.year(),
                    month = date.month()
                )
            } else {
                format!(
                    "{year:04}-{month:02}-{day:02}",
                    year = date.year(),
                    month = date.month(),
                    day = date.day_of_month()
                )
            }
        }
        DateOrDateTime::DateTime(date_time) => date_time.to_string(),
    }
}

fn parse_date_or_date_time(input: &str) -> Result<Option<DateOrDateTime>, String> {
    const INVALID_DATE: &str = "Expected YYYY, YYYY-MM, YYYY-MM-DD, or an RFC 3339 date/time";
    let input = input.trim();
    if input.is_empty() {
        return Ok(None);
    }
    if input.contains('T') {
        return input
            .parse::<OffsetDateTimeMs>()
            .map(|date_time| Some(date_time.into()))
            .map_err(|_| INVALID_DATE.to_owned());
    }
    let mut pieces = input.split('-');
    let year = pieces.next().and_then(|year| {
        (year.len() == 4)
            .then(|| year.parse::<i16>().ok())
            .flatten()
    });
    let month = pieces.next().map(str::parse::<i8>);
    let day = pieces.next().map(str::parse::<i8>);
    let (Some(year), None) = (year, pieces.next()) else {
        return Err(INVALID_DATE.to_owned());
    };
    let (date, has_month, has_day) = match (month, day) {
        (None, None) => (YyyyMmDdDate::from_year(year), false, false),
        (Some(Ok(month)), None) => (YyyyMmDdDate::from_year_month(year, month), true, false),
        (Some(Ok(month)), Some(Ok(day))) => (
            YyyyMmDdDate::new_unchecked(
                i32::from(year) * 10_000 + i32::from(month) * 100 + i32::from(day),
            ),
            true,
            true,
        ),
        _ => return Err(INVALID_DATE.to_owned()),
    };
    // A month or day of 0 would be interpreted as missing.
    if !date.is_valid() || (has_month && date.month() == 0) || (has_day && date.day_of_month() == 0)
    {
        return Err(format!("Invalid date: {input}"));
    }
    Ok(Some(date.into()))
}

fn parse_bpm(input: &str) -> Result<Option<TempoBpm>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(None);
    }
    let bpm = input
        .parse()
        .ok()
        .map(TempoBpm::new)
        .filter(TempoBpm::is_valid)
        .ok_or_else(|| format!("Invalid BPM: {input}"))?;
    Ok(Some(bpm))
}

fn parse_key(input: &str) -> Result<Option<KeySignature>, String> {
    let input = input.trim().to_uppercase();
    if input.is_empty() {
        return Ok(None);
    }
    let code = KeyCode::try_from_lancelot_str(&input)
        .filter(|code| *code != KeyCode::Off)
        .ok_or_else(|| format!("Invalid Lancelot key: {input}"))?;
    Ok(Some(KeySignature::new(code)))
}

#[must_use]
fn format_color(color: Color) -> String {
    match color {
        Color::Rgb(rgb_color) => rgb_color.to_string(),
        Color::Index(index) => index.to_string(),
    }
}

fn parse_color(input: &str) -> Result<Option<Color>, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(None);
    }
    if let Ok(index) = input.parse::<ColorIndex>() {
        return Ok(Some(Color::Index(index)));
    }
    let rgb_color = input
        .to_uppercase()
        .parse::<RgbColor>()
        .map_err(|_| format!("Expected #RRGGBB or a color index: {input}"))?;
    Ok(Some(Color::Rgb(rgb_color)))
}

#[cfg(test)]
mod tests;
