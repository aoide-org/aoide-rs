// aoide.org - Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![warn(clippy::clone_on_ref_ptr)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `..Default::default()` expressions
// is not needed since the context is obvious.
#![allow(clippy::default_trait_access)]
// Using wildcard imports consciously is acceptable.
#![allow(clippy::wildcard_imports)]
// Importing all enum variants into a narrow, local scope is acceptable.
#![allow(clippy::enum_glob_use)]
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

use std::{borrow::Cow, fs, path::Path};

use num_traits::cast::ToPrimitive as _;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::{AllQuery, Query as _, TermQuery},
    schema::{Field, IndexRecordOption, Schema, Value, INDEXED, STORED, STRING, TEXT},
    Document, Index, Searcher, TantivyError, Term,
};

use aoide_core::{
    media::content::ContentMetadata,
    tag::{FacetId as TagFacetId, FacetedTags, PlainTag},
    track::{
        actor::Actors,
        tag::{
            FACET_ACOUSTICNESS, FACET_AROUSAL, FACET_COMMENT, FACET_DANCEABILITY, FACET_ENERGY,
            FACET_GENRE, FACET_GROUPING, FACET_INSTRUMENTALNESS, FACET_LIVENESS, FACET_MOOD,
            FACET_POPULARITY, FACET_SPEECHINESS, FACET_VALENCE,
        },
        PlayCounter,
    },
    util::clock::{DateTime, DateYYYYMMDD},
    CollectionUid, EncodedEntityUid, EntityRevision, EntityUid, TrackEntity, TrackUid,
};

const COLLECTION_UID: &str = "collection_uid";
const UID: &str = "uid";
const REV: &str = "rev";
const CONTENT_PATH: &str = "content_path";
const CONTENT_TYPE: &str = "content_type";
const COLLECTED_AT: &str = "collected_at";
const DURATION_MS: &str = "duration_ms";
const TRACK_TITLE: &str = "track_title";
const TRACK_ARTIST: &str = "track_artist";
const ALBUM_TITLE: &str = "album_title";
const ALBUM_ARTIST: &str = "album_artist";
const RECORDED_AT_YYYYMMDD: &str = "recorded_at_yyyymmdd";
const RELEASED_AT_YYYYMMDD: &str = "released_at_yyyymmdd";
const RELEASED_ORIG_AT_YYYYMMDD: &str = "released_orig_at_yyyymmdd";
const TEMPO_BPM: &str = "tempo_bpm";
const KEY_CODE: &str = "key_signature";
const TIMES_PLAYED: &str = "times_played";
const LAST_PLAYED_AT: &str = "last_played_at";
const GENRE: &str = "genre";
const MOOD: &str = "mood";
const COMMENT: &str = "comment";
const GROUPING: &str = "grouping";
const TAG: &str = "tag";
const ACOUSTICNESS: &str = "acousticness";
const AROUSAL: &str = "arousal";
const DANCEABILITY: &str = "danceability";
const ENERGY: &str = "energy";
const INSTRUMENTALNESS: &str = "instrumentalness";
const LIVENESS: &str = "liveness";
const POPULARITY: &str = "popularity";
const SPEECHINESS: &str = "speechiness";
const VALENCE: &str = "valence";

#[derive(Debug, Clone)]
pub struct TrackFields {
    pub collection_uid: Field,
    pub uid: Field,
    pub rev: Field,
    pub content_path: Field,
    pub content_type: Field,
    pub collected_at: Field,
    pub duration_ms: Field,
    pub track_title: Field,
    pub track_artist: Field,
    pub album_title: Field,
    pub album_artist: Field,
    pub recorded_at_yyyymmdd: Field,
    pub released_at_yyyymmdd: Field,
    pub released_orig_at_yyyymmdd: Field,
    pub tempo_bpm: Field,
    pub key_code: Field,
    pub times_played: Field,
    pub last_played_at: Field,
    pub genre: Field,
    pub mood: Field,
    pub comment: Field,
    pub grouping: Field,
    pub tag: Field,
    pub acousticness: Field,
    pub arousal: Field,
    pub danceability: Field,
    pub energy: Field,
    pub instrumentalness: Field,
    pub liveness: Field,
    pub popularity: Field,
    pub speechiness: Field,
    pub valence: Field,
}

