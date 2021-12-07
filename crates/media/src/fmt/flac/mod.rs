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

use std::{path::Path, time::Duration};

use metaflac::block::PictureType;
use num_traits::FromPrimitive as _;

use aoide_core::{
    audio::{channel::ChannelCount, signal::SampleRateHz, AudioContent},
    media::{ApicType, Artwork, Content, ContentMetadataFlags},
    tag::TagsMap,
    track::{
        actor::ActorRole,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_MOOD},
        Track,
    },
    util::{Canonical, CanonicalizeInto as _},
};

use crate::{
    io::{
        export::{self, *},
        import::{self, *},
    },
    util::{push_next_actor_role_name, serato, try_ingest_embedded_artwork_image},
    Error, Result,
};

use super::vorbis;

impl vorbis::CommentReader for metaflac::Tag {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        self.get_vorbis(key).and_then(|mut i| i.next())
    }
}

impl vorbis::CommentWriter for metaflac::Tag {
    fn write_multiple_values(&mut self, key: String, values: Vec<String>) {
        if values.is_empty() {
            self.remove_vorbis(&key);
        } else {
            self.set_vorbis(key, values);
        }
    }
    fn remove_all_values(&mut self, key: &str) {
        self.remove_vorbis(key);
    }
}

use triseratops::tag::{TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat};

fn map_err(err: metaflac::Error) -> Error {
    let metaflac::Error { kind, description } = err;
    match kind {
        metaflac::ErrorKind::Io(err) => Error::Io(err),
        kind => Error::Other(anyhow::Error::from(metaflac::Error { kind, description })),
    }
}

