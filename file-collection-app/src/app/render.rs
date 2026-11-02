// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use eframe::Frame;
use egui::{
    Align, Button, CentralPanel, Grid, Layout, OpenUrl, Panel, ScrollArea, TextEdit, Ui,
    load::SizedTexture, scroll_area::ScrollSource,
};

use crate::library::{
    export_unsynchronized_tracks, track_search,
    ui::{ARTWORK_THUMBNAIL_IMAGE_SIZE, TrackListItem},
};

use super::{
    Action, MessageSender, Model, ModelMode, MusicDirSelection, MusicDirectoryAction,
    TrackEditAction, TrackEditor, TrackSearchAction, TrackSearchEdit, TrackSearchMode,
    TrackSelection, TrackSelectionAction, UiData,
    message::{
        ExportUnsynchronizedTracksAction, MediaTrackerDirListAction, MediaTrackerSyncAction,
    },
    track_editor, track_selection_editor,
};

const TRACK_EDITOR_PANEL_DEFAULT_WIDTH: f32 = 420.0;

const UNSYNCHRONIZED_MARKER: &str = "*";

// In contrast to `AppUpdateContext` the model is immutable during rendering.
// Only the `UiDataState` remains mutable.
pub(super) struct RenderContext<'a> {
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a Model,
    pub(super) ui_data: &'a mut UiData,
}

impl RenderContext<'_> {
    pub(super) fn render_frame(&mut self, ui: &mut Ui, _frm: &mut Frame) {
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
        let track_editor = edit.and_then(TrackSearchEdit::track_editor);
        track_editor::sync_draft(track_editor, &mut ui_data.editor_draft);
        let is_track_editor_draft_modified = track_editor
            .zip(ui_data.editor_draft.as_ref())
            .is_some_and(|(track_editor, draft)| draft.is_modified(track_editor));

        Panel::top("top-panel").show(ui, |ui| {
            render_top_panel(
                ui,
                ui_data,
                msg_tx,
                mdl,
                &library,
                is_track_editor_draft_modified,
            );
        });

        Panel::bottom("bottom-panel").show(ui, |ui| {
            render_bottom_panel(ui, msg_tx, mdl.mode.as_ref(), &library);
        });

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
                        );
                    });
            }
            None => (),
        }

        // CentralPanel must be added after all other panels!
        if let Some(mdl_mode) = &mdl.mode {
            CentralPanel::default().show(ui, |ui| {
                render_central_panel(
                    ui,
                    msg_tx,
                    mdl_mode,
                    &library,
                    is_track_editor_draft_modified,
                );
            });
        }
    }
}

#[expect(clippy::too_many_lines)] // TODO
fn render_top_panel(
    ui: &mut Ui,
    ui_data: &mut UiData,
    msg_tx: &MessageSender,
    mdl: &Model,
    library: &crate::library::CurrentState<'_>,
    is_track_editor_draft_modified: bool,
) {
    let Model {
        music_dir_selection,
        ..
    } = mdl;
    Grid::new("grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            let music_dir = library.settings.music_dir();
            ui.label("Music directory:");
            ui.label(
                music_dir
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
            );
            ui.end_row();

            ui.label("");
            Grid::new("grid")
                .num_columns(3)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                if ui
                    .add_enabled(
                        !matches!(music_dir_selection, Some(MusicDirSelection::Selecting)),
                        Button::new("Select music directory..."),
                    )
                    .on_hover_text("Switch collections or create a new one.")
                    .clicked()
                {
                    msg_tx
                        .send_action(MusicDirectoryAction::Select);
                }
                if ui
                    .add_enabled(
                        !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting)) && library.could_synchronize_music_dir_task(),
                        Button::new("Synchronize music directory"),
                    )
                    .on_hover_text(
                        "Rescan the music directory for added/modified/deleted files and update the collection.",
                    )
                    .clicked()
                {
                    msg_tx.send_action(MediaTrackerSyncAction::SpawnTask);
                }
                if ui
                    .add_enabled(
                        !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting)) && library.could_view_music_dir_list(),
                        Button::new("View music directory list"),
                    )
                    .clicked()
                {
                    msg_tx.send_action(MediaTrackerDirListAction::OpenView);
                }
                if ui
                    .add_enabled(
                        !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting))
                            // Unsaved changes would be lost when switching the mode.
                            && !is_track_editor_draft_modified
                            && library.could_export_unsynchronized_tracks(),
                        Button::new("Write unsynchronized files"),
                    )
                    .on_hover_text(
                        "Write the metadata of all modified tracks into their files. Files that have been modified on disk are skipped.",
                    )
                    .clicked()
                {
                    msg_tx.send_action(ExportUnsynchronizedTracksAction::SpawnTask);
                }
                if ui
                    .add_enabled(
                        !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting))
                            && library.could_reset_music_dir(),
                        Button::new("Reset music directory"),
                    )
                    .on_hover_text("Disconnect from the corresponding collection.")
                    .clicked()
                {
                    msg_tx
                        .send_action(MusicDirectoryAction::Reset);
                }
                ui.end_row();
            });
            ui.end_row();

            let collection_uid = library
                .collection
                .entity_brief()
                .map(|(entity_uid, _)| entity_uid);
            ui.label("Collection UID:");
            ui.label(
                collection_uid
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            );
            ui.end_row();

            let collection_title = library
                .collection
                .entity_brief()
                .and_then(|(_, collection)| {
                    collection.map(|collection| collection.title.as_str())
                });
            ui.label("Collection title:");
            ui.label(collection_title.unwrap_or_default());
            ui.end_row();

            let collection_summary = library
                .collection
                .entity_with_summary()
                .map(|(_, summary)| summary);
            ui.label("Collection summary:");
            ui.label(collection_summary.map_or(String::new(), |summary| {
                format!(
                    "#tracks = {num_tracks}, #playlists = {num_playlists}",
                    num_tracks = summary.tracks.total_count,
                    num_playlists = summary.playlists.total_count
                )
            }));
            ui.end_row();

            ui.label("Search tracks:");
            if ui
                .add_enabled(
                    matches!(mdl.mode, Some(ModelMode::TrackSearch { .. }))
                    && library.could_search_tracks(),
                    TextEdit::singleline(&mut ui_data.track_search_input),
                )
                .lost_focus()
            {
                msg_tx.send_action(TrackSearchAction::Search(ui_data.track_search_input.clone()));
            }
            ui.end_row();
        });
}

