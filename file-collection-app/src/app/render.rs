// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Write as _;

use egui::{
    Align, Button, CentralPanel, Grid, Label, Layout, MenuBar, Panel, RichText, ScrollArea,
    TextEdit, Ui, scroll_area::ScrollSource,
};

use aoide::backend_embedded::batch::synchronize_collection_vfs;

use crate::library::{
    export_unsynchronized_tracks, revert_unsynchronized_tracks, ui::TrackListItem,
};

use super::{
    Action, CloseState, EditorSettingsAction, MessageSender, Model, ModelMode, MusicDirSelection,
    MusicDirectoryAction, TrackEditor, TrackSearchAction, TrackSearchEdit, TrackSearchMode, UiData,
    message::{
        ExportUnsynchronizedTracksAction, MediaTrackerDirListAction, MediaTrackerSyncAction,
        PendingEditsAction, RevertUnsynchronizedTracksAction,
    },
    pending_edits::{self, DraftEdit, PendingEdits},
    track_editor, track_list, track_selection_editor,
};

const TRACK_EDITOR_PANEL_DEFAULT_WIDTH: f32 = 420.0;

// In contrast to `AppUpdateContext` the model is immutable during rendering.
// Only the `UiDataState` remains mutable.
pub(super) struct RenderContext<'a> {
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a Model,
    pub(super) ui_data: &'a mut UiData,
}

impl RenderContext<'_> {
    pub(super) fn render_frame(&mut self, ui: &mut Ui, pending_edits: &PendingEdits) {
        let Self {
            msg_tx,
            mdl,
            ui_data,
        } = self;

        let library = mdl.library.read_current_state();

        let (track_list, edit) = match &mdl.mode {
            Some(ModelMode::TrackSearch(TrackSearchMode {
                track_list, edit, ..
            })) => (track_list.as_deref().unwrap_or_default(), edit.as_ref()),
            _ => (&[][..], None),
        };
        let is_track_editor_draft_modified = pending_edits.is_draft_modified();

        Panel::top("menu-bar").show(ui, |ui| {
            render_menu_bar(ui, msg_tx, mdl, &library);
        });

        Panel::top("top-panel").show(ui, |ui| {
            render_top_panel(ui, ui_data, msg_tx, mdl, &library, pending_edits);
        });

        Panel::bottom("status-bar").show(ui, |ui| {
            render_status_bar(ui, &library);
        });

        // Changed by the editor panels and persisted by an action.
        let mut write_files = mdl.editor_settings.write_files;
        match edit {
            Some(TrackSearchEdit::Multiple(track_selection)) => {
                Panel::right("track-selection-editor-panel")
                    .resizable(true)
                    .default_size(TRACK_EDITOR_PANEL_DEFAULT_WIDTH)
                    .show(ui, |ui| {
                        track_selection_editor::render_track_selection_editor_panel(
                            ui,
                            msg_tx,
                            track_selection,
                            track_list,
                            &mut ui_data.selection_draft,
                            &mut write_files,
                        );
                    });
            }
            Some(TrackSearchEdit::Single(track_editor)) => {
                Panel::right("track-editor-panel")
                    .resizable(true)
                    .default_size(TRACK_EDITOR_PANEL_DEFAULT_WIDTH)
                    .show(ui, |ui| {
                        track_editor::render_track_editor_panel(
                            ui,
                            msg_tx,
                            track_editor,
                            ui_data.editor_draft.as_mut(),
                            &mut write_files,
                        );
                    });
            }
            None => (),
        }
        if write_files != mdl.editor_settings.write_files {
            msg_tx.send_action(EditorSettingsAction::SetWriteFiles(write_files));
        }

        // CentralPanel must be added after all other panels!
        if let Some(mdl_mode) = &mdl.mode {
            CentralPanel::default().show(ui, |ui| {
                render_central_panel(
                    ui,
                    msg_tx,
                    mdl,
                    mdl_mode,
                    &mut ui_data.track_list_table,
                    &library,
                    is_track_editor_draft_modified,
                );
            });
        }

        // Rendered after all panels to block them.
        if mdl.is_revert_confirmation_open {
            pending_edits::render_revert_dialog(ui.ctx(), msg_tx, pending_edits);
        } else if mdl.close_state == CloseState::Requested {
            pending_edits::render_close_dialog(ui.ctx(), msg_tx, pending_edits, mdl.mode.as_ref());
        }
    }
}

