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

use std::{borrow::Cow, convert::TryFrom as _, fmt, path::Path, str::FromStr};

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
    media::artwork::{
        ApicType, Artwork, ArtworkImage, EmbeddedArtwork, ImageDimension, ImageSize,
        Thumbnail4x4Rgb8,
    },
    music::{
        key::{KeyCode, KeySignature},
        tempo::TempoBpm,
    },
    track::{
        actor::{is_valid_summary_individual_actor_name, Actor, ActorKind, ActorRole},
        index::Index,
        title::{Title, TitleKind},
    },
    util::{
        clock::{DateOrDateTime, DateTime, DateYYYYMMDD, YYYYMMDD},
        string::{trimmed_non_empty_from, trimmed_non_empty_from_owned},
    },
};
use time::{
    format_description::{
        well_known::{Rfc2822, Rfc3339},
        FormatItem,
    },
    OffsetDateTime, PrimitiveDateTime,
};

use crate::{io::import::Importer, prelude::*};

use self::digest::MediaDigest;

pub mod digest;
pub mod serato;
pub mod tag;

#[must_use]
pub fn trim_readable(input: &str) -> &str {
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
/// role is adjusted from Summary to Individual, because Summary is singular.
/// Otherwise a new chunk of actors is started, starting with the kind
/// Summary.
fn adjust_summary_actor_kind(actors: &mut [Actor], role: ActorRole, next_name: &str) -> ActorKind {
    // Precodinition: Coherent chunk of actors with the given role at the back of the slice
    debug_assert_eq!(
        actors.iter().filter(|actor| actor.role == role).count(),
        actors
            .iter()
            .rev()
            .take_while(|actor| actor.role == role)
            .count(),
    );
    let proposed_kind = {
        let summary_actor = actors
            .iter_mut()
            .rev()
            // Terminate the iteration if the role changes,
            // i.e. assume coherent chunks of equal roles and
            // a single chunke per role!
            .take_while(|actor| actor.role == role)
            .find(|actor| actor.kind == ActorKind::Summary);
        if let Some(summary_actor) = summary_actor {
            if is_valid_summary_individual_actor_name(&summary_actor.name, next_name) {
                ActorKind::Individual
            } else {
                // Turn the current summary actor into an individual actor
                summary_actor.kind = ActorKind::Individual;
                // Check if the next actor could become the new summary actor
                if is_valid_summary_individual_actor_name(next_name, &summary_actor.name) {
                    ActorKind::Summary
                } else {
                    ActorKind::Individual
                }
            }
        } else {
            // No summary actor for this role yet
            if actors
                .iter()
                .rev()
                .take_while(|actor| actor.role == role)
                .filter(|actor| actor.kind == ActorKind::Individual)
                .all(|actor| next_name.contains(&actor.name))
            {
                ActorKind::Summary
            } else {
                ActorKind::Individual
            }
        }
    };
    match proposed_kind {
        ActorKind::Individual => {
            debug_assert!(actors.iter().any(|actor| actor.role == role));
            debug_assert!(
                actors
                    .iter()
                    .filter(|actor| actor.role == role && actor.kind == ActorKind::Summary)
                    .count()
                    <= 1
            );
        }
        ActorKind::Summary => {
            debug_assert!(!actors
                .iter()
                .any(|actor| actor.role == role && actor.kind == ActorKind::Summary));
            debug_assert!(actors
                .iter()
                .filter(|actor| actor.role == role && actor.kind == ActorKind::Individual)
                .all(|actor| next_name.contains(&actor.name)));
        }
        _ => unreachable!(),
    }
    proposed_kind
}

pub fn push_next_actor_role_name(actors: &mut Vec<Actor>, role: ActorRole, name: String) -> bool {
    if let Some(mut actor) = ingest_actor_from_owned(name, Default::default(), role) {
        actor.kind = adjust_summary_actor_kind(actors.as_mut_slice(), role, &actor.name);
        actors.push(actor);
        true
    } else {
        false
    }
}

pub fn push_next_actor_role_name_from<'a>(
    actors: &mut Vec<Actor>,
    role: ActorRole,
    name: impl Into<Cow<'a, str>>,
) -> bool {
    if let Some(mut actor) = ingest_actor_from(name, Default::default(), role) {
        actor.kind = adjust_summary_actor_kind(actors.as_mut_slice(), role, &actor.name);
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

#[must_use]
pub fn db2lufs(relative_gain_db: f64) -> LoudnessLufs {
    // Reconstruct the LUFS value from the relative gain
    LoudnessLufs(EBU_R128_REFERENCE_LUFS - relative_gain_db)
}

#[must_use]
pub fn lufs2db(loudness: LoudnessLufs) -> f64 {
    EBU_R128_REFERENCE_LUFS - loudness.0
}

#[must_use]
pub fn format_valid_replay_gain(loudness: LoudnessLufs) -> Option<String> {
    LoudnessLufs::validated_from(loudness).ok().map(|loudness| {
        let mut replay_gain_db = lufs2db(*loudness);
        let formatted = format!("{}, dB", format_parseable_value(&mut replay_gain_db));
        let mut importer = Importer::new();
        debug_assert_eq!(
            Some(db2lufs(replay_gain_db)),
            importer.import_replay_gain(&formatted)
        );
        debug_assert!(importer.finish().into_messages().is_empty());
        formatted
    })
}

pub fn parse_replay_gain_db(input: &str) -> IResult<&str, f64> {
    let mut parser = separated_pair(
        preceded(space0, double),
        space0,
        terminated(tag_no_case("dB"), space0),
    );
    let (input, (replay_gain_db, _)) = parser(input)?;
    Ok((input, replay_gain_db))
}

pub fn format_validated_tempo_bpm(tempo_bpm: &mut Option<TempoBpm>) -> Option<String> {
    let validated_tempo_bpm = tempo_bpm
        .map(TempoBpm::validated_from)
        .transpose()
        .ok()
        .flatten();
    *tempo_bpm = validated_tempo_bpm.map(|validated| *validated);
    tempo_bpm.as_mut().map(format_tempo_bpm)
}

pub fn format_tempo_bpm(tempo_bpm: &mut TempoBpm) -> String {
    let formatted_bpm = format_parseable_value(&mut tempo_bpm.to_raw());
    let mut importer = Importer::new();
    debug_assert_eq!(Some(*tempo_bpm), importer.import_tempo_bpm(&formatted_bpm));
    debug_assert!(importer.finish().into_messages().is_empty());
    formatted_bpm
}

#[must_use]
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
    None
}

const RFC3339_WITHOUT_TZ_FORMAT: &[FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");

const RFC3339_WITHOUT_T_TZ_FORMAT: &[FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub fn parse_year_tag(value: &str) -> Option<DateOrDateTime> {
    let input = value.trim();
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
    if let Ok(date_time) =
        OffsetDateTime::parse(input, &Rfc3339).or_else(|_| OffsetDateTime::parse(input, &Rfc2822))
    {
        return Some(DateTime::new(date_time).into());
    }
    if let Ok(date_time) = PrimitiveDateTime::parse(input, RFC3339_WITHOUT_TZ_FORMAT)
        .or_else(|_| PrimitiveDateTime::parse(input, RFC3339_WITHOUT_T_TZ_FORMAT))
    {
        // Assume UTC if time zone is missing
        return Some(DateTime::from(date_time.assume_utc()).into());
    }
    // Replace arbitrary whitespace by a single space and try again
    let recombined = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if recombined != input {
        return parse_year_tag(&recombined);
    }
    None
}

#[must_use]
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
                "Unsupported artwork image format: {image_format:?}"
            )),
            ArtworkImageError::Other(err) => Self::Other(err),
        }
    }
}