fn render_central_panel(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    mode: &ModelMode,
    library: &crate::library::CurrentState<'_>,
    is_track_editor_draft_modified: bool,
) {
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
        }) => {
            let edited_entity_uid = edit
                .as_ref()
                .and_then(TrackSearchEdit::track_editor)
                .map(|TrackEditor { entity_uid, .. }| entity_uid);
            let track_selection = edit.as_ref().and_then(TrackSearchEdit::track_selection);
            let text_style = egui::TextStyle::Body;
            let row_height = ui
                .text_style_height(&text_style)
                .max(ARTWORK_THUMBNAIL_IMAGE_SIZE as _);
            let total_rows = track_list.len();
            ScrollArea::both().show_rows(ui, row_height, total_rows, |ui, row_range| {
                if row_range.end == total_rows
                    // Prevent eagerly fetching more results repeatedly.
                    && Some(total_rows) == library.track_search.fetched_entities_len()
                    && matches!(memo_state, track_search::MemoState::Ready(_))
                    && library.could_fetch_more_track_search_results()
                {
                    log::debug!("Trying to fetch more track search results");
                    msg_tx.send_action(TrackSearchAction::FetchMore);
                }
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    for item in &track_list[row_range] {
                        let interaction = TrackListItemInteraction {
                            is_edited: edited_entity_uid == Some(&item.entity_uid),
                            is_selected: track_selection
                                .is_some_and(|selection| selection.contains(&item.entity_uid)),
                            is_track_editor_draft_modified,
                            is_track_selection_updating: track_selection
                                .is_some_and(TrackSelection::is_updating),
                        };
                        render_track_list_item(ui, msg_tx, item, row_height, interaction);
                    }
                })
            });
        }
        ModelMode::MusicDirSync {
            last_progress,
            final_outcome,
        } => {
            ScrollArea::both()
                .scroll_source(ScrollSource::ALL)
                .show(ui, |ui| {
                    if let Some(final_outcome) = final_outcome {
                        let line = format!("{final_outcome:#?}");
                        ui.label(line);
                    } else if let Some(progress) = last_progress {
                        let line = format!("{progress:#?}");
                        ui.label(line);
                    }
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

#[derive(Debug, Clone, Copy)]
#[expect(clippy::struct_excessive_bools)]
struct TrackListItemInteraction {
    is_edited: bool,
    is_selected: bool,
    is_track_editor_draft_modified: bool,
    is_track_selection_updating: bool,
}

fn render_track_list_item(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    item: &TrackListItem,
    row_height: f32,
    interaction: TrackListItemInteraction,
) {
    let TrackListItemInteraction {
        is_edited,
        is_selected,
        is_track_editor_draft_modified,
        is_track_selection_updating,
    } = interaction;
    debug_assert_eq!(
        item.artwork_thumbnail_texture.size_vec2().x,
        item.artwork_thumbnail_texture.size_vec2().y
    );
    let artwork_texture = SizedTexture {
        id: item.artwork_thumbnail_texture.id(),
        size: egui::Vec2::new(row_height, row_height),
    };
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        let artwork_button = Button::image(artwork_texture).frame(false);
        let mut artwork_response = ui.add(artwork_button);
        if let Some(content_url) = &item.content_url {
            let file_location = content_url.to_file_path().map_or_else(
                |()| content_url.to_string(),
                |path| path.display().to_string(),
            );
            artwork_response = artwork_response.on_hover_text_at_pointer(file_location);
            // Demo interaction handler that simply opens the content URL in a new (browser) tab.
            if artwork_response.clicked() || artwork_response.middle_clicked() {
                ui.open_url(OpenUrl {
                    url: content_url.to_string(),
                    new_tab: true,
                });
            }
        }
        if !item.is_synchronized {
            ui.colored_label(ui.visuals().warn_fg_color, UNSYNCHRONIZED_MARKER)
                .on_hover_text("Modified, but not written into the file yet.");
        }
        let label = track_list_item_label(item);
        let label_response = ui.selectable_label(is_edited || is_selected, label);
        if is_track_editor_draft_modified {
            // Prevent losing unsaved changes unintentionally.
            if !is_edited {
                label_response
                    .on_hover_text("Save or revert the changes of the edited track first.");
            }
            return;
        }
        if is_track_selection_updating {
            return;
        }
        let label_response = label_response
            .on_hover_text("Click to edit the track. Ctrl+click to select multiple tracks.");
        if !label_response.clicked() {
            return;
        }
        if ui.input(|input| input.modifiers.command) {
            msg_tx.send_action(TrackSelectionAction::Toggle(item.entity_uid.clone()));
        } else if !is_edited {
            msg_tx.send_action(TrackEditAction::Open(item.entity_uid.clone()));
        }
    });
}

fn render_export_unsynchronized_tracks(
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

fn render_bottom_panel(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    mode: Option<&ModelMode>,
    library: &crate::library::CurrentState<'_>,
) {
    Grid::new("grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            if let Some(mode) = mode {
                let text;
                let hover_text;
                let enabled;
                let action: Action;
                match mode {
                    ModelMode::TrackSearch(_) => {
                        text = "Fetch more";
                        hover_text = "Fetch the next page of search results.";
                        enabled = library.could_fetch_more_track_search_results();
                        action = TrackSearchAction::FetchMore.into();
                    }
                    ModelMode::MusicDirSync { .. } => {
                        if library.could_abort_synchronize_music_dir_task() {
                            text = "Abort";
                            hover_text = "Stop the current synchronization task.";
                            enabled = true;
                            action = MediaTrackerSyncAction::AbortPendingTask.into();
                        } else {
                            text = "Dismiss";
                            hover_text = "Clear output and return to track search.";
                            enabled = true;
                            action = MediaTrackerSyncAction::Finish.into();
                        }
                    }
                    ModelMode::ExportUnsynchronizedTracks { .. } => {
                        if library.could_abort_export_unsynchronized_tracks() {
                            text = "Abort";
                            hover_text = "Stop writing files after the current file.";
                            enabled = true;
                            action = ExportUnsynchronizedTracksAction::AbortPendingTask.into();
                        } else {
                            text = "Dismiss";
                            hover_text = "Clear output and return to track search.";
                            enabled = true;
                            action = ExportUnsynchronizedTracksAction::Finish.into();
                        }
                    }
                    ModelMode::MusicDirList { .. } => {
                        text = "Dismiss";
                        hover_text = "Clear output and return to track search.";
                        enabled = true;
                        action = MediaTrackerDirListAction::CloseView.into();
                    }
                }
                if ui
                    .add_enabled(enabled, Button::new(text))
                    .on_hover_text(hover_text)
                    .clicked()
                {
                    msg_tx.send_action(action);
                }
                ui.end_row();
            }

            ui.label("Last error:");
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
            if let Some(last_error) = last_error.as_deref() {
                ui.label(last_error);
            }
            ui.end_row();
        });
}

#[must_use]
pub(super) fn track_list_item_label(track: &TrackListItem) -> String {
    let track_title = track.title.as_deref().unwrap_or("Untitled");
    let track_artist = &track.artist;
    let album_title = &track.album_title;
    let album_artist = &track.album_artist;
    let bpm = track.bpm.and_then(|bpm| {
        let value = bpm.value().round();
        if value > 0.0 && value < f64::from(u16::MAX) {
            Some(value as u16)
        } else {
            None
        }
    });
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
    let key = track
        .key
        .map(|key| key.code().as_lancelot_str())
        .unwrap_or_default();
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