/// Menus for the infrequent actions that affect the whole collection.
fn render_menu_bar(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    mdl: &Model,
    library: &crate::library::CurrentState<'_>,
) {
    let is_selecting = matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting));
    MenuBar::new().ui(ui, |ui| {
        ui.menu_button("Collection", |ui| {
            if ui
                .add_enabled(!is_selecting, Button::new("Select music directory…"))
                .on_hover_text("Switch collections or create a new one.")
                .clicked()
            {
                msg_tx.send_action(MusicDirectoryAction::Select);
            }
            if ui
                .add_enabled(
                    !is_selecting && library.could_synchronize_music_dir_task(),
                    Button::new("Synchronize music directory"),
                )
                .on_hover_text(
                    "Rescan the music directory for added/modified/deleted files and update the collection.",
                )
                .clicked()
            {
                msg_tx.send_action(MediaTrackerSyncAction::SpawnTask);
            }
            ui.separator();
            if ui
                .add_enabled(
                    !is_selecting && library.could_reset_music_dir(),
                    Button::new("Reset music directory"),
                )
                .on_hover_text("Disconnect from the corresponding collection.")
                .clicked()
            {
                msg_tx.send_action(MusicDirectoryAction::Reset);
            }
        });
        ui.menu_button("View", |ui| {
            if ui
                .add_enabled(
                    !is_selecting && library.could_view_music_dir_list(),
                    Button::new("Music directory list"),
                )
                .on_hover_text("List all folders of the music directory with their track count.")
                .clicked()
            {
                msg_tx.send_action(MediaTrackerDirListAction::OpenView);
            }
        });
    });
}

fn render_top_panel(
    ui: &mut Ui,
    ui_data: &mut UiData,
    msg_tx: &MessageSender,
    mdl: &Model,
    library: &crate::library::CurrentState<'_>,
    pending_edits: &PendingEdits,
) {
    ui.horizontal(|ui| {
        // Right-to-left: The pending edits are placed at the right edge
        // and the search field fills the remaining space.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let is_selecting =
                matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting));
            render_pending_edits(ui, msg_tx, pending_edits, is_selecting);
            let search_enabled = matches!(mdl.mode, Some(ModelMode::TrackSearch { .. }))
                && library.could_search_tracks();
            let search_input = TextEdit::singleline(&mut ui_data.track_search_input)
                .hint_text("🔍 Search tracks…")
                .desired_width(ui.available_width());
            if ui
                .add_enabled(search_enabled, search_input)
                .on_hover_text("Press Enter to search.")
                .lost_focus()
            {
                msg_tx.send_action(TrackSearchAction::Search(
                    ui_data.track_search_input.clone(),
                ));
            }
        });
    });
}

/// Save button with a drop-down for reverting, only visible if needed.
///
/// Must be rendered into a right-to-left layout.
fn render_pending_edits(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    pending_edits: &PendingEdits,
    is_selecting: bool,
) {
    if pending_edits.is_writing {
        ui.label("Writing files…");
        ui.spinner();
        return;
    }
    let could_revert = pending_edits.could_revert();
    if !could_revert && !pending_edits.could_save() {
        // Nothing pending.
        return;
    }
    ui.add_enabled_ui(!is_selecting, |ui| {
        ui.menu_button("⏷", |ui| {
            if ui
                .add_enabled(
                    could_revert,
                    Button::new(format!("{}…", pending_edits.revert_button_text())),
                )
                .on_hover_text(
                    "Discard the changes of the track editor and restore all modified tracks from their files. Asks for confirmation.",
                )
                .clicked()
            {
                msg_tx.send_action(PendingEditsAction::RequestRevert);
            }
        })
        .response
        .on_hover_text("More actions for the pending edits.");
        let mut save_button = ui
            .add_enabled(
                pending_edits.could_save(),
                Button::new(pending_edits.save_button_text()),
            )
            .on_hover_text(
                "Save the changes of the track editor and write the metadata of all modified tracks into their files. Files that have been modified on disk are skipped.",
            );
        if matches!(pending_edits.draft, DraftEdit::Invalid) {
            save_button = save_button
                .on_disabled_hover_text("Fix the invalid values in the track editor first.");
        }
        if save_button.clicked() {
            msg_tx.send_action(pending_edits.save_action());
        }
        ui.colored_label(ui.visuals().warn_fg_color, "●")
            .on_hover_text("There are unsaved changes.");
    });
}