#[derive(Debug)]
pub struct LoadedArtworkPicture {
    pub media_type: Mime,
    pub picture: DynamicImage,
    pub recoverable_errors: Vec<anyhow::Error>,
}

pub type LoadArtworkPictureResult = std::result::Result<LoadedArtworkPicture, ArtworkImageError>;

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

pub fn load_artwork_picture(
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
) -> LoadArtworkPictureResult {
    let image_format = image_format_hint.or_else(|| guess_format(image_data).ok());
    let mut recoverable_errors = Vec::new();
    if let Some(image_format) = image_format {
        load_from_memory_with_format(image_data, image_format)
    } else {
        load_from_memory(image_data)
    }
    .map_err(Into::into)
    .map_err(ArtworkImageError::Other)
    .and_then(|picture| {
        let media_type = media_type_hint
            .and_then(|media_type_hint| {
                media_type_hint
                    .parse::<Mime>()
                    .map_err(|err| {
                        recoverable_errors.push(anyhow::anyhow!(
                            "Invalid media type hint '{media_type_hint}': {err}"
                        ));
                        err
                    })
                    // Ignore and continue
                    .ok()
            })
            .map(Ok)
            .or_else(|| image_format.map(media_type_from_image_format))
            .transpose()?
            .unwrap_or(IMAGE_STAR);
        Ok(LoadedArtworkPicture {
            media_type,
            picture,
            recoverable_errors,
        })
    })
}

#[derive(Debug)]
pub struct IngestedArtworkImage {
    pub artwork_image: ArtworkImage,
    pub picture: DynamicImage,
    pub recoverable_errors: Vec<anyhow::Error>,
}

