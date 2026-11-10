// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use aoide::{TrackEntity, TrackUid, media::content::ContentPath};

use crate::library::{
    Library, export_unsynchronized_tracks, revert_unsynchronized_tracks, track_search,
    ui::TrackListItem, update_tracks,
};

use super::{CloseState, EditorSettings, ModifiedTrackCount, TrackListColumns};

// Mutually exclusive modes of operation.
#[derive(Debug)]
#[expect(clippy::large_enum_variant)]
pub(crate) enum ModelMode {
    TrackSearch(TrackSearchMode),
    MusicDirSync {
        last_progress: Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>,
        /// The error message if synchronization failed.
        final_outcome: Option<
            Result<
                Box<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>,
                String,
            >,
        >,
    },
    MusicDirList {
        content_paths_with_count: Vec<(ContentPath<'static>, usize)>,
    },
    ExportUnsynchronizedTracks {
        last_progress: Option<export_unsynchronized_tracks::Progress>,
        final_outcome: Option<Result<Box<export_unsynchronized_tracks::Outcome>, String>>,
    },
    RevertUnsynchronizedTracks {
        last_progress: Option<revert_unsynchronized_tracks::Progress>,
        final_outcome: Option<Result<Box<revert_unsynchronized_tracks::Outcome>, String>>,
    },
}

#[derive(Debug, Default)]
pub(crate) struct TrackSearchMode {
    pub(crate) memo_state: track_search::MemoState,
    pub(crate) track_list: Option<Vec<TrackListItem>>,
    pub(crate) edit: Option<TrackSearchEdit>,
    /// The error of the last edit that was applied directly in the track list.
    pub(crate) track_list_edit_error: Option<String>,
}

/// Editing of tracks in the track search results.
#[derive(Debug)]
pub(crate) enum TrackSearchEdit {
    /// Edit all properties of a single track.
    Single(TrackEditor),
    /// Edit some properties of multiple tracks at once.
    Multiple(TrackSelection),
}

impl TrackSearchEdit {
    #[must_use]
    pub(crate) const fn track_editor(&self) -> Option<&TrackEditor> {
        match self {
            Self::Single(track_editor) => Some(track_editor),
            Self::Multiple(_) => None,
        }
    }

    #[must_use]
    pub(crate) const fn track_editor_mut(&mut self) -> Option<&mut TrackEditor> {
        match self {
            Self::Single(track_editor) => Some(track_editor),
            Self::Multiple(_) => None,
        }
    }

    #[must_use]
    pub(crate) const fn track_selection(&self) -> Option<&TrackSelection> {
        match self {
            Self::Single(_) => None,
            Self::Multiple(track_selection) => Some(track_selection),
        }
    }

    #[must_use]
    pub(crate) const fn track_selection_mut(&mut self) -> Option<&mut TrackSelection> {
        match self {
            Self::Single(_) => None,
            Self::Multiple(track_selection) => Some(track_selection),
        }
    }

    /// Check if a pending task prevents switching to another track or selection.
    #[must_use]
    pub(crate) const fn is_pending(&self) -> bool {
        match self {
            Self::Single(track_editor) => {
                matches!(track_editor.state, TrackEditorState::Saving { .. })
            }
            Self::Multiple(track_selection) => track_selection.is_updating(),
        }
    }
}

/// Multiple tracks that are edited at once.
#[derive(Debug)]
pub(crate) struct TrackSelection {
    /// The selected tracks in the order of selection.
    pub(crate) entity_uids: Vec<TrackUid>,
    /// The start of ranges, i.e. the last toggled track.
    pub(crate) anchor_entity_uid: Option<TrackUid>,
    pub(crate) state: TrackSelectionState,
}

impl TrackSelection {
    #[must_use]
    pub(crate) const fn new(
        entity_uids: Vec<TrackUid>,
        anchor_entity_uid: Option<TrackUid>,
    ) -> Self {
        Self {
            entity_uids,
            anchor_entity_uid,
            state: TrackSelectionState::Ready { last_outcome: None },
        }
    }

    /// Add or remove a track.
    ///
    /// The track becomes the anchor for selecting ranges.
    pub(crate) fn toggle(&mut self, entity_uid: TrackUid) {
        if let Some(index) = self
            .entity_uids
            .iter()
            .position(|selected| *selected == entity_uid)
        {
            self.entity_uids.remove(index);
        } else {
            self.entity_uids.push(entity_uid.clone());
        }
        self.anchor_entity_uid = Some(entity_uid);
    }

    /// Add all tracks from the anchor up to and including the target.
    ///
    /// The range is determined by the order of `ordered_entity_uids`, i.e.
    /// the order in the track list. Tracks are added in the direction from
    /// the anchor to the target. Already selected tracks are preserved.
    ///
    /// Only selects the target if the anchor is missing or not listed.
    /// The anchor is not modified for selecting multiple ranges from the
    /// same anchor.
    pub(crate) fn select_range<'a>(
        &mut self,
        ordered_entity_uids: impl IntoIterator<Item = &'a TrackUid>,
        target_entity_uid: TrackUid,
    ) {
        let ordered_entity_uids: Vec<_> = ordered_entity_uids.into_iter().collect();
        let position = |entity_uid: &TrackUid| {
            ordered_entity_uids
                .iter()
                .position(|ordered| *ordered == entity_uid)
        };
        let range = self
            .anchor_entity_uid
            .as_ref()
            .and_then(position)
            .zip(position(&target_entity_uid));
        let Some((anchor_index, target_index)) = range else {
            if !self.contains(&target_entity_uid) {
                self.entity_uids.push(target_entity_uid.clone());
            }
            self.anchor_entity_uid = Some(target_entity_uid);
            return;
        };
        let indices: Box<dyn Iterator<Item = usize>> = if anchor_index <= target_index {
            Box::new(anchor_index..=target_index)
        } else {
            Box::new((target_index..=anchor_index).rev())
        };
        for index in indices {
            let entity_uid = ordered_entity_uids[index];
            if !self.contains(entity_uid) {
                self.entity_uids.push(entity_uid.clone());
            }
        }
    }

