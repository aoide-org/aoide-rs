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

use tantivy::{directory::MmapDirectory, Index};

use aoide_core::entity::EntityUid;
use aoide_core_api::Pagination;
use aoide_index_tantivy::TrackIndex;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

pub async fn index_tracks(
    index_path: Option<&Path>,
    db_gatekeeper: &Gatekeeper,
    collection_uid: &EntityUid,
    index_writer_overall_heap_size_in_bytes: NonZeroUsize,
    batch_size: NonZeroU64,
    mut progress_fn: impl FnMut(u64),
) -> anyhow::Result<(Arc<TrackIndex>, u64)> {
    let (schema, fields) = aoide_index_tantivy::build_schema_for_tracks();
    let index = if let Some(index_path) = index_path {
        fs::create_dir_all(&index_path)?;
        let index_dir = MmapDirectory::open(index_path)?;
        if Index::exists(&index_dir)? {
            log::info!("Opening track index in directory: {}", index_path.display());
            let index = Index::open(index_dir)?;
            let index_schema = index.schema();
            if index_schema != schema {
                anyhow::bail!(
                    "Incompatible track index schema: expected = {:?}, actual = {:?}",
                    schema,
                    index_schema
                );
            }
            let mut index_writer = index.writer(index_writer_overall_heap_size_in_bytes.get())?;
            let index_reader = index.reader()?;
            let searcher = index_reader.searcher();
            let count = super::reindex_tracks::reindex_recently_updated_tracks(
                &fields,
                &mut index_writer,
                &searcher,
                db_gatekeeper,
                collection_uid,
                batch_size,
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
        Index::create_in_dir(index_path, schema)?
    } else {
        log::warn!("Creating track index in RAM");
        Index::create_in_ram(schema)
    };
    let mut offset = 0;
    let mut index_writer = index.writer(index_writer_overall_heap_size_in_bytes.get())?;
    loop {
        let params = Default::default();
        let pagination = Pagination {
            offset: Some(offset),
            limit: Some(batch_size.get()),
        };
        let entities =
            crate::track::search(db_gatekeeper, collection_uid.to_owned(), params, pagination)
                .await?;
        if entities.is_empty() {
            break;
        }
        offset += entities.len() as u64;
        for entity in entities {
            let doc = fields.create_document(&entity);
            index_writer.add_document(doc);
        }
        progress_fn(offset);
    }
    index_writer.commit()?;
    Ok((Arc::new(TrackIndex { fields, index }), offset))
}
