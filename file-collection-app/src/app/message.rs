// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::mpsc;

use egui::Context;

use aoide::{Track, TrackUid, util::fs::DirPath};

use super::TrackListColumn;

use crate::{
    NoReceiverForEvent,
    library::{
        self,
        ui::{MultiTrackEdit, TrackListItem},
    },
};

struct NoReceiverForMessage(Message);

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

    #[expect(clippy::result_large_err)]
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

#[derive(Debug)]
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
    TrackListColumns(TrackListColumnsAction),
    PendingEdits(PendingEditsAction),
}

/// Save pending track edits on demand and before closing the application.
#[derive(Debug)]
pub(crate) enum PendingEditsAction {
    /// Save the draft of the track editor if any and write all modified tracks into their files.
    Save { draft: Option<Box<Track>> },
    /// Closing the application has been requested.
    RequestClose,
    /// Continue working without closing the application.
    CancelClose,
    /// Close the application and keep the modified tracks for writing them later.
    CloseWithoutWriting,
    /// Ask for confirmation before reverting all pending edits.
    RequestRevert,
    /// Don't revert the pending edits.
    CancelRevert,
    /// Discard all pending edits by restoring the modified tracks from their files.
    ConfirmRevert,
}

impl From<PendingEditsAction> for Action {
    fn from(action: PendingEditsAction) -> Self {
        Self::PendingEdits(action)
    }
}

/// Select and arrange the columns of the track list.
#[derive(Debug, Clone, Copy)]
pub(crate) enum TrackListColumnsAction {
    /// Select or deselect a column.
    Toggle(TrackListColumn),
    /// Move a selected column.
    Move {
        column: TrackListColumn,
        /// The column is inserted before the column at this index of the current order.
        insertion_index: usize,
    },
}

impl From<TrackListColumnsAction> for Action {
    fn from(action: TrackListColumnsAction) -> Self {
        Self::TrackListColumns(action)
    }
}

#[derive(Debug)]
pub(crate) enum LibraryAction {
    MusicDirectory(MusicDirectoryAction),
    MediaTracker(MediaTrackerAction),
    Collection(CollectionAction),
    TrackSearch(TrackSearchAction),
    TrackEdit(TrackEditAction),
    TrackSelection(TrackSelectionAction),
    ExportUnsynchronizedTracks(ExportUnsynchronizedTracksAction),
    RevertUnsynchronizedTracks(RevertUnsynchronizedTracksAction),
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

#[derive(Debug)]
pub(crate) enum TrackEditAction {
    /// Open the editor and (re-)load the track.
    ///
    /// Discards all pending edits of the currently edited track.
    Open(TrackUid),
    /// Modify a track directly in the track list without writing the file.
    ///
    /// Independent of the track that is currently edited.
    Apply {
        entity_uid: TrackUid,
        edit: Box<MultiTrackEdit>,
    },
    /// Save the edited track and optionally write the file.
    Save { track: Box<Track>, write_file: bool },
    /// Close the editor and discard all pending edits.
    Close,
}

impl From<TrackEditAction> for LibraryAction {
    fn from(action: TrackEditAction) -> Self {
        Self::TrackEdit(action)
    }
}

/// Edit multiple tracks at once.
#[derive(Debug)]
pub(crate) enum TrackSelectionAction {
    /// Add a track to or remove it from the selection.
    ///
    /// Closes the editor of a single track and selects that track.
    Toggle(TrackUid),
    /// Add all tracks from the last toggled track up to the given track.
    ///
    /// The range is determined by the order of the track list.
    SelectRange(TrackUid),
    Clear,
    /// Modify all selected tracks.
    Update {
        edit: Box<MultiTrackEdit>,
        write_files: bool,
    },
    AbortUpdate,
}

impl From<TrackSelectionAction> for LibraryAction {
    fn from(action: TrackSelectionAction) -> Self {
        Self::TrackSelection(action)
    }
}

/// Write the metadata of all unsynchronized tracks into their files.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ExportUnsynchronizedTracksAction {
    SpawnTask,
    AbortPendingTask,
    /// Dismiss the outcome and return to track search.
    Finish,
}

impl From<ExportUnsynchronizedTracksAction> for LibraryAction {
    fn from(action: ExportUnsynchronizedTracksAction) -> Self {
        Self::ExportUnsynchronizedTracks(action)
    }
}

/// Discard all modifications of unsynchronized tracks by restoring them from their files.
#[derive(Debug, Clone, Copy)]
pub(crate) enum RevertUnsynchronizedTracksAction {
    AbortPendingTask,
    /// Dismiss the outcome and return to track search.
    Finish,
}

impl From<RevertUnsynchronizedTracksAction> for LibraryAction {
    fn from(action: RevertUnsynchronizedTracksAction) -> Self {
        Self::RevertUnsynchronizedTracks(action)
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
