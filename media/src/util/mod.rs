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

use super::TagMappingConfig;

use aoide_core::{
    audio::signal::LoudnessLufs,
    tag::{
        Facet as TagFacet, Label as TagLabel, LabelValue, PlainTag, Score as TagScore, ScoreValue,
        TagsMap,
    },
    track::actor::{Actor, ActorKind, ActorRole},
};

use nom::{
    bytes::complete::{tag, tag_no_case},
    character::complete::multispace0,
    multi::many_m_n,
    number::complete::double,
    sequence::tuple,
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

fn parse_replay_gain_ratio_db(input: &str) -> IResult<&str, f64> {
    let (input, (_, ratio, _, _, _)) = tuple((
        multispace0,
        double,
        multispace0,
        tag_no_case("dB"),
        multispace0,
    ))(input)?;
    Ok((input, ratio))
}

pub fn parse_replay_gain(input: &str) -> Option<LoudnessLufs> {
    match parse_replay_gain_ratio_db(input) {
        Ok((input, relative_gain_db)) => {
            if !input.is_empty() {
                log::info!("Unparsed replay gain input: {}", input);
            }
            Some(db2lufs(relative_gain_db))
        }
        Err(err) => {
            log::warn!("Failed to parse replay gain from '{}': {}", input, err);
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
