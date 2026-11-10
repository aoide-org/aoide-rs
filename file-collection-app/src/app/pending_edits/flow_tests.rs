// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The whole application without a window.
//!
//! Runs on a real database and a music directory with real files.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use egui::{Pos2, RawInput, Rect, Vec2, ViewportEvent, ViewportId, ViewportInfo};
use url::Url;

use aoide::{
    TrackUid, api::media::SyncMode, backend_embedded::media::predefined_import_track_config,
    desktop_app::Environment, media::content::ContentPath, repo::track::ReplaceMode,
    usecases::track::import_and_replace::Params, util::fs::DirPath,
};

use crate::library::{
    Library, settings,
    track_search::{Memo, MemoState},
    ui::{CustomTagsDelta, MultiTrackEdit, MultiTrackFields},
};

use super::super::{
    App, TrackEditAction, TrackEditorState, TrackSearchAction, TrackSearchFetchedItems,
    message::{MediaTrackerSyncAction, PendingEditsAction, RevertUnsynchronizedTracksAction},
};
use super::*;

const TIMEOUT: Duration = Duration::from_secs(10);

struct TestDir(PathBuf);

impl TestDir {
    fn new(test_name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "aoide_pending_edits_test_{pid}_{test_name}",
            pid = std::process::id()
        ));
        drop(fs::remove_dir_all(&path));
        fs::create_dir_all(path.join("music")).unwrap();
        fs::create_dir_all(path.join("config")).unwrap();
        let asset = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../crates/media-file/tests/assets/empty.flac");
        fs::copy(asset, path.join("music").join("track.flac")).unwrap();
        Self(path)
    }

    fn track_file(&self) -> PathBuf {
        self.0.join("music").join("track.flac")
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

struct Flow {
    // Dropped before the directory.
    app: App,
    ctx: Context,
    rt: tokio::runtime::Runtime,
    dir: TestDir,
    entity_uid: TrackUid,
}

impl Flow {
    /// Start the application until the collection is ready.
    fn start(test_name: &str) -> Self {
        let dir = TestDir::new(test_name);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let settings = settings::State {
            database_url: Some(Url::from_file_path(dir.0.join("aoide.sqlite")).unwrap()),
            music_dir: Some(DirPath::from_owned(dir.0.join("music"))),
            collection_kind: None,
        };
        let env = Environment::commission(&settings.create_database_config().unwrap()).unwrap();
        let mdl = Model::new(Library::new(env, settings), dir.0.join("config"));
        let ctx = Context::default();
        let app = {
            // Tokio tasks are spawned from outside of the runtime.
            let _enter = rt.enter();
            App::with_context(ctx.clone(), rt.handle().clone(), mdl)
        };
        let mut flow = Self {
            app,
            ctx,
            rt,
            dir,
            entity_uid: TrackUid::from_untyped(aoide::EntityUid::random()),
        };
        flow.run_until("collection is ready", |app| {
            app.mdl.library.read_collection_state().is_ready()
        });
        flow
    }

    /// Start the application with an imported file.
    fn new(test_name: &str) -> Self {
        let mut flow = Self::start(test_name);
        flow.import_track_file();
        // Independent of fetching the track list in the background.
        flow.app
            .mdl
            .mode
            .get_or_insert_with(|| ModelMode::TrackSearch(Default::default()));
        flow.run_until("collection is ready again", |app| {
            app.mdl
                .library
                .read_current_state()
                .could_export_unsynchronized_tracks()
                && app.mdl.modified_track_count.count == Some(0)
                && !app.mdl.modified_track_count.is_counting
        });
        flow.entity_uid = flow.load_entity_uid();
        flow
    }

    /// Import the file independent of the state of the application.
    fn import_track_file(&self) {
        let library = &self.app.mdl.library;
        let collection_uid = library
            .read_collection_state()
            .entity_uid()
            .cloned()
            .unwrap();
        let outcome = self
            .rt
            .block_on(
                aoide::backend_embedded::track::import_and_replace_many_by_local_file_path(
                    library.env().db_gatekeeper(),
                    collection_uid,
                    Params {
                        sync_mode: SyncMode::Once,
                        import_config: predefined_import_track_config(),
                        replace_mode: ReplaceMode::UpdateOrCreate,
                    },
                    [ContentPath::from("track.flac")],
                    Some(1),
                    |track| track,
                    Arc::new(AtomicBool::new(false)),
                ),
            )
            .unwrap();
        assert_eq!(1, outcome.summary.created.len(), "{outcome:?}");
    }

    fn load_entity_uid(&self) -> TrackUid {
        let library = &self.app.mdl.library;
        let collection_uid = library
            .read_collection_state()
            .entity_uid()
            .cloned()
            .unwrap();
        let tracks = self
            .rt
            .block_on(aoide::backend_embedded::track::search(
                library.env().db_gatekeeper(),
                collection_uid,
                Default::default(),
                Default::default(),
            ))
            .unwrap();
        assert_eq!(1, tracks.len());
        tracks[0].hdr.uid.clone()
    }

    /// Run a frame and return the commands for the window.
    fn run_frame(&mut self, close_requested: bool) -> Vec<ViewportCommand> {
        let mut raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1400.0, 900.0))),
            ..Default::default()
        };
        raw_input.viewports.insert(
            ViewportId::ROOT,
            ViewportInfo {
                events: if close_requested {
                    vec![ViewportEvent::Close]
                } else {
                    Vec::new()
                },
                ..Default::default()
            },
        );
        let app = &mut self.app;
        let mut output = self.ctx.run_ui(raw_input, |ui| app.run_frame(ui));
        output.textures_delta.clear();
        output
            .viewport_output
            .remove(&ViewportId::ROOT)
            .map(|viewport| viewport.commands)
            .unwrap_or_default()
    }

    /// Run frames and return all commands for the window.
    fn run_until(
        &mut self,
        description: &str,
        condition: impl Fn(&App) -> bool,
    ) -> Vec<ViewportCommand> {
        let started = Instant::now();
        let mut commands = Vec::new();
        loop {
            commands.extend(self.run_frame(false));
            if condition(&self.app) {
                return commands;
            }
            assert!(
                started.elapsed() < TIMEOUT,
                "timed out waiting until {description}: mode = {mode:?}, collection = {collection:?}, library = {library:?}",
                mode = self.app.mdl.mode,
                collection = *self.app.mdl.library.read_collection_state(),
                library = self.app.mdl.library.state(),
            );
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn apply_genres(&mut self, genres: &str) {
        let edit = MultiTrackEdit::new(
            MultiTrackFields {
                genres: Some(genres.to_owned()),
                ..Default::default()
            },
            CustomTagsDelta::default(),
        )
        .unwrap();
        self.app.msg_tx.send_action(TrackEditAction::Apply {
            entity_uid: self.entity_uid.clone(),
            edit: Box::new(edit),
        });
        self.run_until("track is modified", |app| {
            app.mdl.modified_track_count.count == Some(1)
                && !app.mdl.modified_track_count.is_counting
                && app.mdl.active_track_writes == 0
        });
    }

    fn open_track_editor_with_title(&mut self, title: &str) {
        self.app
            .msg_tx
            .send_action(TrackEditAction::Open(self.entity_uid.clone()));
        self.run_until("track is loaded", |app| {
            matches!(
                &app.mdl.mode,
                Some(ModelMode::TrackSearch(TrackSearchMode {
                    edit: Some(TrackSearchEdit::Single(editor)),
                    ..
                })) if matches!(editor.state, TrackEditorState::Ready { .. })
            )
        });
        self.run_frame(false);
        self.app
            .ui_data
            .editor_draft
            .as_mut()
            .unwrap()
            .draft_mut()
            .title = title.to_owned();
    }

    fn pending_edits(&mut self) -> PendingEdits {
        PendingEdits::new(&self.app.mdl, &mut self.app.ui_data)
    }

    /// Request closing the window.
    fn request_close(&mut self) {
        let commands = self.run_frame(true);
        assert!(commands.contains(&ViewportCommand::CancelClose));
    }

    fn file_contains(&self, text: &str) -> bool {
        file_position(&self.dir.track_file(), text).is_some()
    }
}

fn file_position(path: &Path, text: &str) -> Option<usize> {
    fs::read(path)
        .unwrap()
        .windows(text.len())
        .position(|window| window == text.as_bytes())
}

fn is_close_confirmed(app: &App) -> bool {
    app.mdl.close_state == CloseState::Confirmed
}

fn is_export_finished(app: &App) -> bool {
    matches!(
        app.mdl.mode,
        Some(ModelMode::ExportUnsynchronizedTracks {
            final_outcome: Some(_),
            ..
        })
    )
}

#[test]
fn closes_immediately_without_pending_edits() {
    let mut flow = Flow::new("closes_immediately_without_pending_edits");
    flow.request_close();
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);
    assert!(commands.contains(&ViewportCommand::Close));
    // The window is actually closed in response.
    assert!(!flow.run_frame(true).contains(&ViewportCommand::CancelClose));
}

