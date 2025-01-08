// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::mpsc;

use egui::Context;

use aoide::util::fs::DirPath;

use crate::{
    library::{self, ui::TrackListItem},
    NoReceiverForEvent,
};

#[allow(missing_debug_implementations)]
struct NoReceiverForMessage(Message);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct MessageSender {
    ctx: Context,
    msg_tx: mpsc::Sender<Message>,
}

impl MessageSender {
    pub(crate) const fn new(ctx: Context, msg_tx: mpsc::Sender<Message>) -> Self {
        Self { ctx, msg_tx }
    }

    pub(crate) fn send_action<T>(&self, action: T)
    where
        T: Into<Action>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Action(action.into())) {
            let Message::Action(action) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action {action:?}");
        }
    }

    pub(crate) fn emit_event<T>(&self, event: T) -> Result<(), NoReceiverForEvent>
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

// Not cloneable so large enum variants should be fine.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Message {
    Action(Action),
    Event(Event),
}

impl From<Action> for Message {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

impl From<Event> for Message {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

#[derive(Debug)]
pub(crate) enum Action {
    Library(LibraryAction),
}

#[derive(Debug)]
pub(crate) enum LibraryAction {
    MusicDirectory(MusicDirectoryAction),
    MediaTracker(MediaTrackerAction),
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

#[derive(Debug)]
pub(crate) enum MusicDirectoryAction {
    Reset,
    Select,
    Update(Option<DirPath<'static>>),
}

impl From<MusicDirectoryAction> for LibraryAction {
    fn from(action: MusicDirectoryAction) -> Self {
        Self::MusicDirectory(action)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MediaTrackerAction {
    Sync(MediaTrackerSyncAction),
    DirList(MediaTrackerDirListAction),
}

impl From<MediaTrackerAction> for LibraryAction {
    fn from(action: MediaTrackerAction) -> Self {
        Self::MediaTracker(action)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MediaTrackerSyncAction {
    SpawnTask,
    AbortPendingTask,
    Finish,
}

impl From<MediaTrackerSyncAction> for LibraryAction {
    fn from(action: MediaTrackerSyncAction) -> Self {
        MediaTrackerAction::Sync(action).into()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MediaTrackerDirListAction {
    OpenView,
    CloseView,
}

impl From<MediaTrackerDirListAction> for LibraryAction {
    fn from(action: MediaTrackerDirListAction) -> Self {
        MediaTrackerAction::DirList(action).into()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CollectionAction {
    RefreshFromDb,
}

impl From<CollectionAction> for LibraryAction {
    fn from(action: CollectionAction) -> Self {
        Self::Collection(action)
    }
}

#[derive(Debug)]
pub(crate) enum TrackSearchAction {
    Search(String),
    FetchMore,
    AbortPendingStateChange,
    ApplyPendingStateChange {
        fetched_items: TrackSearchFetchedItems,
    },
}

#[derive(Debug)]
pub(crate) enum TrackSearchFetchedItems {
    Reset,
    Replace(Vec<TrackListItem>),
    Append(Vec<TrackListItem>),
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
pub(crate) enum Event {
    Library(library::Event),
}

impl From<library::Event> for Event {
    fn from(event: library::Event) -> Self {
        Self::Library(event)
    }
}