fn render_central_panel(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    mdl: &Model,
    mode: &ModelMode,
    track_list_table: &mut track_list::TrackListTableState,
    library: &crate::library::CurrentState<'_>,
    is_track_editor_draft_modified: bool,
) {
    if !matches!(mode, ModelMode::TrackSearch(_)) {
        render_task_header(ui, msg_tx, mode, library);
        ui.separator();
    }
    match mode {
        ModelMode::TrackSearch(TrackSearchMode {
            track_list: None, ..
        }) => {
            if library.collection.is_ready() {
                // The track list should become available soon.
                ui.label("...loading...");
            }
        }
        ModelMode::TrackSearch(TrackSearchMode {
            track_list: Some(track_list),
            memo_state,
            edit,
            track_list_edit_error,
        }) => {
            let interaction = track_list::TrackListInteraction {
                edited_entity_uid: edit
                    .as_ref()
                    .and_then(TrackSearchEdit::track_editor)
                    .map(|TrackEditor { entity_uid, .. }| entity_uid),
                track_selection: edit.as_ref().and_then(TrackSearchEdit::track_selection),
                is_track_editor_draft_modified,
                edit_error: track_list_edit_error.as_deref(),
                tag_cloud: mdl.tag_cloud.as_ref(),
            };
            track_list::render_track_list(
                ui,
                msg_tx,
                &mdl.track_list_columns,
                track_list_table,
                track_list,
                track_list::could_fetch_more(library, memo_state, track_list.len()),
                interaction,
            );
        }
        ModelMode::MusicDirSync {
            last_progress,
            final_outcome,
        } => {
            ScrollArea::both()
                .scroll_source(ScrollSource::ALL)
                .show(ui, |ui| {
                    render_music_dir_sync(ui, last_progress.as_ref(), final_outcome.as_ref());
                });
        }
        ModelMode::ExportUnsynchronizedTracks {
            last_progress,
            final_outcome,
        } => {
            ScrollArea::both()
                .scroll_source(ScrollSource::ALL)
                .show(ui, |ui| {
                    render_export_unsynchronized_tracks(
                        ui,
                        last_progress.as_ref(),
                        final_outcome.as_ref(),
                    );
                });
        }
        ModelMode::RevertUnsynchronizedTracks {
            last_progress,
            final_outcome,
        } => {
            ScrollArea::both()
                .scroll_source(ScrollSource::ALL)
                .show(ui, |ui| {
                    render_revert_unsynchronized_tracks(
                        ui,
                        last_progress.as_ref(),
                        final_outcome.as_ref(),
                    );
                });
        }
        ModelMode::MusicDirList {
            content_paths_with_count,
        } => {
            ScrollArea::both()
                .scroll_source(ScrollSource::ALL)
                .show(ui, |ui| {
                    for (content_path, count) in content_paths_with_count {
                        // Display absolute paths. Otherwise the root folder would become an empty string.
                        ui.label(format!("/{content_path} ({count})"));
                    }
                });
        }
    }
}

/// Join the non-zero counts, e.g. "3 added, 1 modified".
#[must_use]
fn format_counts(counts: &[(usize, &str)]) -> String {
    let text = counts
        .iter()
        .filter(|(count, _)| *count > 0)
        .map(|(count, label)| format!("{count} {label}"))
        .collect::<Vec<_>>()
        .join(", ");
    if text.is_empty() {
        "none".to_owned()
    } else {
        text
    }
}

