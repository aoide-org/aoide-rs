// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::content::resolver::{vfs::RemappingVfsResolver, ContentPathResolver as _},
    track::Entity,
};
use aoide_repo::track::RecordHeader;

use super::*;

#[derive(Debug)]
pub struct ResolveUrlFromVirtualFilePathCollector<'c, C> {
    pub resolver: RemappingVfsResolver,
    pub collector: &'c mut C,
}

impl<'c, C> RecordCollector for ResolveUrlFromVirtualFilePathCollector<'c, C>
where
    C: RecordCollector<Header = RecordHeader, Record = Entity>,
{
    type Header = RecordHeader;
    type Record = Entity;

    fn collect(&mut self, header: Self::Header, mut record: Self::Record) {
        let content_path = &record.body.track.media_source.content.link.path;
        debug_assert!(record.body.content_url.is_none());
        record.body.content_url = self
            .resolver
            .resolve_url_from_path(content_path)
            .map_err(|err| {
                log::error!("Failed to convert media source path '{content_path}' to URL: {err}");
            })
            .ok();
        self.collector.collect(header, record);
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
