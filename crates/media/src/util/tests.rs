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

///////////////////////////////////////////////////////////////////////

use aoide_core::{track::actor::Actors, util::canonical::CanonicalizeInto};

use super::*;

#[test]
fn parse_year_tag_valid() {
    // All test inputs surrounded by whitespaces!
    assert_eq!(
        Some(DateYYYYMMDD::new(19780000).into()),
        parse_year_tag(" 1978 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(20041200).into()),
        parse_year_tag(" 200412 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(20010900).into()),
        parse_year_tag(" 2001 \t - 9 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(19990702).into()),
        parse_year_tag(" 1999 - 7 - 2 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(19991231).into()),
        parse_year_tag(" 1999 - 12 - \t 31 ")
    );
    assert_eq!(
        Some(DateYYYYMMDD::new(20200229).into()),
        parse_year_tag(" \t20200229 ")
    );
    assert_eq!(
        "2009-09-18T07:00:00Z",
        parse_year_tag(" 2009-09-18T07:00:00Z ")
            .unwrap()
            .to_string()
    );
    // No time zone offset
    assert_eq!(
        "2009-09-18T07:00:00Z",
        parse_year_tag(" 2009-09-18T07:00:00\n")
            .unwrap()
            .to_string()
    );
    // Date/time separated by arbitrary whitespace without time zone offset
    assert_eq!(
        "2009-09-18T07:12:34Z",
        parse_year_tag("\t 2009-09-18 \t\r\n 07:12:34 \n")
            .unwrap()
            .to_string()
    );
}

#[test]
fn trim_readable_should_ignore_whitespace_and_control_characters() {
    assert!(trim_readable(" \t \n ").is_empty());
    let input = String::from_utf8(vec![0x11, 0x00, 0x0A, 0x0D, 0x20, 0x09]).unwrap();
    assert!(trim_readable(&input).is_empty());
}

#[test]
fn format_validated_tempo_bpm_none() {
    let mut tempo_bpm = None;
    assert_eq!(None, format_validated_tempo_bpm(&mut tempo_bpm));
    assert_eq!(None, tempo_bpm);
}

#[test]
fn format_validated_tempo_bpm_invalid() {
    let mut tempo_bpm = Some(TempoBpm::from_raw(TempoBpm::min().to_raw() - 1.0));
    assert_eq!(None, format_validated_tempo_bpm(&mut tempo_bpm));
    assert_eq!(None, tempo_bpm);

    let mut tempo_bpm = Some(TempoBpm::from_raw(0.0));
    assert_eq!(None, format_validated_tempo_bpm(&mut tempo_bpm));
    assert_eq!(None, tempo_bpm);

    let mut tempo_bpm = Some(TempoBpm::from_raw(-0.0));
    assert_eq!(None, format_validated_tempo_bpm(&mut tempo_bpm));
    assert_eq!(None, tempo_bpm);
}

#[test]
fn format_validated_tempo_bpm_min_max() {
    let mut tempo_bpm = Some(TempoBpm::from_raw(TempoBpm::min().to_raw()));
    assert_eq!(
        Some(TempoBpm::min().to_raw().to_string()),
        format_validated_tempo_bpm(&mut tempo_bpm)
    );
    assert_eq!(Some(TempoBpm::min()), tempo_bpm);

    let mut tempo_bpm = Some(TempoBpm::from_raw(TempoBpm::max().to_raw()));
    assert_eq!(
        Some(TempoBpm::max().to_raw().to_string()),
        format_validated_tempo_bpm(&mut tempo_bpm)
    );
    assert_eq!(Some(TempoBpm::max()), tempo_bpm);
}

#[test]
fn push_next_actor_role_names() {
    let mut actors = vec![];

    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Artist,
        "Artist1 ft. Artist2".to_owned()
    ));
    assert_eq!(
        Some("Artist1 ft. Artist2"),
        Actors::summary_actor(actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Artist,
        "Artist1".to_owned()
    ));
    assert_eq!(
        Some("Artist1 ft. Artist2"),
        Actors::summary_actor(actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Artist,
        "Artist2".to_owned()
    ));
    assert_eq!(
        Some("Artist1 ft. Artist2"),
        Actors::summary_actor(actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    );

    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Composer,
        "Composer1".to_owned()
    ));
    assert_eq!(
        Some("Composer1"),
        Actors::summary_actor(actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Composer,
        "Composer1, Composer2".to_owned()
    ));
    assert_eq!(
        Some("Composer1, Composer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Composer,
        "Composer2".to_owned()
    ));
    assert_eq!(
        Some("Composer1, Composer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    );

    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Remixer,
        "Remixer2".to_owned()
    ));
    assert_eq!(
        Some("Remixer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Remixer).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Remixer,
        "Remixer1".to_owned()
    ));
    assert!(Actors::summary_actor(actors.iter(), ActorRole::Remixer).is_none());
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Remixer,
        "Remixer1 & Remixer2".to_owned()
    ));
    assert_eq!(
        Some("Remixer1 & Remixer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Remixer).map(|actor| actor.name.as_str())
    );

    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Lyricist,
        "Lyricist1".to_owned()
    ));
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Lyricist,
        "Lyricist2".to_owned()
    ));
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Lyricist,
        // Duplicate name
        "Lyricist1".to_owned()
    ));
    assert!(push_next_actor_role_name(
        &mut actors,
        ActorRole::Lyricist,
        // Duplicate name (again)
        "Lyricist2".to_owned()
    ));
    let actors = actors.canonicalize_into();
    assert_eq!(
        0,
        Actors::filter_kind_role(actors.iter(), ActorKind::Summary, ActorRole::Lyricist).count()
    );
    assert_eq!(
        2,
        Actors::filter_kind_role(actors.iter(), ActorKind::Individual, ActorRole::Lyricist).count()
    );
}