#[test]
fn stays_open_until_decided_with_modified_tracks() {
    let mut flow = Flow::new("stays_open_until_decided_with_modified_tracks");
    flow.apply_genres("Techno\nAmbient\nHouse");
    flow.request_close();
    let commands = flow.run_until("close dialog is open", |app| {
        app.mdl.close_state == CloseState::Requested && !app.mdl.modified_track_count.is_counting
    });
    assert!(!commands.contains(&ViewportCommand::Close));
    for _ in 0..10 {
        assert!(!flow.run_frame(false).contains(&ViewportCommand::Close));
    }

    flow.app.msg_tx.send_action(PendingEditsAction::CancelClose);
    flow.run_until("closing is cancelled", |app| {
        app.mdl.close_state == CloseState::NotRequested
    });
    assert_eq!(Some(1), flow.pending_edits().modified_track_count);
}

#[test]
fn saving_writes_modified_tracks_in_order_and_closes() {
    let mut flow = Flow::new("saving_writes_modified_tracks_in_order_and_closes");
    let unmodified_file = fs::read(flow.dir.track_file()).unwrap();
    flow.apply_genres("Techno\nAmbient\nHouse");
    assert_eq!(unmodified_file, fs::read(flow.dir.track_file()).unwrap());

    flow.request_close();
    flow.run_until("close dialog is open", |app| {
        app.mdl.close_state == CloseState::Requested && !app.mdl.modified_track_count.is_counting
    });
    let pending_edits = flow.pending_edits();
    assert!(pending_edits.could_save());
    flow.app.msg_tx.send_action(pending_edits.save_action());
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);
    assert!(commands.contains(&ViewportCommand::Close));

    assert_eq!(Some(0), flow.app.mdl.modified_track_count.count);
    let techno = file_position(&flow.dir.track_file(), "Techno").expect("written");
    let ambient = file_position(&flow.dir.track_file(), "Ambient").expect("written");
    let house = file_position(&flow.dir.track_file(), "House").expect("written");
    assert!(
        techno < ambient && ambient < house,
        "{techno} {ambient} {house}"
    );
}

