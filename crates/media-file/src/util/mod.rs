// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, fmt, path::Path, str::FromStr};

use aoide_core::{
    audio::signal::LoudnessLufs,
    music::{
        key::{KeyCode, KeySignature},
        tempo::TempoBpm,
    },
    prelude::*,
    track::{
        actor::{
            is_valid_summary_individual_actor_name, Actor, Actors, Kind as ActorKind,
            Role as ActorRole,
        },
        title::{Kind as TitleKind, Title},
    },
    util::{
        clock::{DateOrDateTime, DateTime, DateYYYYMMDD, YYYYMMDD},
        string::{trimmed_non_empty_from, trimmed_non_empty_from_owned},
    },
};
use mime::Mime;
use nom::{
    bytes::complete::{tag, tag_no_case},
    character::complete::{digit1, space0},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};
use time::{
    format_description::{
        well_known::{Rfc2822, Rfc3339},
        FormatItem,
    },
    OffsetDateTime, PrimitiveDateTime,
};

use crate::{io::import::Importer, prelude::*};

pub mod artwork;
pub mod digest;
pub mod tag;

#[cfg(feature = "gigtag")]
pub mod gigtag;

#[cfg(feature = "serato-markers")]
pub mod serato;

#[must_use]
pub fn trim_readable(input: &str) -> &str {
    input.trim_matches(|c: char| c.is_whitespace() || c.is_control())
}

pub fn guess_mime_from_file_ext(file_ext: &str) -> Result<Mime> {
    let mime_guess = mime_guess::from_ext(file_ext);
    if mime_guess.first().is_none() {
        return Err(Error::UnknownContentType(format!(
            "unsupported file extension \"{file_ext}\"",
        )));
    }
    mime_guess
        .iter()
        .filter(|mime| mime.type_() == mime::AUDIO)
        .chain(mime_guess.iter().filter(|mime| mime.type_() == mime::VIDEO))
        .next()
        .ok_or(Error::UnknownContentType(format!(
            "unsupported file extension \"{file_ext}\"",
        )))
}

pub fn guess_mime_from_file_path(path: impl AsRef<Path>) -> Result<Mime> {
    let mime_guess = mime_guess::from_path(path.as_ref());
    if mime_guess.first().is_none() {
        return Err(Error::UnknownContentType(format!(
            "file path \"{path}\"",
            path = path.as_ref().display()
        )));
    }
    mime_guess
        .iter()
        .filter(|mime| mime.type_() == mime::AUDIO)
        .chain(mime_guess.iter().filter(|mime| mime.type_() == mime::VIDEO))
        .next()
        .ok_or(Error::UnknownContentType(format!(
            "file path \"{path}\"",
            path = path.as_ref().display()
        )))
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
    // Precondition: Coherent chunk of actors with the given role at the back of the slice
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
        ActorKind::Sorting => unreachable!(),
    }
    proposed_kind
}