pub fn find_embedded_artwork_image(tag: &metaflac::Tag) -> Option<(ApicType, &str, &[u8])> {
    tag.pictures()
        .filter_map(|p| {
            if p.picture_type == PictureType::CoverFront {
                Some((ApicType::CoverFront, p))
            } else {
                None
            }
        })
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Media {
                Some((ApicType::Media, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Leaflet {
                Some((ApicType::Leaflet, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Other {
                Some((ApicType::Other, p))
            } else {
                None
            }
        }))
        // otherwise take the first picture that could be parsed
        .chain(tag.pictures().map(|p| {
            (
                ApicType::from_u8(p.picture_type as u8).unwrap_or(ApicType::Other),
                p,
            )
        }))
        .map(|(apic_type, p)| (apic_type, p.mime_type.as_str(), p.data.as_slice()))
        .next()
}

#[derive(Debug)]
pub struct ImportTrack;

impl import::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        reader: &mut Box<dyn Reader>,
        config: &ImportTrackConfig,
        track: &mut Track,
    ) -> Result<()> {
        let flac_tag = match metaflac::Tag::read_from(reader) {
            Ok(flac_tag) => flac_tag,
            Err(err) => {
                tracing::warn!(
                    "Failed to parse metadata from media source '{}': {}",
                    track.media_source.path,
                    err
                );
                return Err(map_err(err));
            }
        };

        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            if let Some(streaminfo) = flac_tag.get_streaminfo() {
                let channels = Some(ChannelCount(streaminfo.num_channels.into()).into());
                let duration;
                let sample_rate;
                if streaminfo.sample_rate > 0 {
                    duration = Some(
                        Duration::from_secs_f64(
                            streaminfo.total_samples as f64 / streaminfo.sample_rate as f64,
                        )
                        .into(),
                    );
                    sample_rate = Some(SampleRateHz::from_inner(streaminfo.sample_rate.into()));
                } else {
                    duration = None;
                    sample_rate = None;
                };
                let loudness = vorbis::import_loudness(&flac_tag);
                let encoder = vorbis::import_encoder(&flac_tag).map(Into::into);
                let audio_content = AudioContent {
                    duration,
                    channels,
                    sample_rate,
                    bitrate: None,
                    loudness,
                    encoder,
                };
                track.media_source.content = Content::Audio(audio_content);
            }
        }

        if let Some(tempo_bpm) = vorbis::import_tempo_bpm(&flac_tag) {
            track.metrics.tempo_bpm = Some(tempo_bpm);
        }

        if let Some(key_signature) = vorbis::import_key_signature(&flac_tag) {
            track.metrics.key_signature = key_signature;
        }

        // Track titles
        let track_titles = vorbis::import_track_titles(&flac_tag);
        if !track_titles.is_empty() {
            track.titles = Canonical::tie(track_titles);
        }

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        if let Some(artists) = flac_tag.get_vorbis("ARTIST") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name.to_owned());
            }
        }
        if let Some(artists) = flac_tag.get_vorbis("ARRANGER") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Arranger, name.to_owned());
            }
        }
        if let Some(compersers) = flac_tag.get_vorbis("COMPOSER") {
            for name in compersers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name.to_owned());
            }
        }
        if let Some(conductors) = flac_tag.get_vorbis("CONDUCTOR") {
            for name in conductors {
                push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name.to_owned());
            }
        }
        if let Some(producers) = flac_tag.get_vorbis("PRODUCER") {
            for name in producers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name.to_owned());
            }
        }
        if let Some(remixers) = flac_tag.get_vorbis("REMIXER") {
            for name in remixers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name.to_owned());
            }
        }
        if let Some(mixers) = flac_tag.get_vorbis("MIXER") {
            for name in mixers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Mixer, name.to_owned());
            }
        }
        if let Some(mixers) = flac_tag.get_vorbis("DJMIXER") {
            for name in mixers {
                push_next_actor_role_name(&mut track_actors, ActorRole::DjMixer, name.to_owned());
            }
        }
        if let Some(engineers) = flac_tag.get_vorbis("ENGINEER") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Engineer, name.to_owned());
            }
        }
        if let Some(engineers) = flac_tag.get_vorbis("DIRECTOR") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Director, name.to_owned());
            }
        }
        if let Some(engineers) = flac_tag.get_vorbis("LYRICIST") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name.to_owned());
            }
        }
        if let Some(engineers) = flac_tag.get_vorbis("WRITER") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Writer, name.to_owned());
            }
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie_replace(Default::default());

        // Album titles
        let album_titles = vorbis::import_album_titles(&flac_tag);
        if !album_titles.is_empty() {
            album.titles = Canonical::tie(album_titles);
        }

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in flac_tag
            .get_vorbis("ALBUMARTIST")
            .into_iter()
            .flatten()
            .chain(flac_tag.get_vorbis("ALBUM_ARTIST").into_iter().flatten())
            .chain(flac_tag.get_vorbis("ALBUM ARTIST").into_iter().flatten())
            .chain(flac_tag.get_vorbis("ENSEMBLE").into_iter().flatten())
        {
            push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name.to_owned());
        }
        let album_actors = album_actors.canonicalize_into();
        if !album_actors.is_empty() {
            album.actors = Canonical::tie(album_actors);
        }

        // Album properties
        if let Some(album_kind) = vorbis::import_album_kind(&flac_tag) {
            album.kind = album_kind;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(released_at) = vorbis::import_released_at(&flac_tag) {
            track.release.released_at = Some(released_at);
        }
        if let Some(released_by) = vorbis::import_released_by(&flac_tag) {
            track.release.released_by = Some(released_by);
        }
        if let Some(copyright) = vorbis::import_release_copyright(&flac_tag) {
            track.release.copyright = Some(copyright);
        }

        let mut tags_map = TagsMap::default();
        if config.flags.contains(ImportTrackFlags::MIXXX_CUSTOM_TAGS) {
            if let Some(custom_tags) = vorbis::import_mixxx_custom_tags(&flac_tag) {
                // Initialize map with all existing custom tags as starting point
                debug_assert_eq!(0, tags_map.total_count());
                tags_map = custom_tags.into();
            }
        }

        // Comment tag
        // The original specification only defines a "DESCRIPTION" field,
        // while MusicBrainz recommends to use "COMMENT".
        // http://www.xiph.org/vorbis/doc/v-comment.html
        // https://picard.musicbrainz.org/docs/mappings
        vorbis::import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_COMMENT,
            flac_tag
                .get_vorbis("COMMENT")
                .into_iter()
                .flatten()
                .chain(flac_tag.get_vorbis("DESCRIPTION").into_iter().flatten()),
        );

        // Genre tags
        if let Some(genres) = flac_tag.get_vorbis("GENRE") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_GENRE,
                genres,
            );
        }

        // Mood tags
        if let Some(moods) = flac_tag.get_vorbis("MOOD") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_MOOD,
                moods,
            );
        }

        // Grouping tags
        if let Some(groupings) = flac_tag.get_vorbis("GROUPING") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_GROUPING,
                groupings,
            );
        }

        // ISRC tag
        if let Some(isrc) = flac_tag.get_vorbis("ISRC") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_ISRC,
                isrc,
            );
        }

        if let Some(index) = vorbis::import_track_index(&flac_tag) {
            track.indexes.track = index;
        }
        if let Some(index) = vorbis::import_disc_index(&flac_tag) {
            track.indexes.disc = index;
        }
        if let Some(index) = vorbis::import_movement_index(&flac_tag) {
            track.indexes.movement = index;
        }

        if config.flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
            track.media_source.artwork = find_embedded_artwork_image(&flac_tag)
                .and_then(|(apic_type, media_type, image_data)| {
                    try_ingest_embedded_artwork_image(
                        &track.media_source.path,
                        apic_type,
                        image_data,
                        None,
                        Some(media_type.to_owned()),
                        &mut config.flags.new_artwork_digest(),
                    )
                })
                .map(|(embedded, _)| Artwork::Embedded(embedded))
                .or(Some(Artwork::Missing));
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        // Serato Tags
        if config.flags.contains(ImportTrackFlags::SERATO_TAGS) {
            let mut serato_tags = SeratoTagContainer::new();
            vorbis::import_serato_markers2(&flac_tag, &mut serato_tags, SeratoTagFormat::FLAC);

            let track_cues = serato::read_cues(&serato_tags)?;
            if !track_cues.is_empty() {
                track.cues = Canonical::tie(track_cues);
            }

            track.color = serato::read_track_color(&serato_tags);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ExportTrack;

impl export::ExportTrack for ExportTrack {
    fn export_track_to_path(
        &self,
        config: &ExportTrackConfig,
        path: &Path,
        track: &mut Track,
    ) -> Result<bool> {
        let mut flac_tag = match metaflac::Tag::read_from_path(path) {
            Ok(flac_tag) => flac_tag,
            Err(err) => {
                tracing::warn!(
                    "Failed to parse metadata from media source '{}': {}",
                    track.media_source.path,
                    err
                );
                return Err(map_err(err));
            }
        };

        let vorbis_comments_orig = flac_tag.vorbis_comments().map(ToOwned::to_owned);
        vorbis::export_track(config, track, &mut flac_tag);

        if flac_tag.vorbis_comments() == vorbis_comments_orig.as_ref() {
            // Unmodified
            return Ok(false);
        }
        flac_tag.write_to_path(path).map_err(map_err)?;
        // Modified
        Ok(true)
    }
}
