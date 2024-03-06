// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::media::content::ContentPath;

use crate::library::{track_search, ui::TrackListItem, Library};

// Mutually exclusive modes of operation.
#[derive(Debug)]
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
}

#[derive(Debug)]
pub(crate) enum MusicDirSelection {
    Selecting,
    Selected,
}

/// Application model
///
/// Immutable during rendering.
#[allow(missing_debug_implementations)]
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
