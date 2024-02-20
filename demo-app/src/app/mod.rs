// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, sync::mpsc};

use eframe::{CreationContext, Frame};
use egui::Context;

use aoide::desktop_app::fs::DirPath;

use crate::{
    library::{self, Library},
    NoReceiverForEvent,
};

mod update;
use self::update::UpdateContext;

mod render;
use self::render::RenderContext;

#[derive(Debug)]
struct NoReceiverForMessage(Message);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct MessageSender {
    ctx: Context,
    msg_tx: mpsc::Sender<Message>,
}

impl MessageSender {
    const fn new(ctx: Context, msg_tx: mpsc::Sender<Message>) -> Self {
        Self { ctx, msg_tx }
    }

    fn send_action<T>(&self, action: T)
    where
        T: Into<Action>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Action(action.into())) {
            let Message::Action(action) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action: {action:?}");
        }
    }

    #[allow(dead_code)] // TODO: Currently unused?
    fn emit_event<T>(&self, event: T) -> Result<(), NoReceiverForEvent>
    where
        T: Into<Event>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Event(event.into())) {
            let Message::Event(event) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for event: {event:?}");
            return Err(NoReceiverForEvent);
        }
        Ok(())
    }

    fn send_message(&self, msg: Message) -> Result<(), NoReceiverForMessage> {
        log::debug!("Sending message {msg:?}");
        self.msg_tx.send(msg).map_err(|err| {
            log::warn!("Failed to send message: {err}");
            NoReceiverForMessage(err.0)
        })?;
        // Queued messages are consumed before rendering the next frame.
        self.ctx.request_repaint();
        Ok(())
    }
}

impl library::EventEmitter for MessageSender {
    fn emit_event(&self, event: library::Event) -> Result<(), NoReceiverForEvent> {
        let event: Event = Event::Library(event);
        self.send_message(Message::Event(event))
            .map_err(|NoReceiverForMessage(_)| NoReceiverForEvent)
    }
}

#[derive(Debug)]
// Not cloneable so large enum variants should be fine.
#[allow(clippy::large_enum_variant)]
enum Message {
    Action(Action),
    Event(Event),
}

#[derive(Debug, Clone)]
enum Action {
    Library(LibraryAction),
}

impl From<Action> for Message {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

#[derive(Debug, Clone)]
enum LibraryAction {
    MusicDirectory(MusicDirectoryAction),
    Collection(CollectionAction),
    TrackSearch(TrackSearchAction),
}

impl<T> From<T> for Action
where
    T: Into<LibraryAction>,
{
    fn from(action: T) -> Self {
        Self::Library(action.into())
    }
}

#[derive(Debug, Clone)]
enum MusicDirectoryAction {
    Reset,
    Select,
    Selected(Option<DirPath<'static>>),
    SpawnSyncTask,
    AbortPendingSyncTask,
}

impl From<MusicDirectoryAction> for LibraryAction {
    fn from(action: MusicDirectoryAction) -> Self {
        Self::MusicDirectory(action)
    }
}

#[derive(Debug, Clone)]
enum CollectionAction {
    RefreshFromDb,
}

impl From<CollectionAction> for LibraryAction {
    fn from(action: CollectionAction) -> Self {
        Self::Collection(action)
    }
}

#[derive(Debug, Clone)]
enum TrackSearchAction {
    Search(String),
    FetchMore,
}

impl From<TrackSearchAction> for LibraryAction {
    fn from(action: TrackSearchAction) -> Self {
        Self::TrackSearch(action)
    }
}

/// App-level event
///
/// Not cloneable to prevent unintended storage. Notifications are
/// supposed to be ephemeral and should disappear after being processed.
#[derive(Debug)]
enum Event {
    Library(library::Event),
}

impl From<Event> for Message {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

enum CentralPanelData {
    TrackSearch {
        // TODO: Replace string with "renderable" track item.
        track_list: Vec<String>,
    },
    MusicDirSync {
        progress_log: Vec<String>,
    },
}

/// Application model
///
/// Immutable during rendering.
#[allow(missing_debug_implementations)]
pub struct Model {
    library: Library,

    selecting_music_dir: bool,

    central_panel_data: Option<CentralPanelData>,
}

impl Model {
    #[must_use]
    pub const fn new(library: Library) -> Self {
        Self {
            library,
            selecting_music_dir: false,
            central_panel_data: None,
        }
    }
}

/// UI data bindings
///
/// Stores user input and other mutable UI state that needs to be preserved
/// between frames.
#[derive(Debug, Default)]
struct UiData {
    track_search_input: String,
}

#[allow(missing_debug_implementations)]
pub struct App {
    rt: tokio::runtime::Handle,

    msg_rx: mpsc::Receiver<Message>,
    msg_tx: MessageSender,

    mdl: Model,

    ui_data: UiData,
}

impl App {
    #[must_use]
    pub fn new(
        ctx: &CreationContext<'_>,
        rt: tokio::runtime::Handle,
        mdl: Model,
        settings_dir: PathBuf,
    ) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx.egui_ctx.clone(), msg_tx);
        mdl.library.spawn_background_tasks(&rt, settings_dir);
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
    fn update(&mut self, ctx: &Context, frm: &mut Frame) {
        let (msg_rx, mut update_ctx) = self.update();
        let msg_count = msg_rx
            .try_iter()
            .map(|msg| {
                log::debug!("Received message: {msg:?}");
                update_ctx.on_message(msg);
            })
            .count();
        if msg_count > 0 {
            log::debug!("Processed {msg_count} message(s) before rendering frame");
        }

        let mut render_ctx = self.render();
        render_ctx.render_ui(ctx, frm);
    }
}
