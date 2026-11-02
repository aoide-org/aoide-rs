// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::{EntityUid, TrackUid};

use super::TrackSelection;

fn new_entity_uids(count: usize) -> Vec<TrackUid> {
    (0..count)
        .map(|_| TrackUid::from_untyped(EntityUid::random()))
        .collect()
}

#[test]
fn select_range_forward_and_backward() {
    let list = new_entity_uids(6);
    let mut selection = TrackSelection::new(Vec::new(), None);
    selection.toggle(list[1].clone());
    selection.select_range(&list, list[3].clone());
    assert_eq!(&list[1..=3], selection.entity_uids.as_slice());

    // Multiple ranges from the same anchor in the opposite direction.
    selection.select_range(&list, list[0].clone());
    assert_eq!(
        vec![&list[1], &list[2], &list[3], &list[0]],
        selection.entity_uids.iter().collect::<Vec<_>>()
    );
    assert_eq!(Some(&list[1]), selection.anchor_entity_uid.as_ref());
}

#[test]
fn select_range_preserves_selected_tracks() {
    let list = new_entity_uids(6);
    let mut selection = TrackSelection::new(Vec::new(), None);
    selection.toggle(list[5].clone());
    selection.toggle(list[2].clone());
    // Tracks are added in the direction from the anchor to the target.
    selection.select_range(&list, list[0].clone());
    assert_eq!(
        vec![&list[5], &list[2], &list[1], &list[0]],
        selection.entity_uids.iter().collect::<Vec<_>>()
    );
}

#[test]
fn select_range_without_listed_anchor_selects_target() {
    let list = new_entity_uids(3);
    let mut selection = TrackSelection::new(Vec::new(), None);
    selection.select_range(&list, list[1].clone());
    assert_eq!(&list[1..=1], selection.entity_uids.as_slice());
    assert_eq!(Some(&list[1]), selection.anchor_entity_uid.as_ref());

    // The anchor is no longer listed, e.g. after searching again.
    let other_list = new_entity_uids(3);
    selection.select_range(&other_list, other_list[2].clone());
    assert_eq!(
        vec![&list[1], &other_list[2]],
        selection.entity_uids.iter().collect::<Vec<_>>()
    );
    assert_eq!(Some(&other_list[2]), selection.anchor_entity_uid.as_ref());
}
