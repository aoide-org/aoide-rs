// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::File,
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use diesel::Connection as _;
use nonicle::CanonicalizeInto as _;
use url::Url;

use aoide_core::{
    Collection, CollectionUid, PlainTag, TagFacetId, TagLabel, TagsMap, Track, TrackEntity,
    audio::PositionMs,
    collection::MediaSourceConfig,
    media::content::{ContentPath, ContentPathConfig, VirtualFilePathConfig},
    music::{
        key::{KeyCode, KeySignature},
        tempo::TempoBpm,
    },
    tag::FacetKey,
    track::tag::FACET_ID_COMMENT,
    track::{
        AdvisoryRating,
        cue::{Cue, InMarker},
    },
    util::color::{Color, RgbColor},
    util::url::BaseUrl,
};
use aoide_core_api::media::SyncMode;
use aoide_media_file::io::{export::ExportTrackConfig, import::ImportTrackConfig};
use aoide_repo::track::ReplaceMode;
use aoide_repo_sqlite::{DbConnection, initialize_database, run_migrations};
use aoide_usecases::track::validate_input;

use super::{
    Conflict, ExportParams, Outcome, Params, RevertOutcome, export_one, revert_one, update_one,
};

const ASSET_FILE_PATH: &str = "../media-file/tests/assets/empty.mp3";

const FILE_NAME: &str = "track.mp3";

struct Fixture {
    connection: DbConnection,
    collection_uid: CollectionUid,
    root_dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> anyhow::Result<Self> {
        let mut connection = DbConnection::establish(":memory:")?;
        initialize_database(&mut connection)?;
        run_migrations(&mut connection).map_err(|err| anyhow!(err))?;
        let root_dir = tempfile::tempdir()?;
        std::fs::copy(ASSET_FILE_PATH, root_dir.path().join(FILE_NAME))?;
        let root_url = Url::from_directory_path(root_dir.path())
            .map_err(|()| anyhow!("invalid root directory"))?;
        let collection = Collection {
            title: "Test Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                    root_url: BaseUrl::new(root_url),
                    excluded_paths: Vec::new(),
                }),
            },
        };
        let collection_uid = crate::collection::create(&mut connection, collection)?
            .hdr
            .uid
            .clone();
        Ok(Self {
            connection,
            collection_uid,
            root_dir,
        })
    }

    fn file_path(&self) -> std::path::PathBuf {
        self.root_dir.path().join(FILE_NAME)
    }

    fn import_track(&mut self) -> anyhow::Result<TrackEntity> {
        let params = aoide_usecases::track::import_and_replace::Params {
            sync_mode: SyncMode::Modified,
            import_config: ImportTrackConfig::default(),
            replace_mode: ReplaceMode::UpdateOrCreate,
        };
        let outcome = crate::track::import_and_replace::import_and_replace_many_by_local_file_path(
            &mut self.connection,
            &self.collection_uid,
            [ContentPath::new(FILE_NAME.into())],
            1,
            &params,
            &std::convert::identity,
            &AtomicBool::new(false),
        )?;
        let [entity] = <[_; 1]>::try_from(outcome.summary.created)
            .map_err(|_| anyhow!("track not created"))?;
        Ok(entity)
    }

    fn revert_track(&mut self, entity: &TrackEntity) -> anyhow::Result<RevertOutcome> {
        let outcome = self.connection.transaction(|connection| {
            revert_one(
                connection,
                &self.collection_uid,
                &entity.hdr,
                &ImportTrackConfig::default(),
            )
        })?;
        Ok(outcome)
    }

    /// Save an edited track in the database without writing the file.
    fn save_track(&mut self, entity: &TrackEntity, track: Track) -> anyhow::Result<TrackEntity> {
        let Outcome::Updated { entity, .. } = self.update_track(entity, track, false)? else {
            panic!("rejected");
        };
        Ok(entity)
    }

    fn update_track(
        &mut self,
        entity: &TrackEntity,
        track: Track,
        write_file: bool,
    ) -> anyhow::Result<Outcome> {
        let (validated_input, _) = validate_input(track)?;
        let export = write_file.then(export_params);
        let params = Params { export };
        let outcome = self.connection.transaction(|connection| {
            update_one(
                connection,
                &self.collection_uid,
                &entity.hdr,
                validated_input,
                &params,
            )
        })?;
        Ok(outcome)
    }
}

