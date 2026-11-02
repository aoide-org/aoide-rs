// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use aoide_core::{
    Track,
    media::{
        self,
        content::{AudioContentMetadata, ContentLink},
    },
    track::EntityBody,
    util::clock::OffsetDateTimeMs,
};

use super::*;

fn new_entity(i: usize) -> Entity {
    let collected_at = OffsetDateTimeMs::now_local();
    let media_source = media::Source {
        collected_at,
        content: media::Content {
            link: ContentLink {
                path: format!("file{i}.mp3").into(),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata::default().into(),
            digest: None,
        },
        artwork: Default::default(),
    };
    let body = EntityBody {
        track: Track::new_from_media_source(media_source),
        updated_at: collected_at.to_utc(),
        last_synchronized_rev: None,
        content_url: Some(Url::parse(&format!("file:///music/file{i}.mp3")).unwrap()),
    };
    Entity::new(EntityHeader::initial_random(), body)
}

fn new_ready_state(entities: Vec<Entity>) -> State {
    let mut last_memo = FetchedEntitiesMemo::empty();
    let fetched_entities = entities
        .into_iter()
        .map(|entity| {
            last_memo = last_memo.next(&entity.hdr);
            FetchedEntity {
                offset_hash: last_memo.offset_hash,
                entity,
            }
        })
        .collect();
    State {
        fetch: FetchState::Ready {
            fetched_entities,
            can_fetch_more: true,
        },
        ..Default::default()
    }
}

fn next_rev(entity: &Entity) -> Entity {
    let mut entity = entity.clone();
    entity.raw.hdr = entity.raw.hdr.next_rev().unwrap();
    entity.raw.body.content_url = None;
    entity.raw.body.track.set_track_title("Modified");
    entity
}

#[test]
fn replace_fetched_entity_in_place_without_changing_memo() {
    let entities: Vec<_> = (0..3).map(new_entity).collect();
    let mut state = new_ready_state(entities.clone());
    let mut memo = state.clone_memo();

    let modified_entity = next_rev(&entities[1]);
    assert!(matches!(
        state.replace_fetched_entity(modified_entity.clone()),
        ActionEffect::Changed
    ));

    let fetched_entities = state.fetched_entities().unwrap();
    assert_eq!(3, fetched_entities.len());
    let replaced_entity = &fetched_entities[1].entity;
    assert_eq!(modified_entity.hdr, replaced_entity.hdr);
    assert_eq!(Some("Modified"), replaced_entity.body.track.track_title());
    // The unresolved content URL is preserved.
    assert_eq!(
        entities[1].body.content_url,
        replaced_entity.body.content_url
    );
    // Other entities are not affected.
    assert_eq!(entities[0].hdr, fetched_entities[0].entity.hdr);
    assert_eq!(entities[2].hdr, fetched_entities[2].entity.hdr);
    // The memo is unchanged, i.e. consumers don't need to replace all fetched entities.
    assert_eq!(MemoDiff::Unchanged, state.update_memo(&mut memo));
}

#[test]
fn replace_fetched_entity_should_ignore_outdated_or_unknown_entities() {
    let entities: Vec<_> = (0..2).map(new_entity).collect();
    let mut state = new_ready_state(entities.clone());

    // Same revision
    assert!(matches!(
        state.replace_fetched_entity(entities[0].clone()),
        ActionEffect::Unchanged
    ));
    // Not fetched
    assert!(matches!(
        state.replace_fetched_entity(next_rev(&new_entity(2))),
        ActionEffect::Unchanged
    ));
    // Nothing fetched
    assert!(matches!(
        State::default().replace_fetched_entity(next_rev(&entities[0])),
        ActionEffect::Unchanged
    ));

    let modified_entity = next_rev(&entities[0]);
    assert!(matches!(
        state.replace_fetched_entity(modified_entity.clone()),
        ActionEffect::Changed
    ));
    // Outdated revision
    assert!(matches!(
        state.replace_fetched_entity(entities[0].clone()),
        ActionEffect::Unchanged
    ));
    assert_eq!(
        modified_entity.hdr,
        state.fetched_entities().unwrap()[0].entity.hdr
    );
}
