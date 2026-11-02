// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rendering of the track editor.

use egui::{Button, ComboBox, Grid, ScrollArea, TextEdit, Ui};

use aoide::{TrackEntity, track::AdvisoryRating, track::EntityHeader};

use crate::library::ui::{TrackEditDraft, TrackEditField, TrackEditFieldError};

use super::{MessageSender, TrackEditAction, TrackEditor, TrackEditorState};

/// Editable draft of the edited track.
///
/// Stored in [`super::UiData`] and (re-)initialized from the edited
/// entity whenever the entity header changes, i.e. after (re-)loading
/// or saving the track.
#[derive(Debug)]
pub(super) struct TrackEditorDraft {
    entity_hdr: EntityHeader,
    draft: TrackEditDraft,
}

impl TrackEditorDraft {
    #[must_use]
    fn new(entity: &TrackEntity) -> Self {
        Self {
            entity_hdr: entity.hdr.clone(),
            draft: TrackEditDraft::from_track(&entity.body.track),
        }
    }

    #[must_use]
    pub(super) fn is_modified(&self, track_editor: &TrackEditor) -> bool {
        track_editor
            .entity()
            .is_some_and(|entity| self.draft.is_modified(&entity.body.track))
    }
}

/// Synchronize the draft with the edited entity.
pub(super) fn sync_draft(track_editor: Option<&TrackEditor>, draft: &mut Option<TrackEditorDraft>) {
    let Some(entity) = track_editor.and_then(TrackEditor::entity) else {
        *draft = None;
        return;
    };
    if draft.as_ref().map(|draft| &draft.entity_hdr) != Some(&entity.hdr) {
        log::debug!(
            "Initializing track editor draft for {hdr:?}",
            hdr = entity.hdr
        );
        *draft = Some(TrackEditorDraft::new(entity));
    }
}

pub(super) const FIELD_HINT_RECORDED_AT: &str = "YYYY[-MM[-DD]]";
const FIELD_HINT_KEY: &str = "Lancelot, e.g. 8A";
pub(super) const FIELD_HINT_GENRES: &str = "One genre per line";
pub(super) const FIELD_HINT_COLOR: &str = "#RRGGBB";
pub(super) const FIELD_HINT_CUSTOM_TAGS: &str = "#label facet#label ?s=0.5#scored-label";

pub(super) fn render_track_editor_panel(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    track_editor: &TrackEditor,
    draft: Option<&mut TrackEditorDraft>,
) {
    let TrackEditor {
        entity_uid,
        state,
        last_error,
    } = track_editor;

    ui.horizontal(|ui| {
        ui.heading("Edit track");
        if ui
            .button("Close")
            .on_hover_text("Close the editor and discard all unsaved changes.")
            .clicked()
        {
            msg_tx.send_action(TrackEditAction::Close);
        }
    });
    ui.separator();

    match state {
        TrackEditorState::Loading => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading...");
            });
        }
        TrackEditorState::LoadingFailed | TrackEditorState::Ready { .. } => (),
        TrackEditorState::Saving { write_file, .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(if *write_file {
                    "Saving and writing file..."
                } else {
                    "Saving..."
                });
            });
        }
    }
    if let Some(last_error) = last_error {
        ui.colored_label(ui.visuals().error_fg_color, last_error);
        if ui
            .button("Reload")
            .on_hover_text("Reload the track and discard all unsaved changes.")
            .clicked()
        {
            msg_tx.send_action(TrackEditAction::Open(entity_uid.clone()));
        }
    }

    let (Some(entity), Some(TrackEditorDraft { draft, .. })) = (track_editor.entity(), draft)
    else {
        return;
    };
    let original_track = &entity.body.track;
    let is_modified = draft.is_modified(original_track);
    let is_synchronized = super::update::is_synchronized(entity);
    if !is_synchronized {
        ui.label("The file has not been written since the track has been modified.");
    }
    let edited_track = draft.to_track(original_track);
    let errors = edited_track.as_ref().err().map_or(&[][..], Vec::as_slice);

    let editable = track_editor.could_save();
    ui.add_enabled_ui(editable, |ui| {
        render_buttons(
            ui,
            msg_tx,
            edited_track.as_ref().ok().filter(|_| editable),
            is_modified,
            is_synchronized,
            || *draft = TrackEditDraft::from_track(original_track),
        );
        ui.separator();
        ScrollArea::vertical().show(ui, |ui| {
            render_fields(ui, draft, errors);
        });
    });
}

fn render_buttons(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    edited_track: Option<&aoide::Track>,
    is_modified: bool,
    is_synchronized: bool,
    revert: impl FnOnce(),
) {
    ui.horizontal(|ui| {
        let save = |write_file| {
            if let Some(track) = edited_track {
                msg_tx.send_action(TrackEditAction::Save {
                    track: Box::new(track.clone()),
                    write_file,
                });
            }
        };
        if ui
            .add_enabled(edited_track.is_some() && is_modified, Button::new("Save"))
            .on_hover_text("Save the changes in the collection without writing the file.")
            .clicked()
        {
            save(false);
        }
        if ui
            .add_enabled(
                edited_track.is_some() && (is_modified || !is_synchronized),
                Button::new("Save & write file"),
            )
            .on_hover_text("Save the changes in the collection and write them into the file.")
            .clicked()
        {
            save(true);
        }
        if ui
            .add_enabled(is_modified, Button::new("Revert"))
            .on_hover_text("Discard all unsaved changes.")
            .clicked()
        {
            revert();
        }
        if is_modified {
            ui.label("Unsaved changes");
        }
    });
}

