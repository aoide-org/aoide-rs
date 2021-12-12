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

use std::{convert::TryFrom as _, fmt, path::Path, str::FromStr};

use anyhow::Context as _;
use chrono::{NaiveDateTime, Utc};
use image::{
    guess_format, load_from_memory, load_from_memory_with_format, DynamicImage, GenericImageView,
    ImageFormat,
};
use mime::{Mime, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG, IMAGE_STAR};
use nom::{
    bytes::complete::{tag, tag_no_case},
    character::complete::{digit1, space0},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};
use semval::{IsValid as _, ValidatedFrom as _};

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{
        ApicType, Artwork, ArtworkImage, EmbeddedArtwork, ImageDimension, ImageSize, SourcePath,
        Thumbnail4x4Rgb8,
    },
    music::{
        key::{KeyCode, KeySignature},
        time::TempoBpm,
    },
    track::{
        actor::{Actor, ActorKind, ActorRole},
        index::Index,
        release::DateOrDateTime,
        title::{Title, TitleKind},
    },
    util::clock::{DateTime, DateTimeInner, DateYYYYMMDD, YYYYMMDD},
};

use crate::prelude::*;

use self::digest::MediaDigest;

pub mod digest;
pub mod serato;
pub mod tag;

fn trim_readable(input: &str) -> &str {
    input.trim_matches(|c: char| c.is_whitespace() || c.is_control())
}

pub fn guess_mime_from_path(path: impl AsRef<Path>) -> Result<Mime> {
    let mime_guess = mime_guess::from_path(path);
    if mime_guess.first().is_none() {
        return Err(Error::UnknownContentType);
    }
    mime_guess
        .iter()
        .filter(|mime| mime.type_() == mime::AUDIO)
        .chain(mime_guess.iter().filter(|mime| mime.type_() == mime::VIDEO))
        .next()
        .ok_or(Error::UnknownContentType)
}

/// Determines the next kind and adjusts the previous kind.
///
/// The `actors` slice must contain continues chunks of the same role,
/// at most a single chunk per role.
///
/// If the last chunk matches the given role then it is continued and the
/// role is adjusted from Summary to Primary, because Summary is singular.
/// Otherwise a new chunk of actors is started, starting with the kind
/// Summary.
fn adjust_last_actor_kind(actors: &mut [Actor], role: ActorRole) -> ActorKind {
    if let Some(last_actor) = actors.last_mut() {
        if last_actor.role == role {
            // ActorKind::Summary is only allowed once for each role
            last_actor.kind = ActorKind::Primary;
            return ActorKind::Primary;
        }
    }
    ActorKind::Summary
}

pub fn push_next_actor_role_name(
    actors: &mut Vec<Actor>,
    role: ActorRole,
    name: impl AsRef<str> + Into<String>,
) -> bool {
    if let Some(mut actor) = import_actor(name, Default::default(), role) {
        actor.kind = adjust_last_actor_kind(actors.as_mut_slice(), role);
        actors.push(actor);
        true
    } else {
        false
    }
}

pub fn format_parseable_value<T>(value: &mut T) -> String
where
    T: Copy + PartialEq + ToString + FromStr,
    <T as FromStr>::Err: fmt::Debug,
{
    // Iron out rounding errors that occur due to string formatting
    // by repeated formatting and parsing until the values converge.
    let mut value_formatted;
    loop {
        value_formatted = value.to_string();
        let value_parsed = value_formatted.parse().expect("valid value");
        if value_parsed == *value {
            break;
        }
        *value = value_parsed;
    }
    value_formatted
}

// Assumption: Gain has been calculated with the EBU R128 algorithm
const EBU_R128_REFERENCE_LUFS: f64 = -18.0;

fn db2lufs(relative_gain_db: f64) -> LoudnessLufs {
    // Reconstruct the LUFS value from the relative gain
    LoudnessLufs(EBU_R128_REFERENCE_LUFS - relative_gain_db)
}

fn lufs2db(loudness: LoudnessLufs) -> f64 {
    EBU_R128_REFERENCE_LUFS - loudness.0
}

pub fn format_valid_replay_gain(loudness: LoudnessLufs) -> Option<String> {
    LoudnessLufs::validated_from(loudness).ok().map(|loudness| {
        let mut replay_gain_db = lufs2db(loudness);
        let formatted = format!("{}, dB", format_parseable_value(&mut replay_gain_db));
        debug_assert_eq!(Some(db2lufs(replay_gain_db)), parse_replay_gain(&formatted));
        formatted
    })
}