pub fn push_next_actor<'a>(
    actors: &mut Vec<Actor>,
    name: impl Into<Cow<'a, str>>,
    kind: ActorKind,
    role: ActorRole,
) -> bool {
    let Some(name) = trimmed_non_empty_from(name) else {
        return false;
    };
    let kind = match kind {
        ActorKind::Summary => adjust_summary_actor_kind(actors.as_mut_slice(), role, &name),
        ActorKind::Individual => ActorKind::Individual,
        ActorKind::Sorting => {
            if let Some(actor) = Actors::filter_kind_role(actors.as_slice(), kind, role).next() {
                // Only a single sorting actor is supported
                if name == actor.name {
                    // Silently ignore redundant/duplicate sorting actors
                    return true;
                }
                // Warn about ambiguous sorting actors
                log::warn!(
                    "Ignoring {role:?} actor \"{name}\" because \"{actor_name}\" is already used \
                     for sorting",
                    actor_name = actor.name
                );
                return false;
            }
            ActorKind::Sorting
        }
    };
    let actor = Actor {
        name: name.into(),
        kind,
        role,
        role_notes: None,
    };
    actors.push(actor);
    true
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
            importer.import_loudness_from_replay_gain(&formatted)
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

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum FormattedTempoBpm {
    Fractional(String),
    NonFractional(String),
}

impl From<FormattedTempoBpm> for String {
    fn from(formatted_tempo_bpm: FormattedTempoBpm) -> Self {
        match formatted_tempo_bpm {
            FormattedTempoBpm::Fractional(formatted)
            | FormattedTempoBpm::NonFractional(formatted) => formatted,
        }
    }
}

pub(crate) fn format_validated_tempo_bpm(
    tempo_bpm: &mut Option<TempoBpm>,
    format: TempoBpmFormat,
) -> Option<FormattedTempoBpm> {
    let validated_tempo_bpm = tempo_bpm
        .map(TempoBpm::validated_from)
        .transpose()
        .ok()
        .flatten();
    *tempo_bpm = validated_tempo_bpm.map(|validated| *validated);
    tempo_bpm
        .as_mut()
        .map(|tempo_bpm| format_tempo_bpm(tempo_bpm, format))
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TempoBpmFormat {
    Integer,
    Float,
}

pub(crate) fn format_tempo_bpm(
    tempo_bpm: &mut TempoBpm,
    format: TempoBpmFormat,
) -> FormattedTempoBpm {
    match format {
        TempoBpmFormat::Integer => {
            // Do not touch the original value when rounding to integer!
            let tempo_bpm = TempoBpm::new(tempo_bpm.to_inner().round());
            FormattedTempoBpm::NonFractional(format_parseable_value(&mut tempo_bpm.to_inner()))
        }
        TempoBpmFormat::Float => {
            let mut value = tempo_bpm.to_inner();
            let formatted = format_parseable_value(&mut value);
            debug_assert!({
                // Verify the formatted float value by re-parsing it.
                let mut importer = Importer::new();
                debug_assert_eq!(
                    Some(*tempo_bpm),
                    importer.import_tempo_bpm(&formatted).map(Into::into)
                );
                debug_assert!(importer.finish().into_messages().is_empty());
                true
            });
            if value.fract() == 0.0 {
                FormattedTempoBpm::NonFractional(formatted)
            } else {
                FormattedTempoBpm::Fractional(formatted)
            }
        }
    }
}

#[must_use]
pub(crate) fn parse_key_signature(input: &str) -> Option<KeySignature> {
    let input = trim_readable(input);
    if input.is_empty() {
        return None;
    }
    if input.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        if input.starts_with(|c: char| c.is_ascii_digit()) {
            // Camelot is the most common encoding and also recommended
            // as an alternative for the TKEY frame in ID3v2.
            let key_code = KeyCode::try_from_camelot_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
            let key_code = KeyCode::try_from_openkey_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
        } else {
            // Try the ID3v2 recommendation for TKEY first.
            let key_code = KeyCode::try_from_serato_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
            let key_code = KeyCode::try_from_canonical_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
            let key_code = KeyCode::try_from_traditional_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
            let key_code = KeyCode::try_from_traditional_ascii_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
            let key_code = KeyCode::try_from_beatport_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
            let key_code = KeyCode::try_from_traxsource_str(input);
            if let Some(key_code) = key_code {
                return Some(key_code.into());
            }
        }
    }
    None
}

#[must_use]
pub(crate) fn key_signature_as_str(key_signature: KeySignature) -> &'static str {
    // Follow the ID3v2 recommendation, independent of the actual format.
    // See also: https://mutagen-specs.readthedocs.io/en/latest/id3/id3v2.4.0-frames.html#tkey
    // TODO: Should this be configurable depending on the format?
    key_signature.code().as_serato_str()
}

const RFC3339_WITHOUT_TZ_FORMAT: &[FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");

const RFC3339_WITHOUT_T_TZ_FORMAT: &[FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub(crate) fn parse_year_tag(value: &str) -> Option<DateOrDateTime> {
    let input = value.trim();
    let mut digits_parser = delimited(space0, digit1, space0);
    let digits_parsed: IResult<_, _> = digits_parser(input);
    if let Ok((remainder, digits_input)) = digits_parsed {
        if remainder.is_empty()
            && (/* YYYY */digits_input.len() == 4 ||
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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