fn add_date_field(doc: &mut Document, field: Field, date_time: DateTime) {
    doc.add_date(field, tantivy::DateTime::from_utc(date_time.to_inner()));
}

const TAG_LABEL_PREFIX: char = '#';

impl TrackFields {
    fn format_tag_field_text<'a>(
        &self,
        facet_id: Option<&TagFacetId<'_>>,
        tag: &'a PlainTag<'a>,
    ) -> Option<(Field, Cow<'a, str>)> {
        let PlainTag { label, score } = tag;
        let Some(label) = label else {
            return None
        };
        debug_assert!(!label.is_empty());
        if *score != Default::default() {
            // TODO: How to take the score into account?
            if let Some(facet_id) = facet_id {
                log::trace!("Ignoring non-default score of \"{facet_id}\" tag: {tag:?}");
            } else {
                log::trace!("Ignoring non-default score of plain tag: {tag:?}");
            }
        }
        // Special case handling for faceted tags with dedicated document fields
        match facet_id.map(TagFacetId::as_str) {
            Some(FACET_COMMENT) => {
                return Some((self.comment, Cow::Borrowed(label.as_str())));
            }
            Some(FACET_GENRE) => {
                return Some((self.genre, Cow::Borrowed(label.as_str())));
            }
            Some(FACET_GROUPING) => {
                return Some((self.grouping, Cow::Borrowed(label.as_str())));
            }
            Some(FACET_MOOD) => {
                return Some((self.mood, Cow::Borrowed(label.as_str())));
            }
            _ => {
                // Generic tag field
                let facet_prefix = facet_id
                    .map(|facet_id| {
                        debug_assert!(!facet_id.is_empty());
                        debug_assert!(!facet_id.as_str().contains(TAG_LABEL_PREFIX));
                        facet_id.as_str()
                    })
                    .unwrap_or_default();
                let text = if label.as_str().starts_with(TAG_LABEL_PREFIX) {
                    // Omit the redundant prefix
                    if facet_prefix.is_empty() {
                        return Some((self.tag, Cow::Borrowed(label.as_str())));
                    }
                    format!("{facet_prefix}{label}")
                } else {
                    format!("{facet_prefix}{TAG_LABEL_PREFIX}{label}")
                };
                Some((self.tag, Cow::Owned(text)))
            }
        }
    }

    /// Create a new document from a track entity
    ///
    /// When storing tracks from multiple collections in a single index
    /// the `collection_uid` should be provided.
    #[allow(clippy::too_many_lines)] // TODO
    #[must_use]
    pub fn create_document(
        &self,
        collection_uid: Option<&CollectionUid>,
        entity: &TrackEntity,
        play_counter: Option<&PlayCounter>,
    ) -> Document {
        // TODO (optimization): Consuming the entity would avoid string allocations for text fields
        let mut doc = Document::new();
        if let Some(collection_uid) = collection_uid {
            doc.add_text(self.collection_uid, collection_uid);
        }
        doc.add_text(self.uid, &entity.hdr.uid);
        doc.add_u64(self.rev, entity.hdr.rev.to_inner());
        doc.add_text(
            self.content_path,
            &entity.body.track.media_source.content.link.path,
        );
        add_date_field(
            &mut doc,
            self.collected_at,
            entity.body.track.media_source.collected_at,
        );
        let ContentMetadata::Audio(audio_metadata) =
            &entity.body.track.media_source.content.metadata;
        if let Some(duration) = audio_metadata.duration {
            doc.add_f64(self.duration_ms, duration.to_inner());
        }
        if let Some(track_title) = entity.body.track.track_title() {
            doc.add_text(self.track_title, track_title);
        }
        // Index all track actors as `track_artist` by name, independent of their role
        for track_artist in &Actors::collect_all_unique_actor_names(entity.body.track.actors.iter())
        {
            doc.add_text(self.track_artist, track_artist);
        }
        if let Some(album_title) = entity.body.track.album_title() {
            doc.add_text(self.album_title, album_title);
        }
        // Index all album actors as `album_artist` by name, independent of their role
        for album_artist in &Actors::collect_all_unique_actor_names(entity.body.track.actors.iter())
        {
            doc.add_text(self.album_artist, album_artist);
        }
        if let Some(recorded_at_yyyymmdd) = entity.body.track.recorded_at.map(DateYYYYMMDD::from) {
            doc.add_i64(self.album_artist, recorded_at_yyyymmdd.to_inner().into());
        }
        if let Some(released_at_yyyymmdd) = entity.body.track.released_at.map(DateYYYYMMDD::from) {
            doc.add_i64(self.album_artist, released_at_yyyymmdd.to_inner().into());
        }
        if let Some(released_orig_at_yyyymmdd) =
            entity.body.track.released_orig_at.map(DateYYYYMMDD::from)
        {
            doc.add_i64(
                self.album_artist,
                released_orig_at_yyyymmdd.to_inner().into(),
            );
        }
        if let Some(tempo_bpm) = entity.body.track.metrics.tempo_bpm {
            doc.add_f64(self.tempo_bpm, tempo_bpm.to_inner());
        }
        if let Some(key_signature) = entity.body.track.metrics.key_signature {
            doc.add_u64(
                self.key_code,
                key_signature
                    .code()
                    .to_u64()
                    .expect("valid key signature code"),
            );
        }
        if let Some(play_counter) = play_counter {
            let PlayCounter {
                times_played,
                last_played_at,
            } = play_counter;
            if let Some(times_played) = times_played {
                doc.add_u64(self.times_played, *times_played);
            }
            if let Some(last_played_at) = last_played_at {
                add_date_field(&mut doc, self.last_played_at, *last_played_at);
            }
        }
        for tag in &entity.body.track.tags.plain {
            if let Some((field, text)) = self.format_tag_field_text(None, tag) {
                debug_assert_eq!(self.tag, field);
                doc.add_text(field, text);
            }
        }
        for faceted_tags in &entity.body.track.tags.facets {
            let FacetedTags { facet_id, tags } = faceted_tags;
            debug_assert!(!facet_id.is_empty());
            let score_field = match facet_id.as_str() {
                FACET_ACOUSTICNESS => self.acousticness,
                FACET_AROUSAL => self.arousal,
                FACET_DANCEABILITY => self.danceability,
                FACET_ENERGY => self.energy,
                FACET_INSTRUMENTALNESS => self.instrumentalness,
                FACET_LIVENESS => self.liveness,
                FACET_POPULARITY => self.popularity,
                FACET_SPEECHINESS => self.speechiness,
                FACET_VALENCE => self.valence,
                _ => {
                    for tag in tags {
                        if let Some((field, text)) = self.format_tag_field_text(Some(facet_id), tag)
                        {
                            doc.add_text(field, text);
                        } else {
                            log::debug!("Ignoring faceted tag \"{facet_id}\": {tag:?}");
                        }
                    }
                    continue;
                }
            };
            for tag in tags {
                let PlainTag { label, score } = tag;
                if label.is_some() {
                    log::debug!("Ignoring label of \"{facet_id}\" tag: {tag:?}");
                }
                doc.add_f64(score_field, score.value());
            }
        }
        doc
    }

    #[must_use]
    pub fn collection_uid_term(&self, collection_uid: &CollectionUid) -> Term {
        Term::from_field_text(
            self.collection_uid,
            EncodedEntityUid::from(collection_uid).as_str(),
        )
    }

    #[must_use]
    pub fn collection_uid_query(&self, collection_uid: &CollectionUid) -> TermQuery {
        TermQuery::new(
            self.collection_uid_term(collection_uid),
            IndexRecordOption::Basic,
        )
    }

    #[must_use]
    pub fn read_collection_uid(&self, doc: &Document) -> Option<CollectionUid> {
        doc.get_first(self.collection_uid)
            .and_then(Value::as_text)
            .map(EntityUid::decode_from)
            .transpose()
            .ok()
            .flatten()
            .map(CollectionUid::from_untyped)
    }

    #[must_use]
    pub fn uid_term(&self, uid: &TrackUid) -> Term {
        Term::from_field_text(self.uid, EncodedEntityUid::from(uid).as_str())
    }

    #[must_use]
    pub fn uid_query(&self, uid: &TrackUid) -> TermQuery {
        TermQuery::new(self.uid_term(uid), IndexRecordOption::Basic)
    }

    #[must_use]
    pub fn read_uid(&self, doc: &Document) -> Option<TrackUid> {
        doc.get_first(self.uid)
            .and_then(Value::as_text)
            .map(EntityUid::decode_from)
            .transpose()
            .ok()
            .flatten()
            .map(TrackUid::from_untyped)
    }

    #[must_use]
    pub fn read_rev(&self, doc: &Document) -> Option<EntityRevision> {
        doc.get_first(self.rev)
            .and_then(Value::as_u64)
            .map(EntityRevision::new)
    }

    pub fn find_rev_by_uid(
        &self,
        searcher: &Searcher,
        uid: &TrackUid,
    ) -> tantivy::Result<Option<EntityRevision>> {
        let query = self.uid_query(uid);
        // Search for 2 documents
        let top_docs = searcher.search(&query, &TopDocs::with_limit(2))?;
        debug_assert!(top_docs.len() <= 1);
        if let Some((_score, doc_addr)) = top_docs.into_iter().next() {
            let doc = searcher.doc(doc_addr)?;
            debug_assert_eq!(Some(uid), self.read_uid(&doc).as_ref());
            let rev = self.read_rev(&doc);
            debug_assert!(rev.is_some());
            Ok(rev)
        } else {
            Ok(None)
        }
    }
}