#[must_use]
fn music_dir_sync_progress_text(progress: &synchronize_collection_vfs::Progress) -> String {
    use synchronize_collection_vfs::Progress;
    const NUM_STEPS: usize = 8;
    let (step, text) = match progress {
        Progress::Step1UntrackExcludedDirectories => {
            (1, "Untracking excluded directories…".to_owned())
        }
        Progress::Step2ScanDirectories(event) => (
            2,
            format!(
                "Scanning directories: {directories} directories, {entries} ({elapsed} s)",
                directories = event.progress.directories.finished,
                entries = format_counts(&[
                    (event.progress.entries.finished, "files"),
                    (event.progress.entries.skipped, "skipped"),
                ]),
                elapsed = event.elapsed.as_secs(),
            ),
        ),
        Progress::Step3UntrackOrphanedDirectories => {
            (3, "Untracking orphaned directories…".to_owned())
        }
        Progress::Step4ImportFiles(event) => {
            let tracks = &event.summary.tracks;
            (
                4,
                format!(
                    "Importing files: {counts} ({elapsed} s)",
                    counts = format_counts(&[
                        (tracks.created, "created"),
                        (tracks.updated, "updated"),
                        (tracks.unchanged, "unchanged"),
                        (tracks.skipped, "skipped"),
                        (tracks.failed, "failed"),
                    ]),
                    elapsed = event.elapsed.as_secs(),
                ),
            )
        }
        Progress::Step5PurgeUntrackedMediaSources => {
            (5, "Purging untracked media sources…".to_owned())
        }
        Progress::Step6PurgeOrphanedMediaSources => {
            (6, "Purging orphaned media sources…".to_owned())
        }
        Progress::Step7FindUntrackedFiles(event) => (
            7,
            format!(
                "Finding untracked files: {files} files ({elapsed} s)",
                files = event.progress.entries.finished,
                elapsed = event.elapsed.as_secs(),
            ),
        ),
        Progress::Step8FindUnsynchronizedTracks => (8, "Finding unsynchronized tracks…".to_owned()),
    };
    format!("Step {step} of {NUM_STEPS}: {text}")
}

/// The counts of all synchronization steps.
fn render_music_dir_sync_summary(ui: &mut Ui, outcome: &synchronize_collection_vfs::Outcome) {
    let synchronize_collection_vfs::Outcome {
        completion: _,
        untrack_excluded_directories,
        scan_directories,
        untrack_orphaned_directories,
        import_files,
        purge_untracked_media_sources,
        purge_orphaned_media_sources,
        find_untracked_files,
        find_unsynchronized_tracks,
    } = outcome;
    Grid::new("music-dir-sync-outcome")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            let mut row = |label: &str, text: String| {
                ui.label(label);
                ui.label(text);
                ui.end_row();
            };
            if let Some(outcome) = scan_directories {
                let summary = &outcome.summary;
                row(
                    "Directories:",
                    format_counts(&[
                        (summary.current, "unchanged"),
                        (summary.added, "added"),
                        (summary.modified, "modified"),
                        (summary.orphaned, "orphaned"),
                        (summary.skipped, "skipped"),
                    ]),
                );
            }
            let untracked_directories = untrack_excluded_directories
                .as_ref()
                .map_or(0, |outcome| outcome.summary.untracked)
                + untrack_orphaned_directories
                    .as_ref()
                    .map_or(0, |outcome| outcome.summary.untracked);
            if untracked_directories > 0 {
                row("Untracked directories:", untracked_directories.to_string());
            }
            if let Some(outcome) = import_files {
                let tracks = &outcome.summary.tracks;
                row(
                    "Tracks:",
                    format_counts(&[
                        (tracks.created, "created"),
                        (tracks.updated, "updated"),
                        (tracks.unchanged, "unchanged"),
                        (tracks.skipped, "skipped"),
                        (tracks.failed, "failed"),
                        (tracks.not_imported, "not imported"),
                        (tracks.not_created, "not created"),
                        (tracks.not_updated, "not updated"),
                    ]),
                );
            }
            let purged_sources = purge_untracked_media_sources
                .as_ref()
                .map_or(0, |outcome| outcome.summary.purged)
                + purge_orphaned_media_sources
                    .as_ref()
                    .map_or(0, |outcome| outcome.summary.purged);
            if purged_sources > 0 {
                row("Purged tracks:", purged_sources.to_string());
            }
            if let Some(outcome) = find_untracked_files {
                row("Untracked files:", outcome.content_paths.len().to_string());
            }
            if let Some(tracks) = find_unsynchronized_tracks {
                row("Unsynchronized tracks:", tracks.len().to_string());
            }
        });
}

