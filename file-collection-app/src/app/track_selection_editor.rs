// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rendering of the editor for multiple tracks.

use egui::{Button, Grid, ScrollArea, Ui};

use aoide::TrackUid;

use aoide::track::AdvisoryRating;

use crate::library::{
    track_edit::Conflict,
    ui::{
        CustomTagsDelta, MultiTrackEdit, MultiTrackFields, TrackEditField, TrackEditFieldError,
        TrackListItem,
    },
    update_tracks,
};

use super::{
    MessageSender, TrackSelection, TrackSelectionAction, TrackSelectionState,
    track_editor::{
        FIELD_HINT_COLOR, FIELD_HINT_CUSTOM_TAGS, FIELD_HINT_GENRES, FIELD_HINT_RECORDED_AT,
        advisory_rating_combo_box, multiline_text_edit, text_edit,
    },
};

/// A field that is only applied if enabled.
///
/// The value is preserved while disabled.
#[derive(Debug, Default)]
struct OptionalField<T> {
    enabled: bool,
    value: T,
}

impl<T: Clone> OptionalField<T> {
    #[must_use]
    fn to_option(&self) -> Option<T> {
        self.enabled.then(|| self.value.clone())
    }
}

/// Editable inputs for the selected tracks.
///
/// Preserved when the selection changes, e.g. for applying the
/// same changes to multiple selections.
#[derive(Debug, Default)]
pub(super) struct TrackSelectionDraft {
    artist: OptionalField<String>,
    album_title: OptionalField<String>,
    album_artist: OptionalField<String>,
    album_subtitle: OptionalField<String>,
    recorded_at: OptionalField<String>,
    genres: OptionalField<String>,
    grouping: OptionalField<String>,
    comment: OptionalField<String>,
    publisher: OptionalField<String>,
    copyright: OptionalField<String>,
    advisory_rating: OptionalField<Option<AdvisoryRating>>,
    color: OptionalField<String>,
    add_custom_tags: String,
    remove_custom_tags: String,
}

impl TrackSelectionDraft {
    #[must_use]
    fn fields(&self) -> MultiTrackFields {
        let Self {
            artist,
            album_title,
            album_artist,
            album_subtitle,
            recorded_at,
            genres,
            grouping,
            comment,
            publisher,
            copyright,
            advisory_rating,
            color,
            add_custom_tags: _,
            remove_custom_tags: _,
        } = self;
        MultiTrackFields {
            artist: artist.to_option(),
            album_title: album_title.to_option(),
            album_artist: album_artist.to_option(),
            album_subtitle: album_subtitle.to_option(),
            recorded_at: recorded_at.to_option(),
            genres: genres.to_option(),
            grouping: grouping.to_option(),
            comment: comment.to_option(),
            publisher: publisher.to_option(),
            copyright: copyright.to_option(),
            advisory_rating: advisory_rating.to_option(),
            color: color.to_option(),
        }
    }

    fn to_edit(&self) -> Result<MultiTrackEdit, Vec<TrackEditFieldError>> {
        let custom_tags = CustomTagsDelta::parse(&self.add_custom_tags, &self.remove_custom_tags);
        let edit = MultiTrackEdit::new(self.fields(), custom_tags.clone().unwrap_or_default());
        match (edit, custom_tags) {
            (Ok(edit), Ok(_)) => Ok(edit),
            (edit, custom_tags) => Err(edit
                .err()
                .into_iter()
                .flatten()
                .chain(custom_tags.err().into_iter().flatten())
                .collect()),
        }
    }
}

const FIELD_HINT_REMOVE_CUSTOM_TAGS: &str = "#label facet#label";