#[test]
fn saving_on_demand_writes_the_files_and_stays_open() {
    let mut flow = Flow::new("saving_on_demand_writes_the_files_and_stays_open");
    flow.apply_genres("Techno\nAmbient");
    let pending_edits = flow.pending_edits();
    assert_eq!("Save pending edits (1)", pending_edits.save_button_text());
    flow.app.msg_tx.send_action(pending_edits.save_action());
    let commands = flow.run_until("nothing is pending", |app| {
        is_export_finished(app)
            && app.mdl.modified_track_count.count == Some(0)
            && !app.mdl.modified_track_count.is_counting
    });
    assert!(!commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::NotRequested, flow.app.mdl.close_state);
    assert!(flow.file_contains("Techno"));
    assert!(!flow.pending_edits().could_save());
}

#[test]
fn saving_on_demand_saves_the_track_editor_draft_first() {
    let mut flow = Flow::new("saving_on_demand_saves_the_track_editor_draft_first");
    flow.open_track_editor_with_title("Draft title");
    let pending_edits = flow.pending_edits();
    assert!(matches!(pending_edits.draft, DraftEdit::Valid(_)));
    assert!(pending_edits.could_save());
    flow.app.msg_tx.send_action(pending_edits.save_action());
    flow.run_until("files are written", |app| {
        is_export_finished(app)
            && app.mdl.modified_track_count.count == Some(0)
            && !app.mdl.modified_track_count.is_counting
    });
    assert!(flow.file_contains("Draft title"));
    assert!(!flow.pending_edits().is_draft_modified());
}

