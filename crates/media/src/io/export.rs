// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use bitflags::bitflags;

use aoide_core::track::{
    actor::{Actor, Actors, Kind as ActorKind, Role as ActorRole},
    Track,
};

use crate::{util::tag::FacetedTagMappingConfig, Error, Result};

use super::import::ImportTrackFlags;

bitflags! {
    pub struct ExportTrackFlags: u16 {
        /// Use iTunes grouping/movement/work mapping
        ///
        /// Use the mapping for grouping and movement/work fields as introduced
        /// by iTunes v12.5.4. This is the preferred mapping and existing files
        /// that still use TIT1 instead of GRP1 for storing the grouping property
        /// should be updated accordingly.
        ///
        /// Implies METADATA.
        const COMPATIBILITY_ID3V2_ITUNES_GROUPING_MOVEMENT_WORK = 0b0000_0001_0000_0001;

        #[cfg(feature = "gigtag")]
        const GIGTAGS = ImportTrackFlags::GIGTAGS.bits();

        #[cfg(feature = "serato-markers")]
        const SERATO_MARKERS = ImportTrackFlags::SERATO_MARKERS.bits();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ExportTrackFlags,
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    match track.media_source.content.r#type.essence_str() {
        #[cfg(feature = "fmt-flac")]
        "audio/flac" => crate::fmt::flac::export_track_to_path(path, config, track),
        #[cfg(feature = "fmt-mp3")]
        "audio/mpeg" => crate::fmt::mp3::export_track_to_path(path, config, track),
        #[cfg(feature = "fmt-mp4")]
        "audio/m4a" | "video/mp4" => crate::fmt::mp4::export_track_to_path(path, config, track),
        // TODO: Add support for audio/ogg
        _ => {
            log::debug!(
                "Skipping export of track {media_source_content_link:?}: {path:?} {config:?}",
                media_source_content_link = track.media_source.content.link
            );
            Err(Error::UnsupportedContentType(
                track.media_source.content.r#type.clone(),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilteredActorNames<'a> {
    Summary(&'a str),
    Primary(Vec<&'a str>), // TODO: Replace with impl Iterator<Item = &'a str>! How?
}

impl<'a> FilteredActorNames<'a> {
    #[must_use]
    pub fn new(actors: impl IntoIterator<Item = &'a Actor> + Clone, role: ActorRole) -> Self {
        // At most a single summary actor
        debug_assert!(
            Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).count() <= 1
        );
        // Either a summary actor or individual actors but not both at the same time
        debug_assert!(
            Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role)
                .next()
                .is_none()
                || Actors::filter_kind_role(actors.clone(), ActorKind::Individual, role)
                    .next()
                    .is_none()
        );
        if let Some(summary_actor) =
            Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).next()
        {
            Self::Summary(summary_actor.name.as_str())
        } else {
            let individual_actors = Actors::filter_kind_role(actors, ActorKind::Individual, role);
            Self::Primary(individual_actors.map(|actor| actor.name.as_str()).collect())
        }
    }
}
