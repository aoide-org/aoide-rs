// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use aoide_core::track::actor::Actors;

use super::*;

#[test]
fn parse_year_tag_valid() {
    // All test inputs surrounded by whitespaces!
    assert_eq!(
        Some(YyyyMmDdDate::new_unchecked(19_780_000).into()),
        parse_year_tag(" 1978 ")
    );
    assert_eq!(
        Some(YyyyMmDdDate::new_unchecked(20_041_200).into()),
        parse_year_tag(" 200412 ")
    );
    assert_eq!(
        Some(YyyyMmDdDate::new_unchecked(20_010_900).into()),
        parse_year_tag(" 2001 \t - 9 ")
    );
    assert_eq!(
        Some(YyyyMmDdDate::new_unchecked(19_990_702).into()),
        parse_year_tag(" 1999 - 7 - 2 ")
    );
    assert_eq!(
        Some(YyyyMmDdDate::new_unchecked(19_991_231).into()),
        parse_year_tag(" 1999 - 12 - \t 31 ")
    );
    assert_eq!(
        Some(YyyyMmDdDate::new_unchecked(20_200_229).into()),
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
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Integer)
    );
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Float)
    );
    assert_eq!(None, tempo_bpm);
}

#[test]
fn format_validated_tempo_bpm_invalid() {
    let mut tempo_bpm = Some(TempoBpm::new(TempoBpm::MIN.value() - 1.0));
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Integer)
    );
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Float)
    );
    assert_eq!(None, tempo_bpm);

    let mut tempo_bpm = Some(TempoBpm::new(0.0));
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Integer)
    );
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Float)
    );
    assert_eq!(None, tempo_bpm);

    let mut tempo_bpm = Some(TempoBpm::new(-0.0));
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Integer)
    );
    assert_eq!(
        None,
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Float)
    );
    assert_eq!(None, tempo_bpm);
}

#[test]
fn format_validated_tempo_bpm_min_max() {
    let mut tempo_bpm = Some(TempoBpm::new(TempoBpm::MIN.value()));
    assert_eq!(
        Some(FormattedTempoBpm::NonFractional("0".into())),
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Integer)
    );
    assert_eq!(
        Some(FormattedTempoBpm::Fractional(
            TempoBpm::MIN.value().to_string()
        )),
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Float)
    );
    assert_eq!(Some(TempoBpm::MIN), tempo_bpm);

    let mut tempo_bpm = Some(TempoBpm::new(TempoBpm::MAX.value()));
    assert_eq!(
        Some(FormattedTempoBpm::NonFractional(
            TempoBpm::MAX.value().to_string()
        )),
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Integer)
    );
    assert_eq!(
        Some(FormattedTempoBpm::NonFractional(
            TempoBpm::MAX.value().to_string()
        )),
        format_validated_tempo_bpm(&mut tempo_bpm, TempoBpmFormat::Float)
    );
    assert_eq!(Some(TempoBpm::MAX), tempo_bpm);
}

#[test]
fn format_fractional_bpm() {
    let mut tempo_bpm_round_up = Some(TempoBpm::new(100.5));
    assert_eq!(
        Some(FormattedTempoBpm::NonFractional("101".into())),
        format_validated_tempo_bpm(&mut tempo_bpm_round_up, TempoBpmFormat::Integer)
    );
    assert_eq!(
        Some(FormattedTempoBpm::Fractional("100.5".into())),
        format_validated_tempo_bpm(&mut tempo_bpm_round_up, TempoBpmFormat::Float)
    );
    let mut tempo_bpm_round_down = Some(TempoBpm::new(100.4));
    assert_eq!(
        Some(FormattedTempoBpm::NonFractional("100".into())),
        format_validated_tempo_bpm(&mut tempo_bpm_round_down, TempoBpmFormat::Integer)
    );
    assert_eq!(
        Some(FormattedTempoBpm::Fractional("100.4".into())),
        format_validated_tempo_bpm(&mut tempo_bpm_round_down, TempoBpmFormat::Float)
    );
}

#[test]
#[allow(clippy::too_many_lines)] // TODO
fn push_next_actor_role_names() {
    let mut actors = vec![];

    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Artist1 ft. Artist2"),
        Default::default(),
        ActorRole::Artist,
    ));
    assert_eq!(
        Some("Artist1 ft. Artist2"),
        Actors::summary_actor(actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Artist1"),
        Default::default(),
        ActorRole::Artist,
    ));
    assert_eq!(
        Some("Artist1 ft. Artist2"),
        Actors::summary_actor(actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Artist2"),
        Default::default(),
        ActorRole::Artist,
    ));
    assert_eq!(
        Some("Artist1 ft. Artist2"),
        Actors::summary_actor(actors.iter(), ActorRole::Artist).map(|actor| actor.name.as_str())
    );

    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Composer1"),
        Default::default(),
        ActorRole::Composer,
    ));
    assert_eq!(
        Some("Composer1"),
        Actors::summary_actor(actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Composer1, Composer2"),
        Default::default(),
        ActorRole::Composer,
    ));
    assert_eq!(
        Some("Composer1, Composer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Composer2"),
        Default::default(),
        ActorRole::Composer,
    ));
    assert_eq!(
        Some("Composer1, Composer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Composer).map(|actor| actor.name.as_str())
    );

    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Remixer2"),
        Default::default(),
        ActorRole::Remixer,
    ));
    assert_eq!(
        Some("Remixer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Remixer).map(|actor| actor.name.as_str())
    );
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Remixer1"),
        Default::default(),
        ActorRole::Remixer,
    ));
    assert!(Actors::summary_actor(actors.iter(), ActorRole::Remixer).is_none());
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Remixer1 & Remixer2"),
        Default::default(),
        ActorRole::Remixer,
    ));
    assert_eq!(
        Some("Remixer1 & Remixer2"),
        Actors::summary_actor(actors.iter(), ActorRole::Remixer).map(|actor| actor.name.as_str())
    );

    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Lyricist1"),
        Default::default(),
        ActorRole::Lyricist,
    ));
    assert!(push_next_actor(
        &mut actors,
        Cow::Borrowed("Lyricist2"),
        Default::default(),
        ActorRole::Lyricist,
    ));
    assert!(push_next_actor(
        &mut actors,
        // Duplicate name
        Cow::Borrowed("Lyricist1"),
        Default::default(),
        ActorRole::Lyricist,
    ));
    assert!(push_next_actor(
        &mut actors,
        // Duplicate name (again)
        Cow::Borrowed("Lyricist2"),
        Default::default(),
        ActorRole::Lyricist,
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
