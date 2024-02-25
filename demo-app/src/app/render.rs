// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use eframe::Frame;
use egui::{
    load::SizedTexture, Align, Button, CentralPanel, Context, Grid, Layout, ScrollArea, TextEdit,
    TopBottomPanel,
};

use super::{
    Action, CollectionAction, MessageSender, Model, ModelMode, MusicDirSelection,
    MusicDirectoryAction, TrackListItem, TrackSearchAction, UiData, ARTWORK_THUMBNAIL_IMAGE_SIZE,
};

// In contrast to `AppUpdateContext` the model is immutable during rendering.
// Only the `UiDataState` remains mutable.
pub(super) struct RenderContext<'a> {
    pub(super) msg_tx: &'a MessageSender,
    pub(super) mdl: &'a Model,
    pub(super) ui_data: &'a mut UiData,
}

impl<'a> RenderContext<'a> {
    #[allow(clippy::float_cmp)] // Texture size (x/y) comparison.
    #[allow(clippy::too_many_lines)] // TODO
    pub(super) fn render_frame(&mut self, ctx: &Context, _frm: &mut Frame) {
        let Self {
            msg_tx,
            mdl,
            ui_data,
        } = self;
        let current_library_state = mdl.library.read_lock_current_state();

        TopBottomPanel::top("top-panel").show(ctx, |ui| {
        Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                let music_dir = current_library_state.music_dir();
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
                            !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting)),
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
                            !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting)) && current_library_state.could_synchronize_music_dir_task(),
                            Button::new("Synchronize music directory"),
                        )
                        .on_hover_text(
                            "Rescan the music directory for added/modified/deleted files and update the collection.",
                        )
                        .clicked()
                    {
                        msg_tx.send_action(MusicDirectoryAction::SpawnSyncTask);
                    }
                    if ui
                        .add_enabled(
                            !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting)) && current_library_state.could_view_music_dir_list(),
                            Button::new("View music directory list"),
                        )
                        .clicked()
                    {
                        msg_tx.send_action(MusicDirectoryAction::ViewList);
                    }
                    if ui
                        .add_enabled(
                            !matches!(mdl.music_dir_selection, Some(MusicDirSelection::Selecting))
                                && current_library_state.could_reset_music_dir(),
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

                let collection_uid = current_library_state
                    .collection()
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

                let collection_title = current_library_state
                    .collection()
                    .entity_brief()
                    .and_then(|(_, collection)| {
                        collection.map(|collection| collection.title.as_str())
                    });
                ui.label("Collection title:");
                ui.label(collection_title.unwrap_or_default());
                ui.end_row();

                let collection_summary = current_library_state
                    .collection()
                    .entity_with_summary()
                    .map(|(_, summary)| summary);
                ui.label("Collection summary:");
                ui.label(collection_summary.map_or("<none>".to_owned(), |summary| {
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
                        current_library_state.could_search_tracks(),
                        TextEdit::singleline(&mut ui_data.track_search_input),
                    )
                    .lost_focus()
                {
                    msg_tx.send_action(TrackSearchAction::Search(ui_data.track_search_input.clone()));
                }
                ui.end_row();
            });
    });

        if let Some(mdl_mode) = &mdl.mode {
            CentralPanel::default().show(ctx, |ui| match mdl_mode {
                ModelMode::TrackSearch { track_list } => {
                    let text_style = egui::TextStyle::Body;
                    let row_height = ui
                        .text_style_height(&text_style)
                        .max(ARTWORK_THUMBNAIL_IMAGE_SIZE as _);
                    let total_rows = track_list.len();
                    ScrollArea::both().drag_to_scroll(true).show_rows(
                        ui,
                        row_height,
                        total_rows,
                        |ui, row_range| {
                            if row_range.end == total_rows
                                // Prevent eagerly fetching more results repeatedly.
                                && Some(total_rows) == current_library_state.track_search().fetched_entities_len()
                                && current_library_state.could_fetch_more_track_search_results()
                            {
                                log::debug!("Trying to fetch more track search results");
                                msg_tx.send_action(TrackSearchAction::FetchMore);
                            }
                            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                                for item in &track_list[row_range] {
                                    debug_assert_eq!(
                                        item.artwork_thumbnail_texture.size_vec2().x,
                                        item.artwork_thumbnail_texture.size_vec2().y
                                    );
                                    let texture = SizedTexture {
                                        id: item.artwork_thumbnail_texture.id(),
                                        size: egui::Vec2::new(row_height, row_height),
                                    };
                                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                        ui.image(texture);
                                        ui.label(track_list_item_label(item));
                                    });
                                }
                            })
                        },
                    );
                }
                ModelMode::MusicDirSync { last_progress, final_outcome } => {
                    ScrollArea::both().drag_to_scroll(true).show(ui, |ui| {
                        if let Some(final_outcome) = final_outcome {
                            let line =format!("{final_outcome:#?}");
                            ui.label(line);
                        } else if let Some(progress) = last_progress {
                            let line = format!("{progress:#?}");
                            ui.label(line);
                        }
                    });
                }
                ModelMode::MusicDirList {
                    content_paths_with_count,
                } => {
                    ScrollArea::both().drag_to_scroll(true).show(ui, |ui| {
                        for (content_path, count) in content_paths_with_count {
                            // Display absolute paths. Otherwise the root folder would become an empty string.
                            ui.label(format!("/{content_path} ({count})"));
                        }
                    });
                }
            });
        }

        TopBottomPanel::bottom("bottem-panel").show(ctx, |ui| {
            Grid::new("grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    if let Some(mdl_mode) = &mdl.mode {
                        let text;
                        let hover_text;
                        let enabled;
                        let action: Action;
                        match mdl_mode {
                            ModelMode::TrackSearch { .. } => {
                                text = "Fetch more";
                                hover_text = "Fetch the next page of search results.";
                                enabled =
                                    current_library_state.could_fetch_more_track_search_results();
                                action = TrackSearchAction::FetchMore.into();
                            }
                            ModelMode::MusicDirSync { .. } => {
                                if current_library_state.could_abort_synchronize_music_dir_task() {
                                    text = "Abort";
                                    hover_text = "Stop the current synchronization task.";
                                    enabled = true;
                                    action = MusicDirectoryAction::AbortPendingSyncTask.into();
                                } else {
                                    text = "Dismiss";
                                    hover_text = "Clear output and return to track search.";
                                    enabled = true;
                                    action = CollectionAction::RefreshFromDb.into();
                                }
                            }
                            ModelMode::MusicDirList { .. } => {
                                text = "Dismiss";
                                hover_text = "Clear output and return to track search.";
                                enabled = true;
                                action = CollectionAction::RefreshFromDb.into();
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
                    let last_error = current_library_state
                        .collection()
                        .last_error()
                        .map(ToOwned::to_owned)
                        .or_else(|| {
                            current_library_state
                                .track_search()
                                .last_fetch_error()
                                .map(ToString::to_string)
                        });
                    if let Some(last_error) = last_error.as_deref() {
                        ui.label(last_error);
                    }
                    ui.end_row();
                });
        });
    }
}

#[must_use]
fn track_list_item_label(track: &TrackListItem) -> String {
    let track_title = track.title.as_deref().unwrap_or("Untitled");
    let track_artist = &track.artist;
    let album_title = &track.album_title;
    let album_artist = &track.album_artist;
    match (track_artist, album_title, album_artist) {
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
    }
}
