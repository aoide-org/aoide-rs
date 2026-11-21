// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::Seek as _,
    path::Path,
};

use bitflags::bitflags;
use lofty::{
    file::{AudioFile, FileType},
    ogg::{OpusFile, VorbisFile},
    probe::Probe,
};

use aoide_core::{
    media::artwork::EditEmbeddedArtworkImage,
    track::{
        Track,
        actor::{Actor, Actors, Kind as ActorKind, Role as ActorRole},
    },
};

#[cfg(feature = "gigtag")]
use aoide_core::tag::FacetId;

use crate::{Error, Result, fmt::parse_options, util::tag::FacetedTagMappingConfig};

use super::import::ImportTrackFlags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ExportTrackFlags: u16 {
        /// See also: [`super::import::ImportTrackFlags`]
        const COMPATIBILITY_ID3V2_APPLE_GRP1 = ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1.bits();

        #[cfg(feature = "serato-markers")]
        const SERATO_MARKERS                 = ImportTrackFlags::SERATO_MARKERS.bits();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ExportTrackFlags,

    // Encode gig tags into the corresponding file tag.
    #[cfg(feature = "gigtag")]
    pub encode_gigtags: Option<FacetId>,
}

impl Default for ExportTrackConfig {
    fn default() -> Self {
        Self {
            faceted_tag_mapping: Default::default(),
            flags: ExportTrackFlags::all()
                .difference(ExportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
            #[cfg(feature = "gigtag")]
            encode_gigtags: None,
        }
    }
}

#[must_use]
fn file_extension_str(path: &Path) -> Option<&str> {
    path.extension().and_then(OsStr::to_str)
}

pub fn export_track_to_file_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    // Existing tags must be read before they could be replaced.
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let file_type = if let Some(file_type) = file_extension_str(path).and_then(FileType::from_ext) {
        file_type
    } else {
        let probe = Probe::new(file.try_clone()?).guess_file_type()?;
        // Ensure that the file could be read again.
        file.rewind()?;
        let Some(file_type) = probe.file_type() else {
            log::debug!(
                "Skipping export of track {media_source_content_link:?}: {config:?}",
                media_source_content_link = track.media_source.content.link
            );
            return Err(Error::UnsupportedContentType(
                track.media_source.content.r#type.clone(),
            ));
        };
        file_type
    };
    export_track_to_file_with_type(
        &mut file,
        file_type,
        config,
        track,
        edit_embedded_artwork_image,
    )
}

pub fn export_track_to_file_with_type(
    file: &mut File,
    file_type: FileType,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    // Both reading and saving require the file cursor to be at the beginning.
    file.rewind()?;
    match file_type {
        FileType::Aiff => export_track_to_file_with(
            file,
            config,
            track,
            edit_embedded_artwork_image,
            crate::fmt::aiff::export_track_to_file_id3v2,
        ),
        FileType::Flac => export_track_to_file_with(
            file,
            config,
            track,
            edit_embedded_artwork_image,
            crate::fmt::flac::export_track_to_file_vorbis_comments,
        ),
        FileType::Mp4 => export_track_to_file_with(
            file,
            config,
            track,
            edit_embedded_artwork_image,
            crate::fmt::mp4::export_track_to_file_ilst,
        ),
        FileType::Mpeg => export_track_to_file_with(
            file,
            config,
            track,
            edit_embedded_artwork_image,
            crate::fmt::mpeg::export_track_to_file_id3v2,
        ),
        FileType::Opus => export_track_to_file_with(
            file,
            config,
            track,
            edit_embedded_artwork_image,
            crate::fmt::ogg::export_track_to_file::<OpusFile>,
        ),
        FileType::Vorbis => export_track_to_file_with(
            file,
            config,
            track,
            edit_embedded_artwork_image,
            crate::fmt::ogg::export_track_to_file::<VorbisFile>,
        ),
        _ => {
            log::debug!(
                "Skipping export of track {media_source_content_link:?}: {config:?}",
                media_source_content_link = track.media_source.content.link
            );
            Err(Error::UnsupportedContentType(
                track.media_source.content.r#type.clone(),
            ))
        }
    }
}

type ExportTrackToFileFn<F> =
    fn(&mut F, &ExportTrackConfig, &mut Track, Option<EditEmbeddedArtworkImage>) -> Result<()>;

/// Read the file, export the track metadata into it, and save it.
///
/// The file cursor must be at the beginning.
fn export_track_to_file_with<F: AudioFile>(
    file: &mut File,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
    export_track_to_file: ExportTrackToFileFn<F>,
) -> Result<()> {
    let mut audio_file = F::read_from(file, parse_options())?;
    export_track_to_file(&mut audio_file, config, track, edit_embedded_artwork_image)?;
    file.rewind()?;
    audio_file.save_to(file, Default::default())?;
    Ok(())
}

#[derive(Debug, Clone)]
pub enum FilteredActorNames<'a> {
    Summary(&'a str),
    Individual(Vec<&'a str>), // TODO: Replace with impl Iterator<Item = &'a str>! How?
    Sorting(&'a str),
}

impl<'a> FilteredActorNames<'a> {
    #[must_use]
    pub fn filter(
        actors: impl IntoIterator<Item = &'a Actor> + Clone,
        role: ActorRole,
        kind: ActorKind,
    ) -> Option<Self> {
        match kind {
            ActorKind::Summary => {
                // At most a single summary actor
                debug_assert!(
                    Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).count() <= 1
                );
                if let Some(summary_actor) =
                    Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).next()
                {
                    Some(Self::Summary(summary_actor.name.as_str()))
                } else {
                    // Fallback: Individual actors
                    Self::filter(actors, role, ActorKind::Individual)
                }
            }
            ActorKind::Individual => {
                let individual_actors =
                    Actors::filter_kind_role(actors, ActorKind::Individual, role)
                        .map(|actor| actor.name.as_str())
                        .collect::<Vec<_>>();
                if individual_actors.is_empty() {
                    None
                } else {
                    Some(Self::Individual(individual_actors))
                }
            }
            ActorKind::Sorting => {
                // At most a single sorting actor
                debug_assert!(
                    Actors::filter_kind_role(actors.clone(), ActorKind::Sorting, role).count() <= 1
                );
                Actors::filter_kind_role(actors, ActorKind::Sorting, role)
                    .next()
                    .map(|actor| Self::Sorting(actor.name.as_str()))
            }
        }
    }
}