pub(super) fn render_music_dir_sync(
    ui: &mut Ui,
    last_progress: Option<&synchronize_collection_vfs::Progress>,
    final_outcome: Option<&Result<Box<synchronize_collection_vfs::Outcome>, String>>,
) {
    let Some(final_outcome) = final_outcome else {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                last_progress.map_or_else(|| "Starting…".to_owned(), music_dir_sync_progress_text),
            );
        });
        return;
    };
    let outcome = match final_outcome {
        Ok(outcome) => outcome.as_ref(),
        Err(err) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("Synchronization failed: {err}"),
            );
            return;
        }
    };
    if matches!(
        outcome.completion,
        synchronize_collection_vfs::Completion::Aborted
    ) {
        ui.label("Aborted.");
    }
    render_music_dir_sync_summary(ui, outcome);
    let synchronize_collection_vfs::Outcome {
        import_files,
        find_untracked_files,
        find_unsynchronized_tracks,
        ..
    } = outcome;
    if let Some(outcome) = import_files
        && !outcome.imported_sources_with_issues.is_empty()
    {
        ui.separator();
        ui.label("Imported with issues:");
        for source in &outcome.imported_sources_with_issues {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                format!(
                    "/{path}: {messages}",
                    path = source.path,
                    messages = source.messages.join("; ")
                ),
            );
        }
    }
    if let Some(outcome) = find_untracked_files
        && !outcome.content_paths.is_empty()
    {
        ui.separator();
        ui.label("Untracked files:");
        for content_path in &outcome.content_paths {
            ui.label(format!("/{content_path}"));
        }
    }
    if let Some(tracks) = find_unsynchronized_tracks
        && !tracks.is_empty()
    {
        ui.separator();
        ui.label("Unsynchronized tracks, the collection differs from their files:");
        for track in tracks {
            ui.label(format!("/{}", track.raw.body.content_link.path));
        }
    }
}

pub(super) fn render_export_unsynchronized_tracks(
    ui: &mut Ui,
    last_progress: Option<&export_unsynchronized_tracks::Progress>,
    final_outcome: Option<&Result<Box<export_unsynchronized_tracks::Outcome>, String>>,
) {
    let Some(final_outcome) = final_outcome else {
        ui.horizontal(|ui| {
            ui.spinner();
            if let Some(export_unsynchronized_tracks::Progress { total, finished }) = last_progress
            {
                ui.label(format!("Writing files: {finished} of {total}"));
            } else {
                ui.label("Finding unsynchronized tracks...");
            }
        });
        return;
    };
    let outcome = match final_outcome {
        Ok(outcome) => outcome,
        Err(err) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("Writing files failed: {err}"),
            );
            return;
        }
    };
    let export_unsynchronized_tracks::Outcome {
        completion,
        exported,
        skipped,
        failed,
    } = outcome.as_ref();
    if matches!(
        completion,
        export_unsynchronized_tracks::Completion::Aborted
    ) {
        ui.label("Aborted.");
    }
    ui.label(format!(
        "Written: {exported}, skipped: {skipped}, failed: {failed}",
        exported = exported.len(),
        skipped = skipped.len(),
        failed = failed.len(),
    ));
    if !skipped.is_empty() {
        ui.separator();
        ui.label("Skipped files that have been modified on disk or never been imported. Synchronize the music directory first:");
        for (_, content_path, _) in skipped {
            ui.label(format!("/{content_path}"));
        }
    }
    if !failed.is_empty() {
        ui.separator();
        ui.label("Failed:");
        for (_, content_path, error) in failed {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("/{content_path}: {error}"),
            );
        }
    }
}