fn export_params() -> ExportParams {
    ExportParams {
        export_config: ExportTrackConfig {
            encode_gigtags: Some(FACET_ID_COMMENT.clone()),
            ..Default::default()
        },
        import_config: ImportTrackConfig::default(),
    }
}

fn custom_tags() -> TagsMap<'static> {
    let mut tags_map = TagsMap::default();
    tags_map.insert_one(
        FacetKey::unfaceted(),
        PlainTag {
            label: TagLabel::clamp_from("warmup"),
            ..Default::default()
        },
    );
    tags_map.insert_one(
        FacetKey::new(TagFacetId::clamp_from("venue")),
        PlainTag {
            label: TagLabel::clamp_from("club-x"),
            ..Default::default()
        },
    );
    tags_map
}

fn edit_track(track: &Track) -> Track {
    let mut track = track.clone();
    track.set_track_title("Edited title");
    track.set_track_artist("Edited artist".to_owned());
    let mut tags_map: TagsMap<'static> = std::mem::take(&mut track.tags).untie().into();
    tags_map.insert_one(
        FacetKey::from(FACET_ID_COMMENT.clone()),
        PlainTag {
            label: TagLabel::clamp_from("Some comment"),
            ..Default::default()
        },
    );
    tags_map.merge(custom_tags());
    track.tags = tags_map.canonicalize_into();
    track
}

fn touch_file(file_path: &Path) -> anyhow::Result<()> {
    let modified = File::open(file_path)?.metadata()?.modified()?;
    let file = File::options().write(true).open(file_path)?;
    file.set_modified(modified.max(SystemTime::now()) + Duration::from_secs(10))?;
    Ok(())
}

#[test]
fn update_and_write_file_with_custom_tags() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);

    let Outcome::Updated { entity, exported } =
        fixture.update_track(&imported_entity, edited_track.clone(), true)?
    else {
        panic!("rejected");
    };
    assert!(exported);
    assert!(entity.hdr.rev > imported_entity.hdr.rev);
    // The track is synchronized with the file after re-importing it.
    assert_eq!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);
    assert_ne!(
        imported_entity.body.track.media_source.content.link.rev,
        entity.body.track.media_source.content.link.rev,
    );
    // All edited metadata survived the roundtrip through the file.
    assert_eq!(Some("Edited title"), entity.body.track.track_title());
    assert_eq!(Some("Edited artist"), entity.body.track.track_artist());
    assert_eq!(edited_track.tags, entity.body.track.tags);
    Ok(())
}

#[test]
fn update_without_writing_file() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);

    let Outcome::Updated { entity, exported } =
        fixture.update_track(&imported_entity, edited_track.clone(), false)?
    else {
        panic!("rejected");
    };
    assert!(!exported);
    assert_eq!(edited_track.titles, entity.body.track.titles);
    assert_eq!(edited_track.actors, entity.body.track.actors);
    assert_eq!(edited_track.tags, entity.body.track.tags);
    // The track is no longer synchronized with the file.
    assert_ne!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);
    Ok(())
}

#[test]
fn reject_update_if_entity_modified() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let Outcome::Updated { entity, .. } =
        fixture.update_track(&imported_entity, edited_track.clone(), false)?
    else {
        panic!("rejected");
    };

    // Save again with the stale entity header.
    let outcome = fixture.update_track(&imported_entity, edited_track, true)?;
    assert!(matches!(
        outcome,
        Outcome::Rejected(Conflict::EntityRevision { expected, actual })
            if expected == imported_entity.hdr.rev && actual == entity.hdr.rev
    ));
    Ok(())
}

#[test]
fn reject_update_if_file_modified() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);

    touch_file(&fixture.file_path())?;

    for write_file in [false, true] {
        let outcome = fixture.update_track(&imported_entity, edited_track.clone(), write_file)?;
        assert!(matches!(
            outcome,
            Outcome::Rejected(Conflict::ContentRevision { .. })
        ));
    }
    Ok(())
}

#[test]
fn export_unsynchronized_track() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let Outcome::Updated { entity, .. } =
        fixture.update_track(&imported_entity, edited_track.clone(), false)?
    else {
        panic!("rejected");
    };
    assert_ne!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);

    let collection_uid = fixture.collection_uid.clone();
    let outcome = fixture.connection.transaction(|connection| {
        export_one(connection, &collection_uid, &entity.hdr, export_params())
    })?;
    let Outcome::Updated {
        entity: exported_entity,
        exported,
    } = outcome
    else {
        panic!("rejected");
    };
    assert!(exported);
    assert_eq!(
        Some(exported_entity.hdr.rev),
        exported_entity.body.last_synchronized_rev
    );
    assert_eq!(edited_track.tags, exported_entity.body.track.tags);
    Ok(())
}