#[test]
fn saving_the_draft_before_closing_writes_it_and_closes() {
    let mut flow = Flow::new("saving_the_draft_before_closing_writes_it_and_closes");
    flow.open_track_editor_with_title("Draft title");
    flow.request_close();
    flow.run_until("close dialog is open", |app| {
        app.mdl.close_state == CloseState::Requested && !app.mdl.modified_track_count.is_counting
    });
    assert!(flow.pending_edits().is_draft_modified());
    let action = flow.pending_edits().save_action();
    flow.app.msg_tx.send_action(action);
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);
    assert!(commands.contains(&ViewportCommand::Close));
    assert!(flow.file_contains("Draft title"));
}

#[test]
fn closing_without_writing_keeps_tracks_modified_and_drops_the_draft() {
    let mut flow = Flow::new("closing_without_writing_keeps_tracks_modified_and_drops_the_draft");
    flow.apply_genres("Techno\nAmbient");
    flow.open_track_editor_with_title("Draft title");
    let unmodified_file = fs::read(flow.dir.track_file()).unwrap();
    flow.request_close();
    flow.run_until("close dialog is open", |app| {
        app.mdl.close_state == CloseState::Requested && !app.mdl.modified_track_count.is_counting
    });
    flow.app
        .msg_tx
        .send_action(PendingEditsAction::CloseWithoutWriting);
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);
    assert!(commands.contains(&ViewportCommand::Close));
    assert_eq!(unmodified_file, fs::read(flow.dir.track_file()).unwrap());

    // The modified track is still pending for the next start.
    let db = flow.app.mdl.library.env().db_gatekeeper();
    let collection_uid = flow
        .app
        .mdl
        .library
        .read_collection_state()
        .entity_uid()
        .cloned()
        .unwrap();
    let unsynchronized = flow
        .rt
        .block_on(aoide::backend_embedded::track::find_unsynchronized(
            db,
            collection_uid,
            Default::default(),
            Default::default(),
        ))
        .unwrap();
    assert_eq!(1, unsynchronized.len());
    let entity = flow
        .rt
        .block_on(aoide::backend_embedded::track::load_one(
            db,
            flow.entity_uid.clone(),
        ))
        .unwrap();
    assert_ne!(Some("Draft title"), entity.body.track.track_title());
}

#[test]
fn stays_open_if_a_file_could_not_be_written() {
    let mut flow = Flow::new("stays_open_if_a_file_could_not_be_written");
    flow.apply_genres("Techno\nAmbient");
    // Files that have been modified on disk must not be overwritten.
    std::thread::sleep(Duration::from_millis(1100));
    let mut file = fs::read(flow.dir.track_file()).unwrap();
    file.extend_from_slice(b"modified on disk");
    fs::write(flow.dir.track_file(), &file).unwrap();

    flow.request_close();
    flow.run_until("close dialog is open", |app| {
        app.mdl.close_state == CloseState::Requested && !app.mdl.modified_track_count.is_counting
    });
    let action = flow.pending_edits().save_action();
    flow.app.msg_tx.send_action(action);
    let commands = flow.run_until("writing is finished", |app| {
        is_export_finished(app) && !app.mdl.modified_track_count.is_counting
    });
    assert!(!commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::Requested, flow.app.mdl.close_state);
    assert_eq!(Some(1), flow.app.mdl.modified_track_count.count);
    assert_eq!(file, fs::read(flow.dir.track_file()).unwrap());
    // The user is still able to decide.
    assert!(!flow.pending_edits().is_writing);

    flow.app
        .msg_tx
        .send_action(PendingEditsAction::CloseWithoutWriting);
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);
    assert!(commands.contains(&ViewportCommand::Close));
}

