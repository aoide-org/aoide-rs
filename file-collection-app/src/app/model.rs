// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::{TrackEntity, TrackUid, media::content::ContentPath};

use crate::library::{Library, track_search, ui::TrackListItem};

// Mutually exclusive modes of operation.
#[derive(Debug)]
#[expect(clippy::large_enum_variant)]
pub(crate) enum ModelMode {
    TrackSearch(TrackSearchMode),
    MusicDirSync {
        last_progress: Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>,
        final_outcome:
            Option<Box<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>>,
    },
    MusicDirList {
        content_paths_with_count: Vec<(ContentPath<'static>, usize)>,
    },
}

#[derive(Debug, Default)]
pub(crate) struct TrackSearchMode {
    pub(crate) memo_state: track_search::MemoState,
    pub(crate) track_list: Option<Vec<TrackListItem>>,
    pub(crate) track_editor: Option<TrackEditor>,
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
    #[expect(dead_code, reason = "Not yet used by the UI")]
    pub(crate) fn entity(&self) -> Option<&TrackEntity> {
        match &self.state {
            TrackEditorState::Loading | TrackEditorState::LoadingFailed => None,
            TrackEditorState::Ready { entity } | TrackEditorState::Saving { entity, .. } => {
                Some(entity)
            }
        }
    }

    #[must_use]
    #[expect(dead_code, reason = "Not yet used by the UI")]
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
        #[expect(dead_code, reason = "Not yet used by the UI")]
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
}

impl Model {
    #[must_use]
    pub(crate) const fn new(library: Library) -> Self {
        Self {
            library,
            mode: None,
            music_dir_selection: None,
        }
    }
}