pub(super) fn render_revert_unsynchronized_tracks(
    ui: &mut Ui,
    last_progress: Option<&revert_unsynchronized_tracks::Progress>,
    final_outcome: Option<&Result<Box<revert_unsynchronized_tracks::Outcome>, String>>,
) {
    let Some(final_outcome) = final_outcome else {
        ui.horizontal(|ui| {
            ui.spinner();
            if let Some(revert_unsynchronized_tracks::Progress { total, finished }) = last_progress
            {
                ui.label(format!("Reverting tracks: {finished} of {total}"));
            } else {
                ui.label("Finding modified tracks...");
            }
        });
        return;
    };
    let outcome = match final_outcome {
        Ok(outcome) => outcome,
        Err(err) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("Reverting tracks failed: {err}"),
            );
            return;
        }
    };
    let revert_unsynchronized_tracks::Outcome {
        completion,
        reverted,
        failed,
    } = outcome.as_ref();
    if matches!(
        completion,
        revert_unsynchronized_tracks::Completion::Aborted
    ) {
        ui.label("Aborted.");
    }
    ui.label(format!(
        "Reverted: {reverted}, failed: {failed}",
        reverted = reverted.len(),
        failed = failed.len(),
    ));
    if !failed.is_empty() {
        ui.separator();
        ui.label("Failed, these tracks remain modified:");
        for (_, content_path, error) in failed {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("/{content_path}: {error}"),
            );
        }
    }
}

/// Title of the current task view with a button for aborting or leaving it.
fn render_task_header(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    mode: &ModelMode,
    library: &crate::library::CurrentState<'_>,
) {
    const DISMISS_HOVER_TEXT: &str = "Clear output and return to track search.";
    let (title, text, hover_text, action): (_, _, _, Action) = match mode {
        ModelMode::TrackSearch(_) => {
            // Not a task.
            return;
        }
        ModelMode::MusicDirSync { .. } => {
            let title = "Synchronize music directory";
            if library.could_abort_synchronize_music_dir_task() {
                (
                    title,
                    "Abort",
                    "Stop the current synchronization task.",
                    MediaTrackerSyncAction::AbortPendingTask.into(),
                )
            } else {
                (
                    title,
                    "Dismiss",
                    DISMISS_HOVER_TEXT,
                    MediaTrackerSyncAction::Finish.into(),
                )
            }
        }
        ModelMode::ExportUnsynchronizedTracks { .. } => {
            let title = "Write files";
            if library.could_abort_export_unsynchronized_tracks() {
                (
                    title,
                    "Abort",
                    "Stop writing files after the current file.",
                    ExportUnsynchronizedTracksAction::AbortPendingTask.into(),
                )
            } else {
                (
                    title,
                    "Dismiss",
                    DISMISS_HOVER_TEXT,
                    ExportUnsynchronizedTracksAction::Finish.into(),
                )
            }
        }
        ModelMode::RevertUnsynchronizedTracks { .. } => {
            let title = "Revert tracks";
            if library.could_abort_revert_unsynchronized_tracks() {
                (
                    title,
                    "Abort",
                    "Stop reverting after the current track.",
                    RevertUnsynchronizedTracksAction::AbortPendingTask.into(),
                )
            } else {
                (
                    title,
                    "Dismiss",
                    DISMISS_HOVER_TEXT,
                    RevertUnsynchronizedTracksAction::Finish.into(),
                )
            }
        }
        ModelMode::MusicDirList { .. } => (
            "Music directory list",
            "Dismiss",
            DISMISS_HOVER_TEXT,
            MediaTrackerDirListAction::CloseView.into(),
        ),
    };
    ui.horizontal(|ui| {
        ui.heading(title);
        if ui.button(text).on_hover_text(hover_text).clicked() {
            msg_tx.send_action(action);
        }
    });
}

