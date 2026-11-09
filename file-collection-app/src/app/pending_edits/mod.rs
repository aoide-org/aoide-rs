// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Saving or reverting pending track edits on demand and before closing the application.
//!
//! Track edits are pending until they have been written into the files:
//!
//! - The modified draft of the track editor has not been saved at all.
//! - Modified tracks have been saved in the collection and survive a restart,
//!   but their files have not been written yet.

use egui::{Button, Context, Id, Modal, ScrollArea, Ui, ViewportCommand};

use aoide::Track;

use super::{
    Model, ModelMode, TrackSearchEdit, TrackSearchMode, UiData,
    message::{
        ExportUnsynchronizedTracksAction, MessageSender, PendingEditsAction,
        RevertUnsynchronizedTracksAction,
    },
    render::{render_export_unsynchronized_tracks, render_revert_unsynchronized_tracks},
    track_editor,
};

const CLOSE_DIALOG_WIDTH: f32 = 440.0;
const CLOSE_DIALOG_OUTCOME_MAX_HEIGHT: f32 = 200.0;

/// The number of modified tracks that have not been written into their files.
///
/// Counting is asynchronous. Only the result of the most recent request
/// is accepted, because results might arrive out of order.
#[derive(Debug, Default)]
pub(crate) struct ModifiedTrackCount {
    count: Option<usize>,
    last_request_id: u64,
    is_counting: bool,
}

impl ModifiedTrackCount {
    /// Start counting and return the id of the request.
    pub(crate) const fn start_counting(&mut self) -> u64 {
        self.last_request_id += 1;
        self.is_counting = true;
        self.last_request_id
    }

    /// Accept the result of a request.
    ///
    /// The count is unknown if counting failed.
    pub(crate) const fn finish_counting(&mut self, request_id: u64, count: Option<usize>) -> bool {
        if request_id != self.last_request_id {
            return false;
        }
        self.count = count;
        self.is_counting = false;
        true
    }

    /// Without a collection there are no modified tracks.
    pub(crate) const fn reset(&mut self) {
        // Results of pending requests are outdated.
        self.last_request_id += 1;
        self.is_counting = false;
        self.count = Some(0);
    }
}

/// Closing the application.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloseState {
    #[default]
    NotRequested,
    /// Waiting until no edits are pending or for the user to decide.
    Requested,
    /// Don't intercept closing anymore.
    Confirmed,
}

/// The draft of the track editor.
#[derive(Debug)]
pub(super) enum DraftEdit {
    /// Either no track editor or no unsaved changes.
    Unmodified,
    Valid(Box<Track>),
    /// Could not be saved until the invalid inputs have been fixed.
    Invalid,
}

/// Snapshot of all pending edits.
#[derive(Debug)]
pub(super) struct PendingEdits {
    pub(super) draft: DraftEdit,
    /// Unknown if counting failed.
    pub(super) modified_track_count: Option<usize>,
    pub(super) is_counting: bool,
    /// Tracks are currently written into the database or into their files.
    ///
    /// These tasks would be cancelled when closing the application.
    pub(super) is_writing: bool,
    pub(super) could_write_files: bool,
}

impl PendingEdits {
    /// Capture the current state.
    ///
    /// Must not be invoked while holding a read lock on the state of the library.
    #[must_use]
    pub(super) fn new(mdl: &Model, ui_data: &mut UiData) -> Self {
        let track_editor = match &mdl.mode {
            Some(ModelMode::TrackSearch(TrackSearchMode { edit, .. })) => {
                edit.as_ref().and_then(TrackSearchEdit::track_editor)
            }
            _ => None,
        };
        track_editor::sync_draft(track_editor, &mut ui_data.editor_draft);
        let draft = track_editor
            .zip(ui_data.editor_draft.as_ref())
            .map_or(DraftEdit::Unmodified, |(track_editor, draft)| {
                draft.pending_edit(track_editor)
            });
        let library = mdl.library.read_current_state();
        Self {
            draft,
            modified_track_count: mdl.modified_track_count.count,
            is_counting: mdl.modified_track_count.is_counting,
            is_writing: mdl.active_track_writes > 0
                || mdl.write_files_after_saving_draft
                || library.could_abort_export_unsynchronized_tracks()
                || library.could_abort_revert_unsynchronized_tracks()
                || library.is_updating_tracks(),
            could_write_files: library.could_export_unsynchronized_tracks(),
        }
    }

    #[must_use]
    pub(super) const fn is_draft_modified(&self) -> bool {
        !matches!(self.draft, DraftEdit::Unmodified)
    }

    /// Check if the application could be closed without losing anything.
    #[must_use]
    pub(super) const fn is_empty(&self) -> bool {
        !self.is_draft_modified()
            && !self.is_counting
            && !self.is_writing
            && matches!(self.modified_track_count, Some(0))
    }

