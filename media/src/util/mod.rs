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

use std::unreachable;

use super::TagMappingConfig;

use aoide_core::{
    audio::signal::LoudnessLufs,
    music::{
        key::{KeyCodeValue, KeyMode, KeySignature, LancelotKeySignature, OpenKeySignature},
        time::TempoBpm,
    },
    tag::{
        Facet as TagFacet, Label as TagLabel, LabelValue, PlainTag, Score as TagScore, ScoreValue,
        TagsMap,
    },
    track::{
        actor::{Actor, ActorKind, ActorRole},
        release::DateOrDateTime,
    },
    util::clock::{DateTime, DateTimeInner, DateYYYYMMDD, YYYYMMDD},
};

use chrono::{NaiveDateTime, Utc};
use nom::{
    bytes::complete::{tag, tag_no_case},
    character::complete::{digit1, one_of, space0},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};
use semval::IsValid;

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

pub fn try_import_plain_tag(
    label_value: impl Into<LabelValue>,
    score_value: impl Into<ScoreValue>,
) -> Result<PlainTag, PlainTag> {
    let label = TagLabel::clamp_from(label_value);
    let score = TagScore::clamp_from(score_value);
    let plain_tag = PlainTag {
        label: Some(label),
        score,
    };
    if plain_tag.is_valid() {
        Ok(plain_tag)
    } else {
        Err(plain_tag)
    }
}

pub fn import_faceted_tags(
    tags_map: &mut TagsMap,
    next_score_value: &mut ScoreValue,
    facet: &TagFacet,
    tag_mapping_config: Option<&TagMappingConfig>,
    label_value: impl Into<LabelValue>,
) -> usize {
    let mut import_count = 0;
    let label_value = label_value.into();
    if let Some(tag_mapping_config) = tag_mapping_config {
        if !tag_mapping_config.label_separator.is_empty() {
            for (_, split_label_value) in
                label_value.match_indices(&tag_mapping_config.label_separator)
            {
                match try_import_plain_tag(split_label_value, *next_score_value) {
                    Ok(plain_tag) => {
                        tags_map.insert(facet.to_owned().into(), plain_tag);
                        import_count += 1;
                        *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                    }
                    Err(plain_tag) => {
                        log::warn!("Failed to import faceted '{}' tag: {:?}", facet, plain_tag,);
                    }
                }
            }
        }
    }
    if import_count == 0 {
        match try_import_plain_tag(label_value, *next_score_value) {
            Ok(plain_tag) => {
                tags_map.insert(facet.to_owned().into(), plain_tag);
                import_count += 1;
                if let Some(tag_mapping_config) = tag_mapping_config {
                    *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                }
            }
            Err(plain_tag) => {
                log::warn!("Failed to import faceted '{}' tag: {:?}", facet, plain_tag,);
            }
        }
    }
    import_count
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
                log::warn!("Invalid tempo parsed from input '{}': {}", input, tempo_bpm);
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
    let mut parser = separated_pair(
        preceded(space0, digit1),
        space0,
        terminated(one_of("dmAB"), space0),
    );
    let res: IResult<_, (_, _)> = parser(input);
    if let Ok((remainder, (code_input, mode_input))) = res {
        if !remainder.is_empty() {
            log::warn!(
                "Unexpected remainder '{}' after parsing key signature '{}'",
                remainder,
                input
            );
        }
        if let Ok(key_code) = code_input.parse::<KeyCodeValue>() {
            match mode_input {
                mode_input @ 'd' | mode_input @ 'm' => {
                    // Open Key
                    if key_code >= OpenKeySignature::min_code()
                        && key_code <= OpenKeySignature::max_code()
                    {
                        let key_mode = if mode_input == 'd' {
                            KeyMode::Major
                        } else {
                            KeyMode::Minor
                        };
                        let key_signature = OpenKeySignature::new(key_code, key_mode);
                        log::debug!(
                            "Parsed key signature from input '{}': {}",
                            input,
                            key_signature
                        );
                        return Some(key_signature.into());
                    }
                }
                mode_input @ 'A' | mode_input @ 'B' => {
                    // Lancelot
                    if key_code >= LancelotKeySignature::min_code()
                        && key_code <= LancelotKeySignature::max_code()
                    {
                        let key_mode = if mode_input == 'A' {
                            KeyMode::Minor
                        } else {
                            KeyMode::Major
                        };
                        let key_signature = LancelotKeySignature::new(key_code, key_mode);
                        log::debug!(
                            "Parsed key signature from input '{}': {}",
                            input,
                            key_signature
                        );
                        return Some(key_signature.into());
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    if let Ok(key_code) = input.parse::<KeyCodeValue>() {
        // Fallback: Raw key code
        let key_signature = KeySignature::new(key_code.into());
        log::debug!(
            "Parsed key signature from input '{}': {}",
            input,
            key_signature
        );
        return Some(key_signature);
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
                            if day_of_month >= 0 && day_of_month <= 31 {
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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