/// Create the schema for indexing tracks
///
/// Supports to index tracks from different collections.
#[must_use]
pub fn build_schema_for_tracks() -> (Schema, TrackFields) {
    let mut schema_builder = Schema::builder();
    let collection_uid = schema_builder.add_text_field(COLLECTION_UID, STRING | STORED);
    let uid = schema_builder.add_text_field(UID, STRING | STORED);
    let rev = schema_builder.add_u64_field(REV, INDEXED | STORED);
    let content_path = schema_builder.add_text_field(CONTENT_PATH, STRING);
    let content_type = schema_builder.add_text_field(CONTENT_TYPE, STRING);
    let collected_at = schema_builder.add_date_field(COLLECTED_AT, INDEXED);
    let duration_ms = schema_builder.add_f64_field(DURATION_MS, INDEXED);
    let track_title = schema_builder.add_text_field(TRACK_TITLE, TEXT);
    let track_artist = schema_builder.add_text_field(TRACK_ARTIST, TEXT);
    let album_title = schema_builder.add_text_field(ALBUM_TITLE, TEXT);
    let album_artist = schema_builder.add_text_field(ALBUM_ARTIST, TEXT);
    let recorded_at_yyyymmdd = schema_builder.add_i64_field(RECORDED_AT_YYYYMMDD, INDEXED);
    let released_at_yyyymmdd = schema_builder.add_i64_field(RELEASED_AT_YYYYMMDD, INDEXED);
    let released_orig_at_yyyymmdd =
        schema_builder.add_i64_field(RELEASED_ORIG_AT_YYYYMMDD, INDEXED);
    let tempo_bpm = schema_builder.add_f64_field(TEMPO_BPM, INDEXED);
    let key_code = schema_builder.add_u64_field(KEY_CODE, INDEXED);
    let times_played = schema_builder.add_u64_field(TIMES_PLAYED, INDEXED);
    let last_played_at = schema_builder.add_date_field(LAST_PLAYED_AT, INDEXED);
    let genre = schema_builder.add_text_field(GENRE, TEXT);
    let mood = schema_builder.add_text_field(MOOD, TEXT);
    let comment = schema_builder.add_text_field(COMMENT, TEXT);
    let grouping = schema_builder.add_text_field(GROUPING, TEXT);
    let tag = schema_builder.add_text_field(TAG, TEXT);
    let acousticness = schema_builder.add_f64_field(ACOUSTICNESS, INDEXED);
    let arousal = schema_builder.add_f64_field(AROUSAL, INDEXED);
    let danceability = schema_builder.add_f64_field(DANCEABILITY, INDEXED);
    let energy = schema_builder.add_f64_field(ENERGY, INDEXED);
    let instrumentalness = schema_builder.add_f64_field(INSTRUMENTALNESS, INDEXED);
    let liveness = schema_builder.add_f64_field(LIVENESS, INDEXED);
    let popularity = schema_builder.add_f64_field(POPULARITY, INDEXED);
    let speechiness = schema_builder.add_f64_field(SPEECHINESS, INDEXED);
    let valence = schema_builder.add_f64_field(VALENCE, INDEXED);
    let schema = schema_builder.build();
    let fields = TrackFields {
        collection_uid,
        uid,
        rev,
        content_path,
        content_type,
        collected_at,
        duration_ms,
        track_title,
        track_artist,
        album_title,
        album_artist,
        recorded_at_yyyymmdd,
        released_at_yyyymmdd,
        released_orig_at_yyyymmdd,
        tempo_bpm,
        key_code,
        times_played,
        last_played_at,
        genre,
        mood,
        comment,
        grouping,
        tag,
        acousticness,
        arousal,
        danceability,
        energy,
        instrumentalness,
        liveness,
        popularity,
        speechiness,
        valence,
    };
    (schema, fields)
}