    /// Check if there is something to save and saving is currently possible.
    ///
    /// Tracks are also expected to be pending if the count is unknown.
    #[must_use]
    pub(super) const fn could_save(&self) -> bool {
        let has_pending_edits = match self.draft {
            DraftEdit::Unmodified => !matches!(self.modified_track_count, Some(0)),
            DraftEdit::Valid(_) => true,
            DraftEdit::Invalid => return false,
        };
        has_pending_edits && !self.is_writing && self.could_write_files
    }

    /// Check if there is something to revert and reverting is currently possible.
    ///
    /// In contrast to saving this also includes an invalid draft.
    #[must_use]
    pub(super) const fn could_revert(&self) -> bool {
        (self.is_draft_modified() || !matches!(self.modified_track_count, Some(0)))
            && !self.is_writing
            && self.could_write_files
    }

    /// The label of the button for reverting all pending edits.
    #[must_use]
    pub(super) fn revert_button_text(&self) -> String {
        match self.modified_track_count {
            Some(count) if count > 0 => format!("Revert pending edits ({count})"),
            _ => "Revert pending edits".to_owned(),
        }
    }

    /// The label of the button for saving all pending edits.
    #[must_use]
    pub(super) fn save_button_text(&self) -> String {
        match self.modified_track_count {
            Some(count) if count > 0 => format!("Save pending edits ({count})"),
            _ => "Save pending edits".to_owned(),
        }
    }

    /// Check if the user needs to decide.
    ///
    /// Prevents a flickering dialog if counting finds nothing.
    #[must_use]
    const fn is_close_dialog_needed(&self) -> bool {
        self.is_draft_modified()
            || self.is_writing
            || !self.is_counting
            || !matches!(self.modified_track_count, None | Some(0))
    }

    #[must_use]
    pub(super) fn save_action(&self) -> PendingEditsAction {
        let draft = match &self.draft {
            DraftEdit::Valid(track) => Some(track.clone()),
            DraftEdit::Unmodified | DraftEdit::Invalid => None,
        };
        PendingEditsAction::Save { draft }
    }
}

/// Intercept closing the application while edits are pending.
///
/// Must be invoked once per frame before rendering. The edit of a table
/// cell is finished when requested, because it would otherwise get lost.
pub(super) fn handle_close(
    ctx: &Context,
    msg_tx: &MessageSender,
    close_state: CloseState,
    pending_edits: &PendingEdits,
    finish_cell_edit: impl FnOnce(),
) -> CloseState {
    if close_state == CloseState::Confirmed {
        // Closing must not be cancelled anymore.
        return close_state;
    }
    if ctx.input(|input| input.viewport().close_requested()) {
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        // The decision must not depend on another event.
        ctx.request_repaint();
        if close_state == CloseState::NotRequested {
            finish_cell_edit();
            // Sent after the edit of the table cell to count it as a pending write.
            msg_tx.send_action(PendingEditsAction::RequestClose);
        }
        return close_state;
    }
    if close_state == CloseState::Requested && pending_edits.is_empty() {
        log::info!("Closing the application without any pending edits");
        ctx.send_viewport_cmd(ViewportCommand::Close);
        ctx.request_repaint();
        return CloseState::Confirmed;
    }
    close_state
}

#[must_use]
fn revert_dialog_id() -> Id {
    Id::unique("revert-dialog")
}

/// Ask for confirmation, because reverting could not be undone.
pub(super) fn render_revert_dialog(
    ctx: &Context,
    msg_tx: &MessageSender,
    pending_edits: &PendingEdits,
) {
    let response = Modal::new(revert_dialog_id()).show(ctx, |ui| {
        ui.set_width(CLOSE_DIALOG_WIDTH);
        ui.heading("Revert pending track edits?");
        ui.add_space(4.0);
        match pending_edits.modified_track_count {
            Some(0) => (),
            Some(1) => {
                ui.label("1 modified track will be restored from its file.");
            }
            Some(count) => {
                ui.label(format!(
                    "{count} modified tracks will be restored from their files."
                ));
            }
            None => {
                ui.label("All modified tracks will be restored from their files.");
            }
        }
        if pending_edits.is_draft_modified() {
            ui.label("The unsaved changes of the track editor will be discarded.");
        }
        ui.colored_label(
            ui.visuals().warn_fg_color,
            "This includes edits from earlier sessions and cannot be undone. Files that have \
             been modified on disk replace the track as well. Files are not written.",
        );
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(pending_edits.could_revert(), Button::new("Revert"))
                .clicked()
            {
                msg_tx.send_action(PendingEditsAction::ConfirmRevert);
            }
            if ui
                .button("Cancel")
                .on_hover_text("Keep all pending edits.")
                .clicked()
            {
                msg_tx.send_action(PendingEditsAction::CancelRevert);
            }
        });
    });
    if response.should_close() {
        msg_tx.send_action(PendingEditsAction::CancelRevert);
    }
}

#[must_use]
fn close_dialog_id() -> Id {
    Id::unique("close-dialog")
}

