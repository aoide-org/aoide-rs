// aoide.org - Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    audio::{BitrateBps, ChannelCount, Channels, DurationMs, LoudnessLufs, SampleRateHz},
    media::{
        content::{
            AudioContentMetadata, ContentLink, ContentMetadata, ContentPath, ContentRevision,
        },
        Content, Source as MediaSource,
    },
    track::{Entity, EntityBody, EntityHeader, Track},
    util::clock::DateTime,
};

use crate::{IndexStorage, TrackIndex};

#[test]
fn track_index_smoke_test_to_verify_dynamic_schema_against_static_types() {
    let track_index = TrackIndex::open_or_recreate(IndexStorage::InMemory).unwrap();
    let audio_metadata = AudioContentMetadata {
        bitrate: Some(BitrateBps::new(320_000.0)),
        duration: Some(DurationMs::new(240_000.0)),
        channels: Some(Channels::Count(ChannelCount(2))),
        encoder: Some("encoder".to_owned()),
        loudness: Some(LoudnessLufs(1.234)),
        sample_rate: Some(SampleRateHz::new(44_100.0)),
    };
    let media_source = MediaSource {
        collected_at: DateTime::now_utc(),
        artwork: None,
        content: Content {
            link: ContentLink {
                path: ContentPath::new("content/path/file.mp3".into()),
                rev: Some(ContentRevision::new(1)),
            },
            r#type: "audio/mpeg".parse().unwrap(),
            digest: Some(b"jsdf09w8092r2oijwlfksdf".to_vec()),
            metadata: ContentMetadata::Audio(audio_metadata),
            metadata_flags: Default::default(),
        },
    };
    let track = Track {
        media_source,
        // TODO: Populate all relevant fields that are stored in Tantivy
        actors: Default::default(),
        advisory_rating: None,
        album: Default::default(),
        color: None,
        copyright: None,
        cues: Default::default(),
        indexes: Default::default(),
        metrics: Default::default(),
        publisher: None,
        recorded_at: None,
        released_at: None,
        released_orig_at: None,
        tags: Default::default(),
        titles: Default::default(),
    };
    let entity_body = EntityBody {
        updated_at: DateTime::now_utc(),
        track,
        content_url: Some("https://www.example.com/file.mp3".parse().unwrap()),
        last_synchronized_rev: None,
    };
    let entity = Entity::new(EntityHeader::initial_random(), entity_body);
    let document = track_index.fields.create_document(&entity, None);
    let writer = track_index.index.writer(3_000_000).unwrap();
    let _doc_id = writer.add_document(document).unwrap();
}