    #[must_use]
    pub(crate) fn contains(&self, entity_uid: &TrackUid) -> bool {
        self.entity_uids.contains(entity_uid)
    }

    #[must_use]
    pub(crate) const fn is_updating(&self) -> bool {
        matches!(self.state, TrackSelectionState::Updating { .. })
    }
}

#[derive(Debug)]
pub(crate) enum TrackSelectionState {
    Ready {
        last_outcome: Option<Result<Box<update_tracks::Outcome>, String>>,
    },
    Updating {
        last_progress: Option<update_tracks::Progress>,
        write_files: bool,
    },
}

/// Editor for a single track.
///
/// The editable draft is not part of the model. It is supposed to be
/// (re-)initialized from the entity whenever the entity header changes,
/// i.e. after loading or saving the track.
#[derive(Debug)]
pub(crate) struct TrackEditor {
    pub(crate) entity_uid: TrackUid,
    pub(crate) state: TrackEditorState,
    /// The error of the last load or save operation.
    pub(crate) last_error: Option<String>,
}

impl TrackEditor {
    #[must_use]
    pub(crate) const fn loading(entity_uid: TrackUid) -> Self {
        Self {
            entity_uid,
            state: TrackEditorState::Loading,
            last_error: None,
        }
    }

    /// The entity that is edited.
    #[must_use]
    pub(crate) fn entity(&self) -> Option<&TrackEntity> {
        match &self.state {
            TrackEditorState::Loading | TrackEditorState::LoadingFailed => None,
            TrackEditorState::Ready { entity } | TrackEditorState::Saving { entity, .. } => {
                Some(entity)
            }
        }
    }

    #[must_use]
    pub(crate) const fn could_save(&self) -> bool {
        matches!(self.state, TrackEditorState::Ready { .. })
    }

    pub(crate) fn start_saving(&mut self, write_file: bool) {
        let TrackEditorState::Ready { entity } =
            std::mem::replace(&mut self.state, TrackEditorState::Loading)
        else {
            unreachable!("not ready");
        };
        self.state = TrackEditorState::Saving { entity, write_file };
        self.last_error = None;
    }

    /// Continue editing after saving.
    ///
    /// Continues with the saved entity on success. Otherwise continues
    /// with the entity before saving, preserving all pending edits.
    pub(crate) fn finish_saving(&mut self, result: Result<TrackEntity, String>) {
        let TrackEditorState::Saving { entity, .. } =
            std::mem::replace(&mut self.state, TrackEditorState::Loading)
        else {
            unreachable!("not saving");
        };
        let (entity, last_error) = match result {
            Ok(saved_entity) => (Box::new(saved_entity), None),
            Err(error) => (entity, Some(error)),
        };
        self.state = TrackEditorState::Ready { entity };
        self.last_error = last_error;
    }
}

#[derive(Debug)]
pub(crate) enum TrackEditorState {
    Loading,
    LoadingFailed,
    Ready {
        entity: Box<TrackEntity>,
    },
    Saving {
        /// The entity before saving.
        entity: Box<TrackEntity>,
        write_file: bool,
    },
}

#[derive(Debug)]
pub(crate) enum MusicDirSelection {
    Selecting,
    Selected,
}

/// Application model
///
/// Immutable during rendering.
pub(crate) struct Model {
    pub(crate) library: Library,

    pub(crate) mode: Option<ModelMode>,

    pub(crate) music_dir_selection: Option<MusicDirSelection>,

    pub(crate) track_list_columns: TrackListColumns,

    pub(crate) editor_settings: EditorSettings,

    /// The directory for persistent settings.
    pub(crate) settings_dir: PathBuf,

    pub(crate) modified_track_count: ModifiedTrackCount,

    /// The number of single track edits that are still written into the database.
    pub(crate) active_track_writes: usize,

    /// Continue with writing all modified tracks after the draft of the track editor has been saved.
    pub(crate) write_files_after_saving_draft: bool,

    pub(crate) close_state: CloseState,

    /// Reverting is irreversible and needs to be confirmed.
    pub(crate) is_revert_confirmation_open: bool,
}

impl Model {
    #[must_use]
    pub(crate) fn new(library: Library, settings_dir: PathBuf) -> Self {
        let track_list_columns = TrackListColumns::restore(&settings_dir);
        let editor_settings = EditorSettings::restore(&settings_dir);
        Self {
            library,
            mode: None,
            music_dir_selection: None,
            track_list_columns,
            editor_settings,
            settings_dir,
            modified_track_count: Default::default(),
            active_track_writes: 0,
            write_files_after_saving_draft: false,
            close_state: Default::default(),
            is_revert_confirmation_open: false,
        }
    }
}

#[cfg(test)]
mod tests;