pub(super) fn render_track_selection_editor_panel(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    track_selection: &TrackSelection,
    track_list: &[TrackListItem],
    draft: &mut TrackSelectionDraft,
) {
    let TrackSelection {
        entity_uids, state, ..
    } = track_selection;
    let is_updating = track_selection.is_updating();

    ui.horizontal(|ui| {
        ui.heading(format!(
            "Edit {count} track{suffix}",
            count = entity_uids.len(),
            suffix = if entity_uids.len() == 1 { "" } else { "s" }
        ));
        if ui
            .add_enabled(!is_updating, Button::new("Close"))
            .on_hover_text("Clear the selection.")
            .clicked()
        {
            msg_tx.send_action(TrackSelectionAction::Clear);
        }
    });
    ui.label("Ctrl+click tracks in the list to add them to or remove them from the selection. Shift+click to add a range.");
    ui.separator();

    ui.add_enabled_ui(!is_updating, |ui| {
        render_inputs(ui, msg_tx, draft);
    });

    match state {
        TrackSelectionState::Updating {
            last_progress,
            write_files,
        } => {
            ui.horizontal(|ui| {
                ui.spinner();
                let action = if *write_files {
                    "Updating and writing files"
                } else {
                    "Updating"
                };
                if let Some(update_tracks::Progress { total, finished }) = last_progress {
                    ui.label(format!("{action}: {finished} of {total}"));
                } else {
                    ui.label(format!("{action}..."));
                }
                if ui
                    .button("Abort")
                    .on_hover_text("Stop after the current track.")
                    .clicked()
                {
                    msg_tx.send_action(TrackSelectionAction::AbortUpdate);
                }
            });
        }
        TrackSelectionState::Ready {
            last_outcome: Some(last_outcome),
        } => {
            ui.separator();
            render_outcome(ui, last_outcome, track_list);
        }
        TrackSelectionState::Ready { last_outcome: None } => (),
    }

    ui.separator();
    ScrollArea::vertical().show(ui, |ui| {
        for entity_uid in entity_uids {
            ui.label(track_label(track_list, entity_uid));
        }
    });
}

fn render_inputs(ui: &mut Ui, msg_tx: &MessageSender, draft: &mut TrackSelectionDraft) {
    let edit = draft.to_edit();
    let errors = edit.as_ref().err().map_or(&[][..], Vec::as_slice);
    ui.label("Only checked fields are set for all selected tracks. Empty fields are cleared.");
    ScrollArea::vertical()
        .id_salt("track-selection-editor-inputs")
        .max_height(ui.available_height() / 2.0)
        .show(ui, |ui| {
            Grid::new("track-selection-editor-fields")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    render_field_inputs(ui, errors, draft);
                    render_custom_tags_inputs(ui, errors, draft);
                });
        });
    render_buttons(ui, msg_tx, edit.as_ref().ok());
}

fn render_field_inputs(
    ui: &mut Ui,
    errors: &[TrackEditFieldError],
    draft: &mut TrackSelectionDraft,
) {
    use TrackEditField as Field;
    let TrackSelectionDraft {
        artist,
        album_title,
        album_artist,
        album_subtitle,
        recorded_at,
        genres,
        grouping,
        comment,
        publisher,
        copyright,
        advisory_rating,
        color,
        add_custom_tags: _,
        remove_custom_tags: _,
    } = draft;
    let text_row = |ui: &mut Ui,
                    label: &str,
                    field: Field,
                    optional: &mut OptionalField<String>,
                    hint: Option<&'static str>,
                    multiline: bool| {
        optional_field_row(ui, errors, label, field, &mut optional.enabled, |ui| {
            if multiline {
                multiline_text_edit(ui, &mut optional.value, hint);
            } else {
                text_edit(ui, &mut optional.value, hint);
            }
        });
    };
    text_row(ui, "Artist", Field::Artist, artist, None, false);
    text_row(ui, "Album", Field::AlbumTitle, album_title, None, false);
    text_row(
        ui,
        "Album artist",
        Field::AlbumArtist,
        album_artist,
        None,
        false,
    );
    text_row(
        ui,
        "Album subtitle",
        Field::AlbumSubtitle,
        album_subtitle,
        None,
        false,
    );
    text_row(
        ui,
        "Recorded",
        Field::RecordedAt,
        recorded_at,
        Some(FIELD_HINT_RECORDED_AT),
        false,
    );
    text_row(
        ui,
        "Genres",
        Field::Genres,
        genres,
        Some(FIELD_HINT_GENRES),
        true,
    );
    text_row(ui, "Grouping", Field::Grouping, grouping, None, false);
    text_row(ui, "Comment", Field::Comment, comment, None, true);
    text_row(ui, "Publisher", Field::Publisher, publisher, None, false);
    text_row(ui, "Copyright", Field::Copyright, copyright, None, false);
    optional_field_row(
        ui,
        errors,
        "Advisory rating",
        Field::AdvisoryRating,
        &mut advisory_rating.enabled,
        |ui| {
            advisory_rating_combo_box(
                ui,
                "track-selection-editor-advisory-rating",
                &mut advisory_rating.value,
            );
        },
    );
    text_row(
        ui,
        "Color",
        Field::Color,
        color,
        Some(FIELD_HINT_COLOR),
        false,
    );
}

