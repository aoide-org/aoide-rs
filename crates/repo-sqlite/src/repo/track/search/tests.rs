// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use nonicle::CanonicalizeInto as _;

use aoide_core::{
    Collection, Track, TrackBody, TrackEntity, TrackHeader,
    audio::DurationMs,
    collection::{Entity as CollectionEntity, EntityHeader as CollectionHeader},
    media::{
        self,
        content::{AudioContentMetadata, ContentLink},
    },
    tag::{FacetKey, Label, PlainTag, TagsMap},
    track::tag::FACET_ID_COMMENT,
    util::clock::OffsetDateTimeMs,
};
use aoide_core_api::{
    filtering::StringPredicate,
    tag::search::{FacetsFilter, Filter as TagFilter},
    track::search::{Filter as TrackFilter, Scope, TitlePhraseFilter},
};
use aoide_repo::{
    CollectionId, RecordCollector, ReservableRecordCollector,
    collection::EntityRepo as _,
    media::source::CollectionRepo as _,
    track::{CollectionRepo, EntityRepo as _},
};

use crate::{
    repo::tests::vfs_media_source_config,
    tests::{TestResult, establish_connection},
};

struct DummyCollector<H, R> {
    _header: std::marker::PhantomData<H>,
    _record: std::marker::PhantomData<R>,
}

impl<H, R> DummyCollector<H, R> {
    const fn new() -> Self {
        Self {
            _header: std::marker::PhantomData,
            _record: std::marker::PhantomData,
        }
    }
}

impl<H, R> RecordCollector for DummyCollector<H, R> {
    type Header = H;
    type Record = R;

    fn collect(&mut self, _header: Self::Header, _record: Self::Record) {}
}

impl<H, R> ReservableRecordCollector for DummyCollector<H, R> {
    fn reserve(&mut self, _additional: usize) {}
}

fn create_single_track_collection_with_tags(
    db: &mut crate::Connection<'_>,
) -> TestResult<CollectionId> {
    let collection = Collection {
        title: "Collection".into(),
        notes: None,
        kind: None,
        color: None,
        media_source_config: vfs_media_source_config(),
    };
    let collection_entity = CollectionEntity::new(CollectionHeader::initial_random(), collection);
    let collection_id =
        db.insert_collection_entity(&OffsetDateTimeMs::now_utc(), &collection_entity)?;
    let created_at = OffsetDateTimeMs::now_local();
    let media_source = media::Source {
        collected_at: created_at.clone(),
        content: media::Content {
            link: ContentLink {
                path: "/home/test/file.mp3".into(),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(1.0)),
                ..Default::default()
            }
            .into(),
            digest: None,
        },
        artwork: Default::default(),
    };
    let media_source_id = db
        .insert_media_source(collection_id, created_at.clone(), &media_source)?
        .id;
    let mut track = Track::new_from_media_source(media_source);
    // Track title with Unicode characters.
    track.set_track_title("ÄäÖöÜüÉéÈè");
    // Album title with ASCII characters.
    track.set_album_title("AaOoUuEeEe");
    let plain_tags = (1..10)
        .flat_map(|i| {
            [
                PlainTag {
                    label: Some(Label::from_unchecked(format!("Tag\\{i}"))),
                    score: Default::default(),
                },
                PlainTag {
                    label: Some(Label::from_unchecked(format!("tag\\{i}"))),
                    score: Default::default(),
                },
            ]
        })
        .collect::<Vec<_>>();
    track.tags = [(FacetKey::unfaceted(), plain_tags)]
        .into_iter()
        .collect::<TagsMap<'_>>()
        .canonicalize_into();
    let entity_body = TrackBody {
        track,
        updated_at: created_at,
        last_synchronized_rev: None,
        content_url: None,
    };
    let track_entity = TrackEntity::new(TrackHeader::initial_random(), entity_body);
    db.insert_track_entity(media_source_id, &track_entity)?;
    Ok(collection_id)
}

#[test]
fn filter_plain_tags() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let collection_id = create_single_track_collection_with_tags(&mut db)?;
    let filter = TrackFilter::Tag(TagFilter {
        modifier: None,
        facets: Some(FacetsFilter::AnyOf(vec![FacetKey::unfaceted()])),
        label: Some(StringPredicate::StartsWith("Tag\\".into())),
        score: None,
    });
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&filter),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    let filter = TrackFilter::Tag(TagFilter {
        modifier: None,
        facets: Some(FacetsFilter::NoneOf(vec![FACET_ID_COMMENT.into()])),
        label: Some(StringPredicate::StartsWith("tag\\".into())),
        score: None,
    });
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&filter),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    let filter = TrackFilter::Tag(TagFilter {
        modifier: None,
        facets: None,
        label: Some(StringPredicate::StartsNotWith("tag\\".into())),
        score: None,
    });
    assert_eq!(
        0,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&filter),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    for i in 1..10 {
        let filter = TrackFilter::Tag(TagFilter {
            modifier: None,
            facets: None,
            label: Some(StringPredicate::EndsWith(format!("\\{i}").into())),
            score: None,
        });
        assert_eq!(
            1,
            db.search_tracks(
                collection_id,
                &Default::default(),
                Some(&filter),
                Default::default(),
                &mut DummyCollector::new(),
            )?
        );
        let filter = TrackFilter::Tag(TagFilter {
            modifier: None,
            facets: None,
            label: Some(StringPredicate::EndsNotWith(format!("\\{i}").into())),
            score: None,
        });
        assert_eq!(
            1,
            db.search_tracks(
                collection_id,
                &Default::default(),
                Some(&filter),
                Default::default(),
                &mut DummyCollector::new(),
            )?
        );
    }
    Ok(())
}

#[test]
fn search_title_phrase() -> TestResult<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let collection_id = create_single_track_collection_with_tags(&mut db)?;
    // Case-insensitive ASCII search.
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&TrackFilter::TitlePhrase(TitlePhraseFilter {
                modifier: None,
                scope: Some(Scope::Album),
                kinds: Default::default(),
                name_terms: vec!["aA".into(), "EEee".into()],
            })),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    // Case-sensitive Unicode search.
    assert_eq!(
        1,
        db.search_tracks(
            collection_id,
            &Default::default(),
            Some(&TrackFilter::TitlePhrase(TitlePhraseFilter {
                modifier: None,
                scope: Some(Scope::Track),
                kinds: Default::default(),
                name_terms: vec!["Ää".into(), "ÉéÈè".into()],
            })),
            Default::default(),
            &mut DummyCollector::new(),
        )?
    );
    // Case-insensitive/folding Unicode search.
    // FIXME: Add Unicode-aware LIKE support to SQLite to fix this test.
    // assert_eq!(
    //     1,
    //     db.search_tracks(
    //         collection_id,
    //         &Default::default(),
    //         Some(TrackFilter::TitlePhrase(TitlePhraseFilter {
    //             modifier: None,
    //             scope: Some(Scope::Track),
    //             kinds: Default::default(),
    //             name_terms: vec!["aA".into(), "EEee".into()],
    //         })),
    //         Default::default(),
    //         &mut DummyCollector::new(),
    //     )?
    // );
    Ok(())
}
