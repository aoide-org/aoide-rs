// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::mpsc;

use eframe::{CreationContext, Frame};
use egui::Ui;

mod message;
pub(crate) use self::message::{
    Action, CollectionAction, Event, LibraryAction, Message, MessageSender, MusicDirectoryAction,
    TrackEditAction, TrackListColumnsAction, TrackSearchAction, TrackSearchFetchedItems,
    TrackSelectionAction,
};

mod model;
pub(crate) use self::model::{
    Model, ModelMode, MusicDirSelection, TrackEditor, TrackEditorState, TrackSearchEdit,
    TrackSearchMode, TrackSelection, TrackSelectionState,
};

mod update;
use self::update::UpdateContext;

mod render;
use self::render::RenderContext;

mod track_editor;

mod track_list;
pub(crate) use self::track_list::{TrackListColumn, TrackListColumns};

mod track_selection_editor;

mod pending_edits;
use self::pending_edits::PendingEdits;
pub(crate) use self::pending_edits::{CloseState, ModifiedTrackCount};

/// UI data bindings
///
/// Stores user input and other mutable UI state that needs to be preserved
/// between frames.
#[derive(Debug, Default)]
struct UiData {
    track_search_input: String,
    editor_draft: Option<track_editor::TrackEditorDraft>,
    selection_draft: track_selection_editor::TrackSelectionDraft,
    track_list_table: track_list::TrackListTableState,
    /// Write the files when saving in the track (selection) editor.
    write_files: bool,
}

#[expect(missing_debug_implementations)]
pub struct App {
    rt: tokio::runtime::Handle,

    msg_rx: mpsc::Receiver<Message>,
    msg_tx: MessageSender,

    mdl: Model,

    ui_data: UiData,
}

impl App {
    #[must_use]
    pub(crate) fn new(ctx: &CreationContext<'_>, rt: tokio::runtime::Handle, mdl: Model) -> Self {
        Self::with_context(ctx.egui_ctx.clone(), rt, mdl)
    }

    #[must_use]
    fn with_context(ctx: egui::Context, rt: tokio::runtime::Handle, mdl: Model) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx, msg_tx);
        mdl.library
            .spawn_background_tasks(&rt, mdl.settings_dir.clone());
        mdl.library.spawn_event_tasks(&rt, &msg_tx);
        Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ui_data: Default::default(),
        }
    }

    fn update(&mut self) -> (&mpsc::Receiver<Message>, UpdateContext<'_>) {
        let Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ..
        } = self;
        let ctx = UpdateContext { rt, msg_tx, mdl };
        (msg_rx, ctx)
    }

    fn render(&mut self) -> RenderContext<'_> {
        let Self {
            msg_tx,
            mdl,
            ui_data,
            ..
        } = self;
        RenderContext {
            msg_tx,
            mdl,
            ui_data,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frm: &mut Frame) {
        self.run_frame(ui);
    }
}

impl App {
    fn run_frame(&mut self, ui: &mut Ui) {
        let (msg_rx, mut update_ctx) = self.update();
        let msg_count = msg_rx
            .try_iter()
            .map(|msg| {
                update_ctx.on_message(ui.ctx(), msg);
            })
            .count();
        if msg_count > 0 {
            log::debug!("Processed {msg_count} message(s) before rendering frame");
        }

        let pending_edits = PendingEdits::new(&self.mdl, &mut self.ui_data);
        self.mdl.close_state = pending_edits::handle_close(
            ui.ctx(),
            &self.msg_tx,
            self.mdl.close_state,
            &pending_edits,
            || self.ui_data.track_list_table.finish_cell_edit(&self.msg_tx),
        );

        let mut render_ctx = self.render();
        render_ctx.render_frame(ui, &pending_edits);
    }
}