/// A single line with the collection and the last error, if any.
fn render_status_bar(ui: &mut Ui, library: &crate::library::CurrentState<'_>) {
    ui.horizontal(|ui| {
        if let Some(music_dir) = library.settings.music_dir() {
            let mut hover_text = format!("Music directory: {}", music_dir.display());
            let brief = library.collection.entity_brief();
            if let Some((entity_uid, _)) = brief {
                let _ = write!(hover_text, "\nCollection UID: {entity_uid}");
            }
            let title = brief.and_then(|(_, collection)| collection.map(|c| c.title.as_str()));
            let mut text = title.map_or_else(|| music_dir.display().to_string(), ToOwned::to_owned);
            if let Some((_, summary)) = library.collection.entity_with_summary() {
                let _ = write!(
                    text,
                    " · {num_tracks} tracks · {num_playlists} playlists",
                    num_tracks = summary.tracks.total_count,
                    num_playlists = summary.playlists.total_count,
                );
            }
            ui.label(text).on_hover_text(hover_text);
        } else {
            ui.weak("No music directory selected. Use Collection ▸ Select music directory…");
        }

        let last_error = library
            .collection
            .last_error()
            .map(ToOwned::to_owned)
            .or_else(|| {
                library
                    .track_search
                    .last_fetch_error()
                    .map(ToString::to_string)
            });
        if let Some(last_error) = last_error {
            ui.separator();
            ui.add(
                Label::new(
                    RichText::new(format!("⚠ {last_error}")).color(ui.visuals().error_fg_color),
                )
                .truncate(),
            )
            .on_hover_text(last_error);
        }
    });
}

/// A small icon that shows usage hints on hover.
pub(super) fn help_icon(ui: &mut Ui, hover_text: &str) -> egui::Response {
    ui.weak("(?)").on_hover_text(hover_text)
}

/// The tempo rounded to an integer value for display.
#[must_use]
pub(super) fn track_list_item_bpm(track: &TrackListItem) -> Option<u16> {
    track.bpm.and_then(|bpm| {
        let value = bpm.value().round();
        if value > 0.0 && value < f64::from(u16::MAX) {
            Some(value as u16)
        } else {
            None
        }
    })
}

#[must_use]
pub(super) fn track_list_item_label(track: &TrackListItem) -> String {
    let track_title = track.title.as_deref().unwrap_or("Untitled");
    let track_artist = &track.artist;
    let album_title = &track.album_title;
    let album_artist = &track.album_artist;
    let bpm = track_list_item_bpm(track);
    let label = match (track_artist, album_title, album_artist) {
        (Some(track_artist), Some(album_title), Some(album_artist)) => {
            if track_artist == album_artist {
                format!("{track_artist} - {track_title} [{album_title}]")
            } else {
                format!("{track_artist} - {track_title} [{album_title} by {album_artist}]")
            }
        }
        (None, Some(album_title), Some(album_artist)) => {
            format!("{track_title} [{album_title} by {album_artist}]")
        }
        (Some(track_artist), Some(album_title), None) => {
            format!("{track_artist} - {track_title} [{album_title}]")
        }
        (Some(track_artist), None, _) => {
            format!("{track_artist} - {track_title}")
        }
        (None, Some(album_title), None) => {
            format!("{track_title} [{album_title}]")
        }
        (None, None, _) => track_title.to_string(),
    };
    let key = track.key.map_or_default(|key| key.code().as_lancelot_str());
    if let Some(bpm) = bpm {
        if key.is_empty() {
            format!("{label} {{{bpm}}}")
        } else {
            format!("{label} {{{bpm} {key}}}")
        }
    } else if key.is_empty() {
        label
    } else {
        format!("{label} {{{key}}}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_counts;

    #[test]
    fn format_counts_skips_zeros() {
        assert_eq!(
            "3 added, 1 failed",
            format_counts(&[(3, "added"), (0, "modified"), (1, "failed")])
        );
        assert_eq!("none", format_counts(&[(0, "added")]));
        assert_eq!("none", format_counts(&[]));
    }
}
