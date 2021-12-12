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
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Reader},
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

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Metadata(metaflac::Tag);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        metaflac::Tag::read_from(reader).map(Self).map_err(map_err)
    }

    pub fn find_embedded_artwork_image(&self) -> Option<(ApicType, &str, &[u8])> {
        let Self(metaflac_tag) = self;
        self::find_embedded_artwork_image(metaflac_tag)
    }

    pub fn import_into_track(&self, config: &ImportTrackConfig, track: &mut Track) -> Result<()> {
        let Self(metaflac_tag) = self;
        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            if let Some(streaminfo) = metaflac_tag.get_streaminfo() {
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
                let loudness = vorbis::import_loudness(metaflac_tag);
                let encoder = vorbis::import_encoder(metaflac_tag).map(Into::into);
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

        if let Some(tempo_bpm) = vorbis::import_tempo_bpm(metaflac_tag) {
            track.metrics.tempo_bpm = Some(tempo_bpm);
        }

        if let Some(key_signature) = vorbis::import_key_signature(metaflac_tag) {
            track.metrics.key_signature = key_signature;
        }

        // Track titles
        let track_titles = vorbis::import_track_titles(metaflac_tag);
        if !track_titles.is_empty() {
            track.titles = Canonical::tie(track_titles);
        }

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        if let Some(artists) = metaflac_tag.get_vorbis("ARTIST") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name);
            }
        }
        if let Some(artists) = metaflac_tag.get_vorbis("ARRANGER") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Arranger, name);
            }
        }
        if let Some(compersers) = metaflac_tag.get_vorbis("COMPOSER") {
            for name in compersers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name);
            }
        }
        if let Some(conductors) = metaflac_tag.get_vorbis("CONDUCTOR") {
            for name in conductors {
                push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name);
            }
        }
        if let Some(producers) = metaflac_tag.get_vorbis("PRODUCER") {
            for name in producers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name);
            }
        }
        if let Some(remixers) = metaflac_tag.get_vorbis("REMIXER") {
            for name in remixers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name);
            }
        }
        if let Some(mixers) = metaflac_tag.get_vorbis("MIXER") {
            for name in mixers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Mixer, name);
            }
        }
        if let Some(mixers) = metaflac_tag.get_vorbis("DJMIXER") {
            for name in mixers {
                push_next_actor_role_name(&mut track_actors, ActorRole::DjMixer, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis("ENGINEER") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Engineer, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis("DIRECTOR") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Director, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis("LYRICIST") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis("WRITER") {
            for name in engineers {
                push_next_actor_role_name(&mut track_actors, ActorRole::Writer, name);
            }
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie_replace(Default::default());

        // Album titles
        let album_titles = vorbis::import_album_titles(metaflac_tag);
        if !album_titles.is_empty() {
            album.titles = Canonical::tie(album_titles);
        }

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in metaflac_tag
            .get_vorbis("ALBUMARTIST")
            .into_iter()
            .flatten()
            .chain(
                metaflac_tag
                    .get_vorbis("ALBUM_ARTIST")
                    .into_iter()
                    .flatten(),
            )
            .chain(
                metaflac_tag
                    .get_vorbis("ALBUM ARTIST")
                    .into_iter()
                    .flatten(),
            )
            .chain(metaflac_tag.get_vorbis("ENSEMBLE").into_iter().flatten())
        {
            push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name);
        }
        let album_actors = album_actors.canonicalize_into();
        if !album_actors.is_empty() {
            album.actors = Canonical::tie(album_actors);
        }

        // Album properties
        if let Some(album_kind) = vorbis::import_album_kind(metaflac_tag) {
            album.kind = album_kind;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(released_at) = vorbis::import_released_at(metaflac_tag) {
            track.release.released_at = Some(released_at);
        }
        if let Some(released_by) = vorbis::import_released_by(metaflac_tag) {
            track.release.released_by = Some(released_by);
        }
        if let Some(copyright) = vorbis::import_release_copyright(metaflac_tag) {
            track.release.copyright = Some(copyright);
        }

        let mut tags_map = TagsMap::default();
        if config.flags.contains(ImportTrackFlags::AOIDE_TAGS) {
            // Pre-populate tags
            if let Some(tags) = vorbis::import_aoide_tags(metaflac_tag) {
                debug_assert_eq!(0, tags_map.total_count());
                tags_map = tags.into();
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
            metaflac_tag
                .get_vorbis("COMMENT")
                .into_iter()
                .flatten()
                .chain(metaflac_tag.get_vorbis("DESCRIPTION").into_iter().flatten()),
        );

        // Genre tags
        if let Some(genres) = metaflac_tag.get_vorbis("GENRE") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_GENRE,
                genres,
            );
        }

        // Mood tags
        if let Some(moods) = metaflac_tag.get_vorbis("MOOD") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_MOOD,
                moods,
            );
        }

        // Grouping tags
        if let Some(groupings) = metaflac_tag.get_vorbis("GROUPING") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_GROUPING,
                groupings,
            );
        }

        // ISRC tag
        if let Some(isrc) = metaflac_tag.get_vorbis("ISRC") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_ISRC,
                isrc,
            );
        }

        if let Some(index) = vorbis::import_track_index(metaflac_tag) {
            track.indexes.track = index;
        }
        if let Some(index) = vorbis::import_disc_index(metaflac_tag) {
            track.indexes.disc = index;
        }
        if let Some(index) = vorbis::import_movement_index(metaflac_tag) {
            track.indexes.movement = index;
        }

        if config.flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
            let artwork = if let Some((apic_type, media_type, image_data)) =
                find_embedded_artwork_image(metaflac_tag)
            {
                try_ingest_embedded_artwork_image(
                    &track.media_source.path,
                    apic_type,
                    image_data,
                    None,
                    Some(media_type.to_owned()),
                    &mut config.flags.new_artwork_digest(),
                )
                .0
            } else {
                Artwork::Missing
            };
            track.media_source.artwork = Some(artwork);
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        // Serato Tags
        if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
            let mut serato_tags = SeratoTagContainer::new();
            vorbis::import_serato_markers2(metaflac_tag, &mut serato_tags, SeratoTagFormat::FLAC);

            let track_cues = serato::read_cues(&serato_tags)?;
            if !track_cues.is_empty() {
                track.cues = Canonical::tie(track_cues);
            }

            track.color = serato::read_track_color(&serato_tags);
        }

        Ok(())
    }
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    let mut metaflac_tag = match metaflac::Tag::read_from_path(path) {
        Ok(metaflac_tag) => metaflac_tag,
        Err(err) => {
            tracing::warn!(
                "Failed to parse metadata from media source '{}': {}",
                track.media_source.path,
                err
            );
            return Err(map_err(err));
        }
    };

    let vorbis_comments_orig = metaflac_tag.vorbis_comments().map(ToOwned::to_owned);
    vorbis::export_track(config, track, &mut metaflac_tag);

    if metaflac_tag.vorbis_comments() == vorbis_comments_orig.as_ref() {
        // Unmodified
        return Ok(false);
    }
    metaflac_tag.write_to_path(path).map_err(map_err)?;
    // Modified
    Ok(true)
}
