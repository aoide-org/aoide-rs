// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    fs,
    num::{NonZeroU64, NonZeroUsize},
    path::Path,
    sync::Arc,
};

use tantivy::{
    directory::{error::OpenDirectoryError, MmapDirectory},
    Index,
};

use aoide_core::entity::EntityUid;
use aoide_index_tantivy::TrackIndex;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::batch::reindex_tracks::IndexingMode;

pub fn try_open_track_index(index_path: &Path) -> anyhow::Result<Option<TrackIndex>> {
    let index_dir = match MmapDirectory::open(index_path) {
        Ok(index_dir) => {
            if !Index::exists(&index_dir)? {
                return Ok(None);
            }
            index_dir
        }
        Err(OpenDirectoryError::DoesNotExist(_)) => {
            return Ok(None);
        }
        Err(err) => {
            return Err(err.into());
        }
    };
    let index = Index::open(index_dir)?;
    let actual_schema = index.schema();
    let (expected_schema, fields) = aoide_index_tantivy::build_schema_for_tracks();
    if actual_schema != expected_schema {
        anyhow::bail!(
            "Incompatible track index schema: expected = {:?}, actual = {:?}",
            expected_schema,
            actual_schema
        );
    }
    let track_index = TrackIndex { fields, index };
    Ok(Some(track_index))
}

pub async fn index_tracks(
    index_path: Option<&Path>,
    db_gatekeeper: Arc<Gatekeeper>,
    collection_uid: EntityUid,
    index_writer_overall_heap_size_in_bytes: NonZeroUsize,
    batch_size: NonZeroU64,
    progress_fn: impl FnMut(u64) + Send + 'static,
) -> anyhow::Result<(Arc<TrackIndex>, u64)> {
    let (expected_schema, fields) = aoide_index_tantivy::build_schema_for_tracks();
    let index = if let Some(index_path) = index_path {
        fs::create_dir_all(&index_path)?;
        let index_dir = MmapDirectory::open(index_path)?;
        if Index::exists(&index_dir)? {
            log::info!("Opening track index in directory: {}", index_path.display());
            let index = Index::open(index_dir)?;
            let actual_schema = index.schema();
            if actual_schema != expected_schema {
                anyhow::bail!(
                    "Incompatible track index schema: expected = {:?}, actual = {:?}",
                    expected_schema,
                    actual_schema
                );
            }
            let index_writer = index.writer(index_writer_overall_heap_size_in_bytes.get())?;
            let count = super::reindex_tracks::reindex_tracks(
                fields.clone(),
                index_writer,
                db_gatekeeper,
                collection_uid,
                batch_size,
                IndexingMode::RecentlyUpdated,
                progress_fn,
            )
            .await?;
            let track_index = TrackIndex { fields, index };
            return Ok((Arc::new(track_index), count));
        }
        log::info!(
            "Creating track index in directory: {}",
            index_path.display()
        );
        Index::create_in_dir(index_path, expected_schema)?
    } else {
        log::warn!("Creating temporary track index in RAM");
        Index::create_in_ram(expected_schema)
    };
    let index_writer = index.writer(index_writer_overall_heap_size_in_bytes.get())?;
    let count = super::reindex_tracks::reindex_tracks(
        fields.clone(),
        index_writer,
        db_gatekeeper,
        collection_uid,
        batch_size,
        IndexingMode::All,
        progress_fn,
    )
    .await?;
    let track_index = TrackIndex { fields, index };
    Ok((Arc::new(track_index), count))
}
