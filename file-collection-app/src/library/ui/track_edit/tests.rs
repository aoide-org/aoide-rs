// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::{
    TagFacetKey, TagScore, TagsMap,
    media::{
        self,
        content::{AudioContentMetadata, ContentLink},
    },
    util::clock::OffsetDateTimeMs,
};

use super::*;

fn plain_tag(label: &str) -> PlainTag<'static> {
    PlainTag {
        label: TagLabel::clamp_from(label.to_owned()),
        ..Default::default()
    }
}

fn new_track() -> Track {
    let media_source = media::Source {
        collected_at: OffsetDateTimeMs::now_local(),
        content: media::Content {
            link: ContentLink {
                path: "track.mp3".into(),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata::default().into(),
            digest: None,
        },
        artwork: Default::default(),
    };
    let mut track = Track::new_from_media_source(media_source);
    track.set_track_title("Title");
    track.set_track_artist("Artist".to_owned());
    track.set_album_title("Album");
    track.metrics.tempo_bpm = Some(TempoBpm::new(127.5));
    let mut tags_map = TagsMap::default();
    tags_map.insert_one(
        TagFacetKey::from(FACET_ID_GENRE.clone()),
        PlainTag {
            label: TagLabel::clamp_from("House".to_owned()),
            score: TagScore::new_unchecked(0.75),
        },
    );
    tags_map.insert_one(
        TagFacetKey::from(FACET_ID_COMMENT.clone()),
        plain_tag("First comment"),
    );
    tags_map.insert_one(
        TagFacetKey::from(FACET_ID_COMMENT.clone()),
        plain_tag("Second comment"),
    );
    tags_map.insert_one(
        TagFacetKey::from(FACET_ID_ISRC.clone()),
        plain_tag("USRC17607839"),
    );
    tags_map.insert_one(TagFacetKey::unfaceted(), plain_tag("warmup"));
    tags_map.insert_one(
        TagFacetKey::from(TagFacetId::clamp_from("venue").unwrap()),
        plain_tag("club-x"),
    );
    track.tags = tags_map.canonicalize_into();
    track
}

#[test]
fn unmodified_draft_should_not_modify_track() {
    let track = new_track();
    let draft = TrackEditDraft::from_track(&track);
    assert!(!draft.is_modified(&track));
    assert_eq!(Ok(track.clone()), draft.to_track(&track).map_err(|_| ()));
    assert_eq!("#warmup venue#club-x", draft.custom_tags);
    assert_eq!("First comment\nSecond comment", draft.comment);
    assert_eq!("127.5", draft.bpm);
}

#[test]
fn edit_properties() {
    let track = new_track();
    let mut draft = TrackEditDraft::from_track(&track);
    draft.title = " New title ".to_owned();
    draft.artist = String::new();
    draft.album_artist = "Album artist".to_owned();
    draft.album_subtitle = "Subtitle".to_owned();
    draft.recorded_at = "2024-05".to_owned();
    draft.bpm = "128".to_owned();
    draft.key = "8a".to_owned();
    draft.publisher = "Label".to_owned();
    draft.color = "#ff0000".to_owned();
    draft.advisory_rating = Some(AdvisoryRating::Explicit);
    assert!(draft.is_modified(&track));

    let edited = draft.to_track(&track).unwrap();
    assert_eq!(Some("New title"), edited.track_title());
    assert_eq!(None, edited.track_artist());
    assert_eq!(Some("Album"), edited.album_title());
    assert_eq!(Some("Album artist"), edited.album_artist());
    assert_eq!(Some("Subtitle"), edited.album_subtitle());
    assert_eq!(
        Some(DateOrDateTime::from(YyyyMmDdDate::from_year_month(2024, 5))),
        edited.recorded_at
    );
    assert_eq!(Some(TempoBpm::new(128.0)), edited.metrics.tempo_bpm);
    assert!(
        edited
            .metrics
            .flags
            .contains(MetricsFlags::TEMPO_BPM_INTEGER)
    );
    assert_eq!(
        Some(KeySignature::new(
            KeyCode::try_from_lancelot_str("8A").unwrap()
        )),
        edited.metrics.key_signature
    );
    assert_eq!(Some("Label"), edited.publisher.as_deref());
    assert_eq!(Some(Color::Rgb(RgbColor::RED)), edited.color);
    assert_eq!(Some(AdvisoryRating::Explicit), edited.advisory_rating);
    // Tags are not affected.
    assert_eq!(track.tags, edited.tags);

    // The draft of the edited track matches the edited draft.
    let mut expected_draft = draft.clone();
    expected_draft.title = "New title".to_owned();
    expected_draft.recorded_at = "2024-05".to_owned();
    expected_draft.key = "8A".to_owned();
    expected_draft.color = "#FF0000".to_owned();
    assert_eq!(expected_draft, TrackEditDraft::from_track(&edited));
}

#[test]
fn edit_tags() {
    let track = new_track();
    let mut draft = TrackEditDraft::from_track(&track);
    draft.genres = "House\n\nTechno ".to_owned();
    draft.custom_tags = "#peaktime\n venue#club-y ?s=0.5#rated".to_owned();

    let edited = draft.to_track(&track).unwrap();
    let edited_draft = TrackEditDraft::from_track(&edited);
    assert_eq!("House\nTechno", edited_draft.genres);
    assert_eq!(
        "#peaktime ?s=0.5#rated venue#club-y",
        edited_draft.custom_tags
    );
    // The score of the existing genre is preserved.
    assert_eq!(
        Some(TagScore::new_unchecked(0.75)),
        filter_faceted_tags(&edited.tags, FACET_ID_GENRE)
            .find(|tag| tag.label.as_ref().map(TagLabel::as_str) == Some("House"))
            .map(|tag| tag.score)
    );
    // Unmodified tags are preserved, even if they have multiple labels.
    assert_eq!(draft.comment, edited_draft.comment);
    assert_eq!(1, filter_faceted_tags(&edited.tags, FACET_ID_ISRC).count());
}

#[test]
fn clear_custom_tags() {
    let track = new_track();
    let mut draft = TrackEditDraft::from_track(&track);
    draft.custom_tags = " ".to_owned();
    let edited = draft.to_track(&track).unwrap();
    assert_eq!(0, edited.tags.plain.len());
    assert!(
        edited
            .tags
            .facets
            .iter()
            .all(|faceted_tags| !is_custom_tag_facet(&faceted_tags.facet_id))
    );
    assert_eq!(track.tags.total_count() - 2, edited.tags.total_count());
}

#[test]
fn invalid_fields() {
    let track = new_track();
    let mut draft = TrackEditDraft::from_track(&track);
    draft.recorded_at = "2024-13".to_owned();
    draft.bpm = "fast".to_owned();
    draft.key = "13A".to_owned();
    draft.color = "red".to_owned();
    draft.custom_tags = "warmup no tag here#".to_owned();
    let errors = draft.to_track(&track).unwrap_err();
    assert_eq!(
        vec![
            Field::RecordedAt,
            Field::Bpm,
            Field::Key,
            Field::Color,
            Field::CustomTags
        ],
        errors.iter().map(|err| err.field).collect::<Vec<_>>()
    );
}

#[test]
fn custom_tags_with_dedicated_facets_are_invalid() {
    let track = new_track();
    let mut draft = TrackEditDraft::from_track(&track);
    draft.custom_tags = "#warmup gnre#House".to_owned();
    let errors = draft.to_track(&track).unwrap_err();
    assert_eq!(1, errors.len());
    assert_eq!(Field::CustomTags, errors[0].field);
    assert!(errors[0].message.contains("gnre"));
}

#[test]
fn parse_dates() {
    for valid in ["2024", "2024-05", "2024-05-31", "2024-05-31T12:34:56Z"] {
        assert!(parse_date_or_date_time(valid).unwrap().is_some(), "{valid}");
    }
    for invalid in ["24", "2024-00", "2024-02-30", "2024-05-31-1", "May 2024"] {
        assert!(parse_date_or_date_time(invalid).is_err(), "{invalid}");
    }
    assert_eq!(Ok(None), parse_date_or_date_time(" "));
}

#[test]
fn custom_tags_delta() {
    let track = new_track();
    let delta = CustomTagsDelta::parse("#peaktime ?s=0.5#warmup", "venue#club-x #unknown").unwrap();
    assert!(!delta.is_empty());
    let mut edited = track.clone();
    delta.apply(&mut edited);
    let draft = TrackEditDraft::from_track(&edited);
    // The score of the existing tag has been replaced.
    assert_eq!("#peaktime ?s=0.5#warmup", draft.custom_tags);
    // Other tags are not affected.
    assert_eq!(TrackEditDraft::from_track(&track).comment, draft.comment);
    assert_eq!(1, filter_faceted_tags(&edited.tags, FACET_ID_ISRC).count());

    // Applying the delta again has no effect.
    let mut edited_again = edited.clone();
    delta.apply(&mut edited_again);
    assert_eq!(edited, edited_again);
}

#[test]
fn custom_tags_delta_should_reject_invalid_tags() {
    let errors = CustomTagsDelta::parse("warmup", "gnre#House").unwrap_err();
    assert_eq!(
        vec![Field::AddCustomTags, Field::RemoveCustomTags],
        errors.iter().map(|err| err.field).collect::<Vec<_>>()
    );
    assert!(CustomTagsDelta::parse(" ", "").unwrap().is_empty());
}
