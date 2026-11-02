// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rendering of the editor for multiple tracks.

use egui::{Button, Grid, ScrollArea, TextEdit, Ui};

use aoide::TrackUid;

use crate::library::{
    track_edit::Conflict,
    ui::{CustomTagsDelta, TrackEditField, TrackEditFieldError, TrackListItem},
    update_tracks,
};

use super::{MessageSender, TrackSelection, TrackSelectionAction, TrackSelectionState};

/// Editable inputs for the selected tracks.
///
/// Preserved when the selection changes, e.g. for applying the
/// same changes to multiple selections.
#[derive(Debug, Default)]
pub(super) struct TrackSelectionDraft {
    add_custom_tags: String,
    remove_custom_tags: String,
}

const FIELD_HINT_ADD_CUSTOM_TAGS: &str = "#label facet#label ?s=0.5#scored-label";
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
    let TrackSelectionDraft {
        add_custom_tags,
        remove_custom_tags,
    } = draft;
    let custom_tags_delta = CustomTagsDelta::parse(add_custom_tags, remove_custom_tags);
    let errors = custom_tags_delta
        .as_ref()
        .err()
        .map_or(&[][..], Vec::as_slice);
    Grid::new("track-selection-editor-fields")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            field_row(
                ui,
                errors,
                "Add tags",
                TrackEditField::AddCustomTags,
                add_custom_tags,
                FIELD_HINT_ADD_CUSTOM_TAGS,
            );
            field_row(
                ui,
                errors,
                "Remove tags",
                TrackEditField::RemoveCustomTags,
                remove_custom_tags,
                FIELD_HINT_REMOVE_CUSTOM_TAGS,
            );
        });
    ui.horizontal(|ui| {
        let custom_tags_delta = custom_tags_delta.as_ref().ok();
        let has_changes = custom_tags_delta.is_some_and(|delta| !delta.is_empty());
        let update = |write_files| {
            if let Some(custom_tags_delta) = custom_tags_delta {
                msg_tx.send_action(TrackSelectionAction::Update {
                    custom_tags_delta: Box::new(custom_tags_delta.clone()),
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
            .add_enabled(custom_tags_delta.is_some(), Button::new("Apply & write files"))
            .on_hover_text(
                "Save the changes in the collection and write all selected tracks into their files.",
            )
            .clicked()
        {
            update(true);
        }
    });
}

fn field_row(
    ui: &mut Ui,
    errors: &[TrackEditFieldError],
    label: &str,
    field: TrackEditField,
    value: &mut String,
    hint: &'static str,
) {
    ui.label(label);
    ui.vertical(|ui| {
        ui.add(
            TextEdit::multiline(value)
                .desired_rows(2)
                .hint_text(hint)
                .desired_width(f32::INFINITY),
        );
        for error in errors.iter().filter(|error| error.field == field) {
            ui.colored_label(ui.visuals().error_fg_color, &error.message);
        }
    });
    ui.end_row();
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