pub type IngestArtworkImageResult = std::result::Result<IngestedArtworkImage, ArtworkImageError>;

pub fn ingest_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
    image_digest: &mut MediaDigest,
) -> IngestArtworkImageResult {
    let LoadedArtworkPicture {
        media_type,
        picture,
        recoverable_errors,
    } = load_artwork_picture(image_data, image_format_hint, media_type_hint)?;
    let (width, height) = picture.dimensions();
    let clamped_with = width as ImageDimension;
    let clamped_height = height as ImageDimension;
    if width != clamped_with as u32 && height != clamped_height as u32 {
        return Err(ArtworkImageError::Other(anyhow::anyhow!(
            "Unsupported image size: {width}x{height}"
        )));
    }
    let size = ImageSize {
        width: clamped_with,
        height: clamped_height,
    };
    let digest = image_digest.digest_content(image_data).finalize_reset();
    let picture_4x4 = picture.resize_exact(4, 4, image::imageops::FilterType::Lanczos3);
    let thumbnail = Thumbnail4x4Rgb8::try_from(picture_4x4.to_rgb8().into_raw()).ok();
    debug_assert!(thumbnail.is_some());
    let artwork_image = ArtworkImage {
        media_type,
        apic_type,
        size: Some(size),
        digest,
        thumbnail,
    };
    Ok(IngestedArtworkImage {
        artwork_image,
        picture,
        recoverable_errors,
    })
}

#[derive(Debug)]
pub struct IngestedEmbeddedArtworkImage {
    pub embedded_artwork: EmbeddedArtwork,
    pub picture: DynamicImage,
    pub recoverable_errors: Vec<anyhow::Error>,
}

pub type IngestEmbeddedArtworkImageResult =
    std::result::Result<IngestedEmbeddedArtworkImage, ArtworkImageError>;

pub fn ingest_embedded_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
    image_digest: &mut MediaDigest,
) -> IngestEmbeddedArtworkImageResult {
    let IngestedArtworkImage {
        artwork_image,
        picture,
        recoverable_errors,
    } = ingest_artwork_image(
        apic_type,
        image_data,
        image_format_hint,
        media_type_hint,
        image_digest,
    )?;
    let embedded_artwork = EmbeddedArtwork {
        image: artwork_image,
    };
    Ok(IngestedEmbeddedArtworkImage {
        embedded_artwork,
        picture,
        recoverable_errors,
    })
}

pub fn try_ingest_embedded_artwork_image(
    apic_type: ApicType,
    image_data: &[u8],
    image_format_hint: Option<ImageFormat>,
    media_type_hint: Option<String>,
    image_digest: &mut MediaDigest,
) -> (Artwork, Option<DynamicImage>, Vec<String>) {
    ingest_embedded_artwork_image(
        apic_type,
        image_data,
        image_format_hint,
        media_type_hint,
        image_digest,
    )
    .map(
        |IngestedEmbeddedArtworkImage {
             embedded_artwork,
             picture,
             recoverable_errors,
         }| {
            let issues = recoverable_errors
                .into_iter()
                .map(|err| {
                    format!(
                        "Recoverable error while loading embedded {apic_type:?} artwork image: {err}"
                    )
                })
                .collect();
            (Artwork::Embedded(embedded_artwork), Some(picture), issues)
        },
    )
    .unwrap_or_else(|err| match err {
        ArtworkImageError::UnsupportedFormat(unsupported_format) => {
            let issue = format!("Unsupported image format: {unsupported_format:?}");
            (Artwork::Unsupported, None, vec![issue])
        }
        ArtworkImageError::Other(err) => {
            let issue = format!("Failed to load embedded artwork image: {err}");
            (Artwork::Irregular, None, vec![issue])
        }
    })
}

pub fn ingest_title_from<'a>(name: impl Into<Cow<'a, str>>, kind: TitleKind) -> Option<Title> {
    trimmed_non_empty_from(name).map(|name| Title {
        name: name.into(),
        kind,
    })
}

#[must_use]
pub fn ingest_title_from_owned(name: String, kind: TitleKind) -> Option<Title> {
    trimmed_non_empty_from_owned(name).map(|name| Title {
        name: name.into(),
        kind,
    })
}

pub fn ingest_actor_from<'a>(
    name: impl Into<Cow<'a, str>>,
    kind: ActorKind,
    role: ActorRole,
) -> Option<Actor> {
    trimmed_non_empty_from(name).map(|name| Actor {
        name: name.into(),
        kind,
        role,
        role_notes: None,
    })
}

#[must_use]
pub fn ingest_actor_from_owned(name: String, kind: ActorKind, role: ActorRole) -> Option<Actor> {
    trimmed_non_empty_from_owned(name).map(|name| Actor {
        name: name.into(),
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
