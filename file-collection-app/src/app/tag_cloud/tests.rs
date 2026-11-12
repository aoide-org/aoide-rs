// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::EntityUid;

use super::*;

fn new_tag_cloud() -> TagCloud {
    TagCloud::new(TrackUid::from_untyped(EntityUid::random()))
}

fn labels(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|&label| label.to_owned()).collect()
}

#[test]
fn outdated_results_are_discarded() {
    let mut tag_cloud = new_tag_cloud();
    let first_request_id = tag_cloud.start_loading();
    let second_request_id = tag_cloud.start_loading();
    assert!(!tag_cloud.finish_loading(first_request_id, Ok(vec![("a".to_owned(), 1)])));
    assert!(tag_cloud.is_loading());
    assert!(tag_cloud.finish_loading(second_request_id, Ok(vec![("b".to_owned(), 2)])));
    assert!(!tag_cloud.is_loading());
    assert!(tag_cloud.contains("b"));
    assert!(!tag_cloud.contains("a"));
    // Only once.
    assert!(!tag_cloud.finish_loading(second_request_id, Ok(Vec::new())));
}

#[test]
fn tags_are_preserved_if_loading_fails() {
    let mut tag_cloud = new_tag_cloud();
    let request_id = tag_cloud.start_loading();
    assert!(tag_cloud.finish_loading(request_id, Ok(vec![("a".to_owned(), 1)])));
    let request_id = tag_cloud.start_loading();
    assert!(tag_cloud.finish_loading(request_id, Err("failed".to_owned())));
    assert!(tag_cloud.contains("a"));
    assert_eq!(Some("failed"), tag_cloud.load_error.as_deref());
    // Recovered
    let request_id = tag_cloud.start_loading();
    assert!(tag_cloud.finish_loading(request_id, Ok(Vec::new())));
    assert_eq!(None, tag_cloud.load_error);
}

#[test]
fn aborted_loading_ignores_results() {
    let mut tag_cloud = new_tag_cloud();
    let request_id = tag_cloud.start_loading();
    tag_cloud.abort_loading();
    assert!(!tag_cloud.is_loading());
    assert!(!tag_cloud.finish_loading(request_id, Ok(Vec::new())));
}

#[test]
fn font_scale_grows_with_the_frequency() {
    assert_eq!(MIN_FONT_SCALE, font_scale(0, 0));
    assert_eq!(MIN_FONT_SCALE, font_scale(1, 1));
    assert_eq!(MIN_FONT_SCALE, font_scale(1, 100));
    assert_eq!(MAX_FONT_SCALE, font_scale(100, 100));
    let medium = font_scale(10, 100);
    assert!(MIN_FONT_SCALE < medium && medium < MAX_FONT_SCALE);
    assert!(font_scale(9, 100) < medium);
}

#[test]
fn dragging_a_selected_tag_drags_all_selected_tags() {
    let selected = ["b", "a"].map(ToOwned::to_owned).into_iter().collect();
    assert_eq!(labels(&["a", "b"]), dragged_labels(&selected, "b"));
    assert_eq!(labels(&["c"]), dragged_labels(&selected, "c"));
}

#[test]
fn tags_delta_only_contains_modifications() {
    let tags = labels(&["a", "b"]);
    assert_eq!(
        Some(CustomTagsDelta::plain(["c"], ["b"])),
        tags_delta(Some(&tags), &labels(&["a", "c"]), &labels(&["b", "d"]))
    );
    assert_eq!(
        None,
        tags_delta(Some(&tags), &labels(&["a"]), &labels(&["c"]))
    );
    // Unknown tags
    assert_eq!(
        Some(CustomTagsDelta::plain(["a"], ["c"])),
        tags_delta(None, &labels(&["a"]), &labels(&["c"]))
    );
}

#[test]
fn selected_track_tags_are_reset_when_switching_tracks() {
    let mut tag_cloud = new_tag_cloud();
    let request_id = tag_cloud.start_loading();
    assert!(tag_cloud.finish_loading(request_id, Ok(vec![("a".to_owned(), 1)])));
    let mut selection = TagSelection::default();
    let edited_tags = labels(&["a", "b"]);
    selection.retain_available(&tag_cloud, &edited_tags);
    toggle(&mut selection.track, "a");
    toggle(&mut selection.track, "b");
    toggle(&mut selection.cloud, "a");
    toggle(&mut selection.cloud, "x");

    // Unavailable tags are discarded.
    selection.retain_available(&tag_cloud, &labels(&["b"]));
    assert_eq!(
        labels(&["b"]),
        selection.track.iter().cloned().collect::<Vec<_>>()
    );
    assert_eq!(
        labels(&["a"]),
        selection.cloud.iter().cloned().collect::<Vec<_>>()
    );

    tag_cloud.entity_uid = TrackUid::from_untyped(EntityUid::random());
    selection.retain_available(&tag_cloud, &edited_tags);
    assert!(selection.track.is_empty());
    assert_eq!(
        labels(&["a"]),
        selection.cloud.iter().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn edits_of_a_selected_track_apply_to_all_selected_tracks() {
    let new_entity_uid = || TrackUid::from_untyped(EntityUid::random());
    let unselected = new_entity_uid();
    let track_selection = TrackSelection::new(vec![new_entity_uid(), new_entity_uid()], None);
    let tag_cloud = TagCloud::new(unselected.clone());
    let tag_editing = TagEditing {
        tag_cloud: &tag_cloud,
        track_list: &[],
        track_selection: Some(&track_selection),
    };
    assert_eq!(
        track_selection.entity_uids.as_slice(),
        tag_editing.target_entity_uids(&track_selection.entity_uids[1])
    );
    assert_eq!(
        std::slice::from_ref(&unselected),
        tag_editing.target_entity_uids(&unselected)
    );
}