pub(super) fn render_close_dialog(
    ctx: &Context,
    msg_tx: &MessageSender,
    pending_edits: &PendingEdits,
    mode: Option<&ModelMode>,
) {
    if !pending_edits.is_close_dialog_needed() {
        return;
    }
    let response = Modal::new(close_dialog_id()).show(ctx, |ui| {
        ui.set_width(CLOSE_DIALOG_WIDTH);
        ui.heading("Pending track edits");
        ui.add_space(4.0);
        render_close_dialog_status(ui, pending_edits, mode);
        ui.separator();
        render_close_dialog_buttons(ui, msg_tx, pending_edits, mode);
    });
    if response.should_close() {
        msg_tx.send_action(PendingEditsAction::CancelClose);
    }
}

fn render_close_dialog_status(ui: &mut Ui, pending_edits: &PendingEdits, mode: Option<&ModelMode>) {
    match pending_edits.draft {
        DraftEdit::Unmodified => (),
        DraftEdit::Valid(_) => {
            ui.label("The track editor has unsaved changes.");
        }
        DraftEdit::Invalid => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                "The track editor has unsaved changes with invalid values that cannot be saved. \
                 Cancel to fix them.",
            );
        }
    }
    if let Some(error) = track_editor_error(mode) {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    match pending_edits.modified_track_count {
        Some(0) => (),
        Some(1) => {
            ui.label("1 modified track has not been written into its file.");
        }
        Some(count) => {
            ui.label(format!(
                "{count} modified tracks have not been written into their files."
            ));
        }
        None if !pending_edits.is_counting => {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "Could not determine the number of modified tracks.",
            );
        }
        None => (),
    }
    if let Some(ModelMode::ExportUnsynchronizedTracks {
        last_progress,
        final_outcome,
    }) = mode
    {
        ScrollArea::vertical()
            .max_height(CLOSE_DIALOG_OUTCOME_MAX_HEIGHT)
            .show(ui, |ui| {
                render_export_unsynchronized_tracks(
                    ui,
                    last_progress.as_ref(),
                    final_outcome.as_ref(),
                );
            });
    } else if let Some(ModelMode::RevertUnsynchronizedTracks {
        last_progress,
        final_outcome,
    }) = mode
    {
        ScrollArea::vertical()
            .max_height(CLOSE_DIALOG_OUTCOME_MAX_HEIGHT)
            .show(ui, |ui| {
                render_revert_unsynchronized_tracks(
                    ui,
                    last_progress.as_ref(),
                    final_outcome.as_ref(),
                );
            });
    } else if pending_edits.is_writing {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Saving...");
        });
    }
}

#[must_use]
fn track_editor_error(mode: Option<&ModelMode>) -> Option<&str> {
    let Some(ModelMode::TrackSearch(TrackSearchMode { edit, .. })) = mode else {
        return None;
    };
    edit.as_ref()
        .and_then(TrackSearchEdit::track_editor)
        .and_then(|track_editor| track_editor.last_error.as_deref())
}

fn render_close_dialog_buttons(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    pending_edits: &PendingEdits,
    mode: Option<&ModelMode>,
) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(pending_edits.could_save(), Button::new("Save and close"))
            .on_hover_text(
                "Save the changes of the track editor and write all modified tracks into their \
                 files. The application stays open if files could not be written.",
            )
            .clicked()
        {
            msg_tx.send_action(pending_edits.save_action());
        }
        if ui
            .add_enabled(
                pending_edits.could_revert(),
                Button::new("Revert and close"),
            )
            .on_hover_text(
                "Discard all pending edits by restoring the modified tracks from their files. \
                 The application stays open if tracks could not be restored.",
            )
            .clicked()
        {
            msg_tx.send_action(PendingEditsAction::RequestRevert);
        }
        if ui
            .add_enabled(
                !pending_edits.is_writing,
                Button::new("Close without writing"),
            )
            .on_hover_text(
                "Discard the unsaved changes of the track editor. Modified tracks remain in the \
                 collection and stay marked as modified, their files could be written later.",
            )
            .clicked()
        {
            msg_tx.send_action(PendingEditsAction::CloseWithoutWriting);
        }
        if pending_edits.is_writing
            && matches!(
                mode,
                Some(ModelMode::RevertUnsynchronizedTracks {
                    final_outcome: None,
                    ..
                })
            )
            && ui
                .button("Abort")
                .on_hover_text("Stop reverting after the current track.")
                .clicked()
        {
            msg_tx.send_action(RevertUnsynchronizedTracksAction::AbortPendingTask);
        }
        if pending_edits.is_writing
            && matches!(
                mode,
                Some(ModelMode::ExportUnsynchronizedTracks {
                    final_outcome: None,
                    ..
                })
            )
            && ui
                .button("Abort")
                .on_hover_text("Stop writing files after the current file.")
                .clicked()
        {
            msg_tx.send_action(ExportUnsynchronizedTracksAction::AbortPendingTask);
        }
        if ui
            .button("Cancel")
            .on_hover_text("Continue working without closing the application.")
            .clicked()
        {
            msg_tx.send_action(PendingEditsAction::CancelClose);
        }
    });
}

#[cfg(test)]
mod flow_tests;

#[cfg(test)]
mod tests;