#[test]
fn reject_update_of_track_from_other_collection() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let other_collection = Collection {
        title: "Other Collection".into(),
        notes: None,
        kind: None,
        color: None,
        media_source_config: MediaSourceConfig {
            content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                // Windows requires a drive letter for file URLs to be convertible into paths.
                root_url: BaseUrl::parse_strict(if cfg!(windows) {
                    "file:///C:/other/"
                } else {
                    "file:///other/"
                })?,
                excluded_paths: Vec::new(),
            }),
        },
    };
    let other_collection_uid =
        crate::collection::create(&mut fixture.connection, other_collection)?
            .hdr
            .uid
            .clone();
    let result = fixture.connection.transaction(|connection| {
        export_one(
            connection,
            &other_collection_uid,
            &imported_entity.hdr,
            export_params(),
        )
    });
    assert!(matches!(
        result,
        Err(crate::Error::Repository(aoide_repo::RepoError::NotFound))
    ));
    Ok(())
}

/// Modify all properties that might be missing in a file.
fn edit_track_completely(track: &Track) -> Track {
    let mut track = edit_track(track);
    track.metrics.tempo_bpm = Some(TempoBpm::new(123.0));
    track.metrics.key_signature = Some(KeySignature::new(KeyCode::Cmaj));
    track.color = Some(Color::Rgb(RgbColor::BLACK));
    track.publisher = Some("Edited publisher".to_owned());
    track.copyright = Some("Edited copyright".to_owned());
    track.advisory_rating = Some(AdvisoryRating::Explicit);
    track
}

fn assert_reverted(outcome: RevertOutcome, expected: &TrackEntity) -> TrackEntity {
    let RevertOutcome::Reverted(entity) = outcome else {
        panic!("rejected");
    };
    assert_eq!(expected.body.track, entity.body.track);
    assert_eq!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);
    entity
}

#[test]
fn revert_restores_all_properties_of_a_file_without_tags() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let file_contents = std::fs::read(fixture.file_path())?;
    let edited_track = edit_track_completely(&imported_entity.body.track);
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;
    assert_ne!(imported_entity.body.track, edited_entity.body.track);
    assert_ne!(
        Some(edited_entity.hdr.rev),
        edited_entity.body.last_synchronized_rev
    );

    let outcome = fixture.revert_track(&edited_entity)?;

    let reverted_entity = assert_reverted(outcome, &imported_entity);
    assert!(reverted_entity.hdr.rev > edited_entity.hdr.rev);
    assert_eq!(imported_entity.hdr.uid, reverted_entity.hdr.uid);
    // The file must not be touched.
    assert_eq!(file_contents, std::fs::read(fixture.file_path())?);
    Ok(())
}

#[test]
fn revert_restores_the_written_metadata_and_removes_properties_missing_in_the_file()
-> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let Outcome::Updated {
        entity: written_entity,
        ..
    } = fixture.update_track(&imported_entity, edited_track, true)?
    else {
        panic!("rejected");
    };
    assert_eq!(
        Some("Edited title"),
        written_entity.body.track.track_title()
    );
    let mut edited_track = edit_track_completely(&written_entity.body.track);
    edited_track.set_track_title("Not written");
    let edited_entity = fixture.save_track(&written_entity, edited_track)?;

    let outcome = fixture.revert_track(&edited_entity)?;

    let reverted_entity = assert_reverted(outcome, &written_entity);
    assert_eq!(
        Some("Edited title"),
        reverted_entity.body.track.track_title()
    );
    assert_eq!(None, reverted_entity.body.track.metrics.tempo_bpm);
    assert_eq!(None, reverted_entity.body.track.color);
    Ok(())
}

#[test]
fn revert_marks_a_track_as_synchronized_that_has_been_edited_back() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;
    let unchanged_track = imported_entity.body.track.clone();
    let edited_back_entity = fixture.save_track(&edited_entity, unchanged_track)?;
    assert_eq!(None, edited_back_entity.body.track.track_title());
    assert_ne!(
        Some(edited_back_entity.hdr.rev),
        edited_back_entity.body.last_synchronized_rev
    );

    let outcome = fixture.revert_track(&edited_back_entity)?;

    assert_reverted(outcome, &imported_entity);
    Ok(())
}

