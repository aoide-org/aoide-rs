// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use aoide_core::media::resolver::{FileUrlResolver, SourcePathResolver, VirtualFilePathResolver};
use aoide_repo::{
    collection::EntityRepo as _,
    track::{SearchFilter, SortOrder},
};

mod uc {
    pub use aoide_usecases::{
        collection::resolve_virtual_file_path_collection_id, tracks::search::*, Error,
    };
}

struct ResolveUrlFromVirtualFilePathCollector<'c, C> {
    source_path_resolver: VirtualFilePathResolver,
    collector: &'c mut C,
}

impl<'c, C> RecordCollector for ResolveUrlFromVirtualFilePathCollector<'c, C>
where
    C: RecordCollector<Header = RecordHeader, Record = Entity>,
{
    type Header = RecordHeader;
    type Record = Entity;

    fn collect(&mut self, header: Self::Header, mut record: Self::Record) {
        let path = &record.body.media_source.path;
        match self.source_path_resolver.resolve_url_from_path(path) {
            Ok(url) => {
                record.body.media_source.path = FileUrlResolver
                    .resolve_path_from_url(&url)
                    .expect("percent-encoded URL");
                self.collector.collect(header, record);
            }
            Err(err) => {
                log::error!(
                    "Failed to convert media source path '{}' to URL: {}",
                    path,
                    err
                );
            }
        }
    }
}

impl<'c, C> ReservableRecordCollector for ResolveUrlFromVirtualFilePathCollector<'c, C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
{
    fn reserve(&mut self, additional: usize) {
        self.collector.reserve(additional);
    }
}

pub fn search(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    pagination: &Pagination,
    filter: Option<SearchFilter>,
    ordering: Vec<SortOrder>,
    resolve_url_from_path: bool,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize> {
    let db = RepoConnection::new(&pooled_connection);
    Ok(
        db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
            Ok(if resolve_url_from_path {
                let (collection_id, virtual_file_path_resolver) =
                    uc::resolve_virtual_file_path_collection_id(&db, collection_uid)
                        .map_err(DieselTransactionError::new)?;
                let mut collector = ResolveUrlFromVirtualFilePathCollector {
                    source_path_resolver: virtual_file_path_resolver,
                    collector,
                };
                uc::search(
                    &db,
                    collection_id,
                    pagination,
                    filter,
                    ordering,
                    &mut collector,
                )
            } else {
                let collection_id = db.resolve_collection_id(collection_uid)?;
                uc::search(&db, collection_id, pagination, filter, ordering, collector)
            }?)
        })?,
    )
}
