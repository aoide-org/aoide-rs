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

pub mod digest;
pub mod serato;
pub mod tag;

use crate::prelude::*;

use self::digest::MediaDigest;

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::{Artwork, ImageDimension, ImageSize, Thumbnail4x4Rgb8},
    music::{
        key::{KeyCode, KeySignature},
        time::TempoBpm,
    },
    track::{
        actor::{Actor, ActorKind, ActorRole},
        index::Index,
        release::DateOrDateTime,
    },
    util::clock::{DateTime, DateTimeInner, DateYYYYMMDD, YYYYMMDD},
};

use chrono::{NaiveDateTime, Utc};
use image::{
    guess_format, load_from_memory, load_from_memory_with_format, GenericImageView, ImageFormat,
};
use mime::{Mime, IMAGE_BMP, IMAGE_GIF, IMAGE_JPEG, IMAGE_PNG, IMAGE_STAR};
use nom::{
    bytes::complete::{tag, tag_no_case},
    character::complete::{digit1, space0},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};
use semval::IsValid as _;
use std::{convert::TryFrom as _, path::Path};

pub fn guess_mime_from_path(path: impl AsRef<Path>) -> Result<Mime> {
    let mime_guess = mime_guess::from_path(path);
    if mime_guess.first().is_none() {
        return Err(Error::UnknownContentType);
    }
    mime_guess
        .clone()
        .into_iter()
        .filter(|mime| mime.type_() == mime::AUDIO)
        .chain(
            mime_guess
                .into_iter()
                .filter(|mime| mime.type_() == mime::VIDEO),
        )
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

pub fn push_next_actor_role_name(actors: &mut Vec<Actor>, role: ActorRole, name: String) {
    let kind = adjust_last_actor_kind(actors.as_mut_slice(), role);
    let actor = Actor {
        name,
        kind,
        role,
        role_notes: None,
    };
    actors.push(actor);
}

// Assumption: Gain has been calculated with the EBU R128 algorithm
const EBU_R128_REFERENCE_LUFS: f64 = -18.0;

fn db2lufs(relative_gain_db: f64) -> LoudnessLufs {
    // Reconstruct the LUFS value from the relative gain
    LoudnessLufs(EBU_R128_REFERENCE_LUFS - relative_gain_db)
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
    match parse_replay_gain_db(input) {
        Ok((remainder, relative_gain_db)) => {
            if !remainder.is_empty() {
                log::warn!(
                    "Unexpected remainder '{}' after parsing replay gain input '{}'",
                    remainder,
                    input
                );
            }
            let loudness_lufs = db2lufs(relative_gain_db);
            if !loudness_lufs.is_valid() {
                log::warn!(
                    "Invalid loudness parsed from replay gain input '{}': {}",
                    input,
                    loudness_lufs
                );
                return None;
            }
            log::debug!(
                "Parsed loudness from replay gain input '{}': {}",
                input,
                loudness_lufs
            );
            Some(loudness_lufs)
        }
        Err(err) => {
            log::warn!(
                "Failed to parse replay gain (dB) from input '{}': {}",
                input,
                err
            );
            None
        }
    }
}

pub fn parse_tempo_bpm(input: &str) -> Option<TempoBpm> {
    match input.parse() {
        Ok(bpm) => {
            let tempo_bpm = TempoBpm(bpm);
            if !tempo_bpm.is_valid() {
                log::info!("Invalid tempo parsed from input '{}': {}", input, tempo_bpm);
                return None;
            }
            log::debug!("Parsed tempo from input '{}': {}", input, tempo_bpm);
            Some(tempo_bpm)
        }
        Err(err) => {
            log::warn!(
                "Failed to parse tempo (BPM) from input '{}': {}",
                input,
                err
            );
            None
        }
    }
}

pub fn parse_key_signature(input: &str) -> Option<KeySignature> {
    let input = input.trim();
    let key_code = KeyCode::from_lancelot_str(input);
    if key_code != KeyCode::Unknown {
        return Some(key_code.into());
    }
    let key_code = KeyCode::from_openkey_str(input);
    if key_code != KeyCode::Unknown {
        return Some(key_code.into());
    }
    let key_code = KeyCode::from_traditional_str(input);
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
    log::warn!("Failed to parse key signature from input '{}'", input);
    None
}

pub fn parse_year_tag(input: &str) -> Option<DateOrDateTime> {
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
    log::warn!("Year tag not recognized: {}", input);
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

pub fn parse_artwork_from_embedded_image(
    image_data: &[u8],
    image_format: Option<ImageFormat>,
    image_digest: &mut MediaDigest,
) -> Option<Artwork> {
    let image_format = image_format.or_else(|| guess_format(image_data).ok());
    let media_type = match image_format {
        Some(ImageFormat::Jpeg) => IMAGE_JPEG.to_string(),
        Some(ImageFormat::Png) => IMAGE_PNG.to_string(),
        Some(ImageFormat::Gif) => IMAGE_GIF.to_string(),
        Some(ImageFormat::Bmp) => IMAGE_BMP.to_string(),
        Some(ImageFormat::WebP) => "image/webp".to_string(),
        Some(ImageFormat::Tiff) => "image/tiff".to_string(),
        Some(ImageFormat::Tga) => "image/tga".to_string(),
        Some(format) => {
            log::info!("Unusual image format {:?}", format);
            IMAGE_STAR.to_string()
        }
        None => {
            log::info!("Unknown image format");
            IMAGE_STAR.to_string()
        }
    };
    if let Some(format) = image_format {
        load_from_memory_with_format(image_data, format)
    } else {
        load_from_memory(image_data)
    }
    .map_err(|err| {
        log::warn!("Failed to load image: {}", err);
        err
    })
    .ok()
    .and_then(|image| {
        let (width, height) = image.dimensions();
        let clamped_with = width as ImageDimension;
        let clamped_height = height as ImageDimension;
        if width != clamped_with as u32 && height != clamped_height as u32 {
            log::warn!("Unsupported image size: {}x{}", width, height);
            return None;
        }
        let size = ImageSize {
            width: clamped_with,
            height: clamped_height,
        };
        let digest = image_digest.digest_content(image_data);
        let image_4x4 = image.resize_exact(4, 4, image::imageops::FilterType::Lanczos3);
        let thumbnail = Thumbnail4x4Rgb8::try_from(image_4x4.to_rgb8().into_raw()).ok();
        debug_assert!(thumbnail.is_some());
        Some(Artwork {
            size: Some(size),
            digest,
            thumbnail,
            media_type: Some(media_type),
            uri: None, // embedded
        })
    })
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