fn render_fields(ui: &mut Ui, draft: &mut TrackEditDraft, errors: &[TrackEditFieldError]) {
    let TrackEditDraft {
        title,
        artist,
        album_title,
        album_artist,
        album_subtitle,
        recorded_at,
        bpm,
        key,
        genres,
        grouping,
        comment,
        publisher,
        copyright,
        advisory_rating,
        color,
        custom_tags,
    } = draft;
    Grid::new("track-editor-fields")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            use TrackEditField as Field;
            let row = |ui: &mut Ui, label, field, add_contents: &mut dyn FnMut(&mut Ui)| {
                field_row(ui, errors, label, field, add_contents);
            };
            row(ui, "Title", Field::Title, &mut |ui| {
                text_edit(ui, title, None);
            });
            row(ui, "Artist", Field::Artist, &mut |ui| {
                text_edit(ui, artist, None);
            });
            row(ui, "Album", Field::AlbumTitle, &mut |ui| {
                text_edit(ui, album_title, None);
            });
            row(ui, "Album artist", Field::AlbumArtist, &mut |ui| {
                text_edit(ui, album_artist, None);
            });
            row(ui, "Album subtitle", Field::AlbumSubtitle, &mut |ui| {
                text_edit(ui, album_subtitle, None);
            });
            row(ui, "Recorded", Field::RecordedAt, &mut |ui| {
                text_edit(ui, recorded_at, Some(FIELD_HINT_RECORDED_AT));
            });
            row(ui, "BPM", Field::Bpm, &mut |ui| {
                text_edit(ui, bpm, None);
            });
            row(ui, "Key", Field::Key, &mut |ui| {
                text_edit(ui, key, Some(FIELD_HINT_KEY));
            });
            row(ui, "Genres", Field::Genres, &mut |ui| {
                multiline_text_edit(ui, genres, Some(FIELD_HINT_GENRES));
            });
            row(ui, "Grouping", Field::Grouping, &mut |ui| {
                text_edit(ui, grouping, None);
            });
            row(ui, "Comment", Field::Comment, &mut |ui| {
                multiline_text_edit(ui, comment, None);
            });
            row(ui, "Publisher", Field::Publisher, &mut |ui| {
                text_edit(ui, publisher, None);
            });
            row(ui, "Copyright", Field::Copyright, &mut |ui| {
                text_edit(ui, copyright, None);
            });
            row(ui, "Advisory rating", Field::AdvisoryRating, &mut |ui| {
                advisory_rating_combo_box(ui, "track-editor-advisory-rating", advisory_rating);
            });
            row(ui, "Color", Field::Color, &mut |ui| {
                text_edit(ui, color, Some(FIELD_HINT_COLOR));
            });
            row(ui, "Custom tags", Field::CustomTags, &mut |ui| {
                multiline_text_edit(ui, custom_tags, Some(FIELD_HINT_CUSTOM_TAGS));
            });
        });
}

/// Render a grid row with the field errors below the contents.
fn field_row(
    ui: &mut Ui,
    errors: &[TrackEditFieldError],
    label: &str,
    field: TrackEditField,
    add_contents: &mut dyn FnMut(&mut Ui),
) {
    ui.label(label);
    ui.vertical(|ui| {
        add_contents(ui);
        for error in errors.iter().filter(|error| error.field == field) {
            ui.colored_label(ui.visuals().error_fg_color, &error.message);
        }
    });
    ui.end_row();
}

pub(super) fn advisory_rating_combo_box(
    ui: &mut Ui,
    id_salt: &str,
    advisory_rating: &mut Option<AdvisoryRating>,
) {
    ComboBox::from_id_salt(id_salt)
        .selected_text(advisory_rating_text(*advisory_rating))
        .show_ui(ui, |ui| {
            for value in [
                None,
                Some(AdvisoryRating::Unrated),
                Some(AdvisoryRating::Explicit),
                Some(AdvisoryRating::Clean),
            ] {
                ui.selectable_value(advisory_rating, value, advisory_rating_text(value));
            }
        });
}

pub(super) fn text_edit(ui: &mut Ui, value: &mut String, hint: Option<&'static str>) {
    add_text_edit(ui, TextEdit::singleline(value), hint);
}

pub(super) fn multiline_text_edit(ui: &mut Ui, value: &mut String, hint: Option<&'static str>) {
    add_text_edit(ui, TextEdit::multiline(value).desired_rows(2), hint);
}

fn add_text_edit(ui: &mut Ui, mut text_edit: TextEdit<'_>, hint: Option<&'static str>) {
    if let Some(hint) = hint {
        text_edit = text_edit.hint_text(hint);
    }
    ui.add(text_edit.desired_width(f32::INFINITY));
}

#[must_use]
const fn advisory_rating_text(advisory_rating: Option<AdvisoryRating>) -> &'static str {
    match advisory_rating {
        None => "",
        Some(AdvisoryRating::Unrated) => "Unrated",
        Some(AdvisoryRating::Explicit) => "Explicit",
        Some(AdvisoryRating::Clean) => "Clean",
    }
}