/// Synchronize and list tracks like a user, i.e. without bypassing the application.
fn run_ui_paths(test_name: &str, search_while_fetching: bool) -> Flow {
    let mut flow = Flow::start(test_name);
    flow.app
        .msg_tx
        .send_action(MediaTrackerSyncAction::SpawnTask);
    flow.run_until("music directory is synchronized", |app| {
        matches!(
            app.mdl.mode,
            Some(ModelMode::MusicDirSync {
                final_outcome: Some(Ok(_)),
                ..
            })
        )
    });
    flow.app.msg_tx.send_action(MediaTrackerSyncAction::Finish);
    if search_while_fetching {
        flow.run_until("track search is available", |app| {
            matches!(app.mdl.mode, Some(ModelMode::TrackSearch(_)))
        });
        flow.app
            .msg_tx
            .send_action(TrackSearchAction::Search(String::new()));
    }
    flow.run_until("track is listed", |app| {
        matches!(
            &app.mdl.mode,
            Some(ModelMode::TrackSearch(TrackSearchMode { track_list: Some(track_list), .. }))
                if !track_list.is_empty()
        )
    });
    flow
}

#[test]
fn synchronizing_and_listing_tracks_finishes() {
    drop(run_ui_paths(
        "synchronizing_and_listing_tracks_finishes",
        false,
    ));
}

#[test]
fn searching_while_fetching_still_lists_tracks() {
    drop(run_ui_paths(
        "searching_while_fetching_still_lists_tracks",
        true,
    ));
}

#[test]
fn outdated_track_search_items_are_discarded_and_fetched_again() {
    let mut flow = run_ui_paths(
        "outdated_track_search_items_are_discarded_and_fetched_again",
        false,
    );
    let (memo_delta, _) = flow
        .app
        .mdl
        .library
        .read_current_state()
        .track_search
        .update_memo_delta(&Memo::default());
    let Some(ModelMode::TrackSearch(TrackSearchMode {
        memo_state,
        track_list,
        ..
    })) = &mut flow.app.mdl.mode
    else {
        unreachable!();
    };
    *track_list = None;
    // The memo delta expects fetched entities that are not available anymore.
    *memo_state = MemoState::Pending {
        memo: Memo::default(),
        memo_delta,
        state_changed_again: false,
        pending_since: Instant::now(),
    };
    flow.app
        .msg_tx
        .send_action(TrackSearchAction::ApplyPendingStateChange {
            fetched_items: TrackSearchFetchedItems::Reset,
        });
    flow.run_until("track is listed again", |app| {
        matches!(
            &app.mdl.mode,
            Some(ModelMode::TrackSearch(TrackSearchMode { track_list: Some(track_list), .. }))
                if !track_list.is_empty()
        )
    });
}

fn is_revert_finished(app: &App) -> bool {
    matches!(
        app.mdl.mode,
        Some(ModelMode::RevertUnsynchronizedTracks {
            final_outcome: Some(_),
            ..
        })
    ) && !app.mdl.modified_track_count.is_counting
}

impl Flow {
    fn load_genres(&self) -> String {
        let entity = self
            .rt
            .block_on(aoide::backend_embedded::track::load_one(
                self.app.mdl.library.env().db_gatekeeper(),
                self.entity_uid.clone(),
            ))
            .unwrap();
        crate::library::ui::TrackEditDraft::from_track(&entity.body.track).genres
    }

    fn open_close_dialog(&mut self) {
        self.request_close();
        self.run_until("close dialog is open", |app| {
            app.mdl.close_state == CloseState::Requested
                && !app.mdl.modified_track_count.is_counting
        });
    }

    fn request_revert(&mut self) {
        self.app
            .msg_tx
            .send_action(PendingEditsAction::RequestRevert);
        self.run_until("confirmation is requested", |app| {
            app.mdl.is_revert_confirmation_open
        });
    }
}

#[test]
fn reverting_on_demand_requires_confirmation() {
    let mut flow = Flow::new("reverting_on_demand_requires_confirmation");
    flow.apply_genres("Techno\nAmbient");

    flow.request_revert();
    for _ in 0..10 {
        flow.run_frame(false);
    }
    assert_eq!("Techno\nAmbient", flow.load_genres());

    flow.app
        .msg_tx
        .send_action(PendingEditsAction::CancelRevert);
    flow.run_until("confirmation is cancelled", |app| {
        !app.mdl.is_revert_confirmation_open
    });
    for _ in 0..10 {
        flow.run_frame(false);
    }
    assert_eq!("Techno\nAmbient", flow.load_genres());
    assert_eq!(Some(1), flow.pending_edits().modified_track_count);
}