#[test]
fn replacing_with_an_identical_track_marks_it_as_synchronized_if_requested() -> anyhow::Result<()> {
    use aoide_repo::track::{CollectionRepo as _, EntityRepo as _, ReplaceOutcome, ReplaceParams};

    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;
    let collection_uid = fixture.collection_uid.clone();
    let mut repo = crate::RepoConnection::new(&mut fixture.connection);
    let collection_id =
        aoide_repo::collection::EntityRepo::resolve_collection_id(&mut repo, &collection_uid)?;
    let (_, unsynchronized_entity) = repo.load_track_entity_by_uid(&edited_entity.hdr.uid)?;
    assert_ne!(
        Some(unsynchronized_entity.hdr.rev),
        unsynchronized_entity.body.last_synchronized_rev
    );
    let replace_params = |update_last_synchronized_rev| ReplaceParams {
        mode: ReplaceMode::UpdateOnly,
        preserve_collected_at: true,
        update_last_synchronized_rev,
    };

    let outcome = repo.replace_track_by_media_source_content_path(
        collection_id,
        replace_params(false),
        unsynchronized_entity.body.track.clone(),
    )?;
    assert!(matches!(outcome, ReplaceOutcome::Unchanged(..)));

    let outcome = repo.replace_track_by_media_source_content_path(
        collection_id,
        replace_params(true),
        unsynchronized_entity.body.track.clone(),
    )?;
    let ReplaceOutcome::Updated(_, _, entity) = outcome else {
        panic!("not updated: {outcome:?}");
    };
    assert_eq!(unsynchronized_entity.body.track, entity.body.track);
    assert_eq!(Some(entity.hdr.rev), entity.body.last_synchronized_rev);
    Ok(())
}

#[test]
fn revert_preserves_cues_that_are_not_provided_by_the_file() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let mut edited_track = edit_track(&imported_entity.body.track);
    let cues = vec![Cue {
        bank_index: 0,
        slot_index: None,
        in_marker: Some(InMarker {
            position: PositionMs::new(1000.0),
        }),
        out_marker: None,
        kind: None,
        label: None,
        color: None,
        flags: Default::default(),
    }];
    edited_track.cues = cues.clone().canonicalize_into();
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;

    let RevertOutcome::Reverted(reverted_entity) = fixture.revert_track(&edited_entity)? else {
        panic!("rejected");
    };

    assert_eq!(None, reverted_entity.body.track.track_title());
    assert_eq!(
        cues,
        reverted_entity
            .body
            .track
            .cues
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn revert_accepts_a_file_that_has_been_modified_on_disk() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;
    touch_file(&fixture.file_path())?;

    let RevertOutcome::Reverted(reverted_entity) = fixture.revert_track(&edited_entity)? else {
        panic!("rejected");
    };

    assert_eq!(None, reverted_entity.body.track.track_title());
    assert_ne!(
        imported_entity.body.track.media_source.content.link.rev,
        reverted_entity.body.track.media_source.content.link.rev
    );
    assert_eq!(
        Some(reverted_entity.hdr.rev),
        reverted_entity.body.last_synchronized_rev
    );
    Ok(())
}

#[test]
fn revert_of_a_missing_file_fails_and_preserves_the_edits() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;
    std::fs::remove_file(fixture.file_path())?;

    assert!(fixture.revert_track(&edited_entity).is_err());

    let mut repo = crate::RepoConnection::new(&mut fixture.connection);
    let (_, entity) =
        aoide_repo::track::EntityRepo::load_track_entity_by_uid(&mut repo, &edited_entity.hdr.uid)?;
    assert_eq!(edited_entity.hdr, entity.raw.hdr);
    assert_eq!(edited_entity.body.track, entity.body.track);
    Ok(())
}

#[test]
fn revert_is_rejected_if_the_track_has_been_modified_in_the_meantime() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let imported_entity = fixture.import_track()?;
    let edited_track = edit_track(&imported_entity.body.track);
    let edited_entity = fixture.save_track(&imported_entity, edited_track)?;

    let outcome = fixture.revert_track(&imported_entity)?;

    assert!(matches!(
        outcome,
        RevertOutcome::Rejected(Conflict::EntityRevision { .. })
    ));
    let mut repo = crate::RepoConnection::new(&mut fixture.connection);
    let (_, entity) =
        aoide_repo::track::EntityRepo::load_track_entity_by_uid(&mut repo, &edited_entity.hdr.uid)?;
    assert_eq!(edited_entity.body.track, entity.body.track);
    Ok(())
}
