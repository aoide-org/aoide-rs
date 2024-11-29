// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::content::resolver::{vfs::RemappingVfsResolver, ContentPathResolver as _},
    TrackEntity,
};
use aoide_repo::{track::RecordHeader, RecordCollector, ReservableRecordCollector};

pub mod export_files;

#[derive(Debug)]
pub struct ResolveUrlFromVirtualFilePathCollector<'c, C> {
    pub resolver: RemappingVfsResolver,
    pub collector: &'c mut C,
}

impl<C> RecordCollector for ResolveUrlFromVirtualFilePathCollector<'_, C>
where
    C: RecordCollector<Header = RecordHeader, Record = TrackEntity>,
{
    type Header = RecordHeader;
    type Record = TrackEntity;

    fn collect(&mut self, record_header: Self::Header, mut track_entity: Self::Record) {
        let content_path = &track_entity.body.track.media_source.content.link.path;
        debug_assert!(track_entity.body.content_url.is_none());
        track_entity.body.content_url = self
            .resolver
            .resolve_url_from_path(content_path)
            .map_err(|err| {
                log::error!("Failed to convert media source path '{content_path}' to URL: {err}");
            })
            .ok();
        self.collector.collect(record_header, track_entity);
    }
}

impl<C> ReservableRecordCollector for ResolveUrlFromVirtualFilePathCollector<'_, C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = TrackEntity>,
{
    fn reserve(&mut self, additional: usize) {
        self.collector.reserve(additional);
    }
}