#[test]
fn reverting_on_demand_restores_the_track_without_writing_the_file() {
    let mut flow = Flow::new("reverting_on_demand_restores_the_track_without_writing_the_file");
    let unmodified_file = fs::read(flow.dir.track_file()).unwrap();
    flow.apply_genres("Techno\nAmbient");

    flow.request_revert();
    flow.app
        .msg_tx
        .send_action(PendingEditsAction::ConfirmRevert);
    let commands = flow.run_until("reverting is finished", is_revert_finished);

    assert!(!commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::NotRequested, flow.app.mdl.close_state);
    assert_eq!("", flow.load_genres());
    assert_eq!(Some(0), flow.app.mdl.modified_track_count.count);
    assert_eq!(unmodified_file, fs::read(flow.dir.track_file()).unwrap());
    assert!(!flow.pending_edits().could_revert());

    // Returns to the track search.
    flow.app
        .msg_tx
        .send_action(RevertUnsynchronizedTracksAction::Finish);
    flow.run_until("track search is available again", |app| {
        matches!(app.mdl.mode, Some(ModelMode::TrackSearch(_)))
    });
}

#[test]
fn reverting_before_closing_restores_the_track_and_closes() {
    let mut flow = Flow::new("reverting_before_closing_restores_the_track_and_closes");
    let unmodified_file = fs::read(flow.dir.track_file()).unwrap();
    flow.apply_genres("Techno\nAmbient");
    flow.open_close_dialog();

    flow.request_revert();
    // Nothing happens until confirmed.
    assert!(!flow.run_frame(false).contains(&ViewportCommand::Close));
    flow.app
        .msg_tx
        .send_action(PendingEditsAction::ConfirmRevert);
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);

    assert!(commands.contains(&ViewportCommand::Close));
    assert_eq!("", flow.load_genres());
    assert_eq!(unmodified_file, fs::read(flow.dir.track_file()).unwrap());
}

#[test]
fn cancelling_the_revert_confirmation_keeps_the_close_dialog_open() {
    let mut flow = Flow::new("cancelling_the_revert_confirmation_keeps_the_close_dialog_open");
    flow.apply_genres("Techno\nAmbient");
    flow.open_close_dialog();
    flow.request_revert();
    flow.app
        .msg_tx
        .send_action(PendingEditsAction::CancelRevert);
    flow.run_until("confirmation is cancelled", |app| {
        !app.mdl.is_revert_confirmation_open
    });
    assert_eq!(CloseState::Requested, flow.app.mdl.close_state);
    assert_eq!("Techno\nAmbient", flow.load_genres());
}

#[test]
fn reverting_discards_the_draft_of_the_track_editor() {
    let mut flow = Flow::new("reverting_discards_the_draft_of_the_track_editor");
    flow.open_track_editor_with_title("Draft title");
    assert!(flow.pending_edits().could_revert());
    flow.request_revert();
    flow.app
        .msg_tx
        .send_action(PendingEditsAction::ConfirmRevert);
    flow.run_until("reverting is finished", is_revert_finished);
    assert!(!flow.pending_edits().is_draft_modified());
    assert!(!flow.file_contains("Draft title"));
}

#[test]
fn stays_open_if_a_track_could_not_be_reverted() {
    let mut flow = Flow::new("stays_open_if_a_track_could_not_be_reverted");
    flow.apply_genres("Techno\nAmbient");
    fs::remove_file(flow.dir.track_file()).unwrap();
    flow.open_close_dialog();

    flow.request_revert();
    flow.app
        .msg_tx
        .send_action(PendingEditsAction::ConfirmRevert);
    let commands = flow.run_until("reverting is finished", is_revert_finished);

    assert!(!commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::Requested, flow.app.mdl.close_state);
    assert_eq!(Some(1), flow.app.mdl.modified_track_count.count);
    // The edits have not been lost.
    assert_eq!("Techno\nAmbient", flow.load_genres());

    flow.app
        .msg_tx
        .send_action(PendingEditsAction::CloseWithoutWriting);
    let commands = flow.run_until("closing is confirmed", is_close_confirmed);
    assert!(commands.contains(&ViewportCommand::Close));
}
