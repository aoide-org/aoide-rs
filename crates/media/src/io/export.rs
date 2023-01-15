// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fs::OpenOptions, path::Path};

use bitflags::bitflags;

use aoide_core::track::{
    actor::{Actor, Actors, Kind as ActorKind, Role as ActorRole},
    Track,
};

use crate::{util::tag::FacetedTagMappingConfig, Error, Result};

use super::import::ImportTrackFlags;

bitflags! {
    pub struct ExportTrackFlags: u16 {
        /// See also: [`super::import::ImportTrackFlags`]
        const COMPATIBILITY_ID3V2_APPLE_GRP1 = ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1.bits();

        #[cfg(feature = "gigtag")]
        const GIGTAGS                        = ImportTrackFlags::GIGTAGS.bits();

        #[cfg(feature = "serato-markers")]
        const SERATO_MARKERS                 = ImportTrackFlags::SERATO_MARKERS.bits();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ExportTrackFlags,
}

impl Default for ExportTrackConfig {
    fn default() -> Self {
        Self {
            faceted_tag_mapping: Default::default(),
            flags: ExportTrackFlags::all()
                .difference(ExportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        }
    }
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    match track.media_source.content.r#type.essence_str() {
        "audio/aiff" => {
            let mut file = OpenOptions::new().write(true).open(path)?;
            crate::fmt::aiff::export_track_to_file(&mut file, config, track)
        }
        "audio/flac" => {
            let mut file = OpenOptions::new().write(true).open(path)?;
            crate::fmt::flac::export_track_to_file(&mut file, config, track)
        }
        "audio/mpeg" => {
            let mut file = OpenOptions::new().write(true).open(path)?;
            crate::fmt::mpeg::export_track_to_file(&mut file, config, track)
        }
        "audio/m4a" | "video/mp4" => {
            let mut file = OpenOptions::new().write(true).open(path)?;
            crate::fmt::mp4::export_track_to_file(&mut file, config, track)
        }
        "audio/ogg" => {
            let mut file = OpenOptions::new().write(true).open(path)?;
            crate::fmt::ogg::export_track_to_file(&mut file, config, track)
        }
        "audio/opus" => {
            let mut file = OpenOptions::new().write(true).open(path)?;
            crate::fmt::opus::export_track_to_file(&mut file, config, track)
        }
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
    Individual(Vec<&'a str>), // TODO: Replace with impl Iterator<Item = &'a str>! How?
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
            Self::Individual(individual_actors.map(|actor| actor.name.as_str()).collect())
        }
    }
}