#[derive(Debug)]
pub struct TrackIndex {
    pub fields: TrackFields,
    pub index: Index,
}

#[derive(Debug, Clone, Copy)]
pub enum IndexStorage<'p> {
    InMemory,
    TempDir,
    FileDir { dir_path: &'p Path },
}

impl TrackIndex {
    pub fn open_or_recreate(index_storage: IndexStorage<'_>) -> anyhow::Result<TrackIndex> {
        let (schema, fields) = build_schema_for_tracks();
        let index = match index_storage {
            IndexStorage::InMemory => {
                log::info!("Creating temporary track index in RAM");
                Index::create_in_ram(schema)
            }
            IndexStorage::TempDir => {
                log::info!("Creating temporary track index");
                Index::create_from_tempdir(schema)?
            }
            IndexStorage::FileDir { dir_path } => {
                fs::create_dir_all(dir_path)?;
                let index_dir = MmapDirectory::open(dir_path)?;
                match Index::open_or_create(index_dir, schema) {
                    Ok(index) => index,
                    Err(TantivyError::SchemaError(err)) => {
                        log::warn!("Deleting track index with incompatible schema: {err}");
                        // Delete existing index data
                        fs::remove_dir_all(dir_path)?;
                        // ...and retry.
                        return Self::open_or_recreate(index_storage);
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                }
            }
        };
        Ok(Self { fields, index })
    }

    pub fn count_all(&self) -> anyhow::Result<usize> {
        let searcher = self.index.reader()?.searcher();
        let count_all = AllQuery.count(&searcher)?;
        Ok(count_all)
    }
}

#[cfg(test)]
mod tests;