fn render_custom_tags_inputs(
    ui: &mut Ui,
    errors: &[TrackEditFieldError],
    draft: &mut TrackSelectionDraft,
) {
    let TrackSelectionDraft {
        add_custom_tags,
        remove_custom_tags,
        ..
    } = draft;
    field_row(
        ui,
        errors,
        "Add tags",
        TrackEditField::AddCustomTags,
        |ui| {
            multiline_text_edit(ui, add_custom_tags, Some(FIELD_HINT_CUSTOM_TAGS));
        },
    );
    field_row(
        ui,
        errors,
        "Remove tags",
        TrackEditField::RemoveCustomTags,
        |ui| {
            multiline_text_edit(ui, remove_custom_tags, Some(FIELD_HINT_REMOVE_CUSTOM_TAGS));
        },
    );
}

fn render_buttons(ui: &mut Ui, msg_tx: &MessageSender, edit: Option<&MultiTrackEdit>) {
    ui.horizontal(|ui| {
        let has_changes = edit.is_some_and(|edit| !edit.is_empty());
        let update = |write_files| {
            if let Some(edit) = edit {
                msg_tx.send_action(TrackSelectionAction::Update {
                    edit: Box::new(edit.clone()),
                    write_files,
                });
            }
        };
        if ui
            .add_enabled(has_changes, Button::new("Apply"))
            .on_hover_text("Save the changes in the collection without writing the files.")
            .clicked()
        {
            update(false);
        }
        if ui
            .add_enabled(edit.is_some(), Button::new("Apply & write files"))
            .on_hover_text(
                "Save the changes in the collection and write all selected tracks into their files.",
            )
            .clicked()
        {
            update(true);
        }
    });
}

/// Render a grid row with a checkbox for enabling the field.
fn optional_field_row(
    ui: &mut Ui,
    errors: &[TrackEditFieldError],
    label: &str,
    field: TrackEditField,
    enabled: &mut bool,
    add_contents: impl FnOnce(&mut Ui),
) {
    ui.checkbox(enabled, label);
    let enabled = *enabled;
    ui.vertical(|ui| {
        ui.add_enabled_ui(enabled, add_contents);
        render_field_errors(ui, errors, field);
    });
    ui.end_row();
}

fn field_row(
    ui: &mut Ui,
    errors: &[TrackEditFieldError],
    label: &str,
    field: TrackEditField,
    add_contents: impl FnOnce(&mut Ui),
) {
    ui.label(label);
    ui.vertical(|ui| {
        add_contents(ui);
        render_field_errors(ui, errors, field);
    });
    ui.end_row();
}

fn render_field_errors(ui: &mut Ui, errors: &[TrackEditFieldError], field: TrackEditField) {
    for error in errors.iter().filter(|error| error.field == field) {
        ui.colored_label(ui.visuals().error_fg_color, &error.message);
    }
}

fn render_outcome(
    ui: &mut Ui,
    outcome: &Result<Box<update_tracks::Outcome>, String>,
    track_list: &[TrackListItem],
) {
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(err) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("Updating tracks failed: {err}"),
            );
            return;
        }
    };
    let update_tracks::Outcome {
        completion,
        updated,
        unchanged,
        skipped,
        failed,
    } = outcome.as_ref();
    if matches!(completion, update_tracks::Completion::Aborted) {
        ui.label("Aborted.");
    }
    ui.label(format!(
        "Updated: {updated}, unchanged: {unchanged}, skipped: {skipped}, failed: {failed}",
        updated = updated.len(),
        unchanged = unchanged.len(),
        skipped = skipped.len(),
        failed = failed.len(),
    ));
    for (entity_uid, conflict) in skipped {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            format!(
                "Skipped {track}: {reason}",
                track = track_label(track_list, entity_uid),
                reason = conflict_reason(*conflict)
            ),
        );
    }
    for (entity_uid, error) in failed {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!(
                "Failed {track}: {error}",
                track = track_label(track_list, entity_uid)
            ),
        );
    }
}

#[must_use]
const fn conflict_reason(conflict: Conflict) -> &'static str {
    match conflict {
        Conflict::EntityRevision { .. } => "The track has been modified in the meantime.",
        Conflict::ContentRevision { .. } => {
            "The file has been modified on disk or has never been imported. Synchronize the \
             music directory first."
        }
    }
}

#[must_use]
fn track_label(track_list: &[TrackListItem], entity_uid: &TrackUid) -> String {
    track_list
        .iter()
        .find(|item| item.entity_uid == *entity_uid)
        .map_or_else(
            || entity_uid.to_string(),
            super::render::track_list_item_label,
        )
}