fn parse_replay_gain_db(input: &str) -> IResult<&str, f64> {
    let mut parser = separated_pair(
        preceded(space0, double),
        space0,
        terminated(tag_no_case("dB"), space0),
    );
    let (input, (replay_gain_db, _)) = parser(input)?;
    Ok((input, replay_gain_db))
}

pub fn parse_replay_gain(input: &str) -> Option<LoudnessLufs> {
    let input = trim_readable(input);
    if input.is_empty() {
        return None;
    }
    match parse_replay_gain_db(input) {
        Ok((remainder, relative_gain_db)) => {
            if !remainder.is_empty() {
                tracing::warn!(
                    "Unexpected remainder '{}' after parsing replay gain input '{}'",
                    remainder,
                    input
                );
            }
            let loudness_lufs = db2lufs(relative_gain_db);
            if !loudness_lufs.is_valid() {
                tracing::warn!(
                    "Invalid loudness parsed from replay gain input '{}': {}",
                    input,
                    loudness_lufs
                );
                return None;
            }
            tracing::debug!(
                "Parsed loudness from replay gain input '{}': {}",
                input,
                loudness_lufs
            );
            Some(loudness_lufs)
        }
        Err(err) => {
            // Silently ignore any 0 values
            if input.parse().ok() == Some(0.0) {
                tracing::debug!(
                    "Ignoring invalid replay gain (dB) from input '{}': {}",
                    input,
                    err
                );
            } else {
                tracing::warn!(
                    "Failed to parse replay gain (dB) from input '{}': {}",
                    input,
                    err
                );
            }
            None
        }
    }
}

pub fn parse_tempo_bpm(input: &str) -> Option<TempoBpm> {
    let input = trim_readable(input);
    if input.is_empty() {
        return None;
    }
    match input.parse() {
        Ok(bpm) => {
            let tempo_bpm = TempoBpm(bpm);
            if !tempo_bpm.is_valid() {
                // The value 0 is often used for an unknown bpm.
                // Silently ignore this special value to prevent log spam.
                if bpm != 0.0 {
                    tracing::info!("Invalid tempo parsed from input '{}': {}", input, tempo_bpm);
                }
                return None;
            }
            tracing::debug!("Parsed tempo from input '{}': {}", input, tempo_bpm);
            Some(tempo_bpm)
        }
        Err(err) => {
            tracing::warn!(
                "Failed to parse tempo (BPM) from input '{}': {}",
                input,
                err
            );
            None
        }
    }
}

pub fn format_validated_tempo_bpm(tempo_bpm: &mut Option<TempoBpm>) -> Option<String> {
    *tempo_bpm = tempo_bpm
        .map(TempoBpm::validated_from)
        .transpose()
        .ok()
        .flatten();
    tempo_bpm.as_mut().map(format_tempo_bpm)
}

pub fn format_tempo_bpm(tempo_bpm: &mut TempoBpm) -> String {
    let formatted_bpm = format_parseable_value(&mut tempo_bpm.0);
    debug_assert_eq!(Some(*tempo_bpm), parse_tempo_bpm(&formatted_bpm));
    formatted_bpm
}

pub fn parse_key_signature(input: &str) -> Option<KeySignature> {
    let input = trim_readable(input);
    if input.is_empty() {
        return None;
    }
    if input.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        if input.starts_with(|c: char| c.is_ascii_digit()) {
            let key_code = KeyCode::from_lancelot_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
            let key_code = KeyCode::from_openkey_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
        } else {
            let key_code = KeyCode::from_canonical_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
            let key_code = KeyCode::from_traditional_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
            let key_code = KeyCode::from_traditional_ascii_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
            let key_code = KeyCode::from_serato_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
            let key_code = KeyCode::from_beatport_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
            let key_code = KeyCode::from_traxsource_str(input);
            if key_code != KeyCode::Unknown {
                return Some(key_code.into());
            }
        }
    }
    tracing::warn!(
        "Failed to parse musical key signature from input (UTF-8 bytes): '{}' ({:X?})",
        input,
        input.as_bytes()
    );
    None
}

