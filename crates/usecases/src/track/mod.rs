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

use aoide_core::track::*;
use aoide_media::resolver::{FileUrlResolver, SourcePathResolver as _, VirtualFilePathResolver};
use aoide_repo::track::RecordHeader;

pub mod find_duplicate;
pub mod purge;
pub mod replace;
pub mod resolve;
pub mod search;

#[derive(Debug)]
pub struct ResolveUrlFromVirtualFilePathCollector<'c, C> {
    pub source_path_resolver: VirtualFilePathResolver,
    pub collector: &'c mut C,
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
                tracing::error!(
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