pub fn parse_year_tag(input: &str) -> Option<DateOrDateTime> {
    let input = input.trim();
    let mut digits_parser = delimited(space0, digit1, space0);
    let digits_parsed: IResult<_, _> = digits_parser(input);
    if let Ok((remainder, digits_input)) = digits_parsed {
        if remainder.is_empty()
            && (/*YYYY*/digits_input.len() == 4 ||
            /*YYYYMM*/ digits_input.len() == 6 ||
            /*YYYYMMDD*/ digits_input.len() == 8)
        {
            if let Ok(yyyymmdd) =
                digits_input
                    .parse::<YYYYMMDD>()
                    .map(|val| match digits_input.len() {
                        4 => val * 10000,
                        6 => val * 100,
                        8 => val,
                        _ => unreachable!(),
                    })
            {
                let date = DateYYYYMMDD::new(yyyymmdd);
                if date.is_valid() {
                    return Some(date.into());
                }
            }
        }
    }
    let mut year_month_parser = separated_pair(
        delimited(space0, digit1, space0),
        tag("-"),
        delimited(space0, digit1, space0),
    );
    let year_month_parsed: IResult<_, _> = year_month_parser(input);
    if let Ok((remainder, (year_input, month_input))) = year_month_parsed {
        if year_input.len() == 4 && month_input.len() <= 2 {
            if let (Ok(year), Ok(month)) = (
                year_input.parse::<YYYYMMDD>(),
                month_input.parse::<YYYYMMDD>(),
            ) {
                if remainder.is_empty() {
                    let date = DateYYYYMMDD::new(year * 10000 + month * 100);
                    if date.is_valid() {
                        return Some(date.into());
                    }
                }
                let mut day_of_month_parser = delimited(pair(tag("-"), space0), digit1, space0);
                let day_of_month_parsed: IResult<_, _> = day_of_month_parser(remainder);
                if let Ok((remainder, day_of_month_input)) = day_of_month_parsed {
                    if remainder.is_empty() {
                        if let Ok(day_of_month) = day_of_month_input.parse::<YYYYMMDD>() {
                            if (0..=31).contains(&day_of_month) {
                                let date =
                                    DateYYYYMMDD::new(year * 10000 + month * 100 + day_of_month);
                                if date.is_valid() {
                                    return Some(date.into());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(datetime) = input.parse::<DateTimeInner>() {
        return Some(DateTime::from(datetime).into());
    }
    if let Ok(datetime) = input.parse::<NaiveDateTime>() {
        // Assume UTC if time zone is missing
        let datetime_utc: chrono::DateTime<Utc> = chrono::DateTime::from_utc(datetime, Utc);
        return Some(DateTime::from(datetime_utc).into());
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
        // Assume UTC if time zone is missing
        let datetime_utc: chrono::DateTime<Utc> = chrono::DateTime::from_utc(datetime, Utc);
        return Some(DateTime::from(datetime_utc).into());
    }
    tracing::warn!("Year tag not recognized: {}", input);
    None
}

pub fn parse_index_numbers(input: &str) -> Option<Index> {
    let mut split = if input.contains('/') {
        input.split('/')
    } else if input.contains('-') {
        input.split('-')
    } else {
        return input.parse().ok().map(|number| Index {
            number: Some(number),
            total: None,
        });
    };
    let number = split.next().and_then(|input| input.parse().ok());
    let total = split.next().and_then(|input| input.parse().ok());
    if number.is_none() && total.is_none() {
        None
    } else {
        Some(Index { number, total })
    }
}

#[derive(Debug, Error)]
pub enum ArtworkImageError {
    #[error("unsupported format {0:?}")]
    UnsupportedFormat(ImageFormat),

    #[error(transparent)]
    Other(anyhow::Error),
}

impl From<ArtworkImageError> for Error {
    fn from(err: ArtworkImageError) -> Error {
        match err {
            ArtworkImageError::UnsupportedFormat(image_format) => Self::Other(anyhow::anyhow!(
                "Unsupported artwork image format: {:?}",
                image_format
            )),
            ArtworkImageError::Other(err) => Self::Other(err),
        }
    }
}

pub type LoadArtworkImageResult = std::result::Result<(Mime, DynamicImage), ArtworkImageError>;

pub fn media_type_from_image_format(
    image_format: ImageFormat,
) -> std::result::Result<Mime, ArtworkImageError> {
    let media_type = match image_format {
        ImageFormat::Jpeg => IMAGE_JPEG,
        ImageFormat::Png => IMAGE_PNG,
        ImageFormat::Gif => IMAGE_GIF,
        ImageFormat::Bmp => IMAGE_BMP,
        ImageFormat::WebP => "image/webp".parse().unwrap(),
        ImageFormat::Tiff => "image/tiff".parse().unwrap(),
        ImageFormat::Tga => "image/tga".parse().unwrap(),
        unsupported_format => {
            return Err(ArtworkImageError::UnsupportedFormat(unsupported_format));
        }
    };
    Ok(media_type)
}

pub fn load_artwork_image(
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
) -> LoadArtworkImageResult {
    let image_format = image_format_hint.or_else(|| guess_format(image_data).ok());
    if let Some(image_format) = image_format {
        load_from_memory_with_format(image_data, image_format)
    } else {
        load_from_memory(image_data)
    }
    .with_context(|| "Failed to load embedded artwork image")
    .map_err(ArtworkImageError::Other)
    .and_then(|image| {
        let media_type = if let Some(media_type_hint) = media_type_hint {
            media_type_hint
                .parse()
                .map_err(anyhow::Error::from)
                .map_err(ArtworkImageError::Other)?
        } else if let Some(image_format) = image_format {
            media_type_from_image_format(image_format)?
        } else {
            IMAGE_STAR
        };
        Ok((media_type, image))
    })
}

pub type IngestArtworkImageResult =
    std::result::Result<(ArtworkImage, DynamicImage), ArtworkImageError>;

pub fn ingest_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
    image_digest: &mut MediaDigest,
) -> IngestArtworkImageResult {
    let (media_type, image) = load_artwork_image(image_data, image_format_hint, media_type_hint)?;
    let (width, height) = image.dimensions();
    let clamped_with = width as ImageDimension;
    let clamped_height = height as ImageDimension;
    if width != clamped_with as u32 && height != clamped_height as u32 {
        return Err(ArtworkImageError::Other(anyhow::anyhow!(
            "Unsupported image size: {}x{}",
            width,
            height
        )));
    }
    let size = ImageSize {
        width: clamped_with,
        height: clamped_height,
    };
    let digest = image_digest.digest_content(image_data).finalize_reset();
    let image_4x4 = image.resize_exact(4, 4, image::imageops::FilterType::Lanczos3);
    let thumbnail = Thumbnail4x4Rgb8::try_from(image_4x4.to_rgb8().into_raw()).ok();
    debug_assert!(thumbnail.is_some());
    Ok((
        ArtworkImage {
            media_type,
            apic_type,
            size: Some(size),
            digest,
            thumbnail,
        },
        image,
    ))
}

pub type IngestLoadedArtworkImageResult =
    std::result::Result<(EmbeddedArtwork, DynamicImage), ArtworkImageError>;

pub fn ingest_embedded_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
    image_digest: &mut MediaDigest,
) -> IngestLoadedArtworkImageResult {
    ingest_artwork_image(
        apic_type,
        image_data,
        image_format_hint,
        media_type_hint,
        image_digest,
    )
    .map(|(artwork_image, dynamic_image)| {
        (
            EmbeddedArtwork {
                image: artwork_image,
            },
            dynamic_image,
        )
    })
}

pub fn try_ingest_embedded_artwork_image(
    media_source_path: &SourcePath,
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
    image_digest: &mut MediaDigest,
) -> (Artwork, Option<DynamicImage>) {
    ingest_embedded_artwork_image(
        apic_type,
        image_data,
        image_format_hint,
        media_type_hint,
        image_digest,
    )
    .map(|(embedded, image)| (Artwork::Embedded(embedded), Some(image)))
    .unwrap_or_else(|err| match err {
        ArtworkImageError::UnsupportedFormat(unsupported_format) => {
            tracing::info!(
                "Unsupported image format in {}: {:?}",
                media_source_path,
                unsupported_format
            );
            (Artwork::Unsupported, None)
        }
        ArtworkImageError::Other(err) => {
            tracing::warn!(
                "Failed to load embedded artwork image from {}: {}",
                media_source_path,
                err
            );
            (Artwork::Irregular, None)
        }
    })
}

pub fn import_trimmed_name(name: impl AsRef<str> + Into<String>) -> Option<String> {
    let trimmed_name = name.as_ref().trim();
    if trimmed_name.is_empty() {
        return None;
    }
    let name = if trimmed_name == name.as_ref() {
        name.into()
    } else {
        trimmed_name.to_owned()
    };
    Some(name)
}

pub fn import_title(name: impl AsRef<str> + Into<String>, kind: TitleKind) -> Option<Title> {
    import_trimmed_name(name).map(|name| Title { name, kind })
}

pub fn import_actor(
    name: impl AsRef<str> + Into<String>,
    kind: ActorKind,
    role: ActorRole,
) -> Option<Actor> {
    import_trimmed_name(name).map(|name| Actor {
        name,
        kind,
        role,
        role_notes: None,
    })
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
