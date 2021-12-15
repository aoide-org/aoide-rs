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

use opus_headers::{CommentHeader, OpusHeaders, ParseError as OpusError};
use semval::IsValid as _;
use triseratops::tag::{TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat};

use aoide_core::{
    audio::{channel::ChannelCount, signal::SampleRateHz, AudioContent},
    media::{ApicType, Artwork, Content, ContentMetadataFlags},
    tag::TagsMap,
    track::{
        actor::ActorRole,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_MOOD},
        Track,
    },
    util::canonical::{Canonical, CanonicalizeInto as _},
};

use crate::{
    io::import::*,
    util::{push_next_actor_role_name_from, serato, try_ingest_embedded_artwork_image},
    Error, Result,
};

use super::vorbis;

fn map_err(err: OpusError) -> Error {
    match err {
        OpusError::Io(err) => Error::Io(err),
        err => Error::Other(anyhow::Error::from(err)),
    }
}

fn vorbis_comments(header: &'_ CommentHeader) -> impl Iterator<Item = (&'_ str, &'_ str)> + Clone {
    header
        .user_comments
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
}

fn filter_vorbis_comment_values<'h>(
    header: &'h CommentHeader,
    key: &'h str,
) -> impl Iterator<Item = &'h str> {
    vorbis::filter_comment_values(vorbis_comments(header), key)
}

impl vorbis::CommentReader for CommentHeader {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        // TODO: Use vorbis::filter_comment_values()
        self.user_comments.iter().find_map(|(k, v)| {
            if k.eq_ignore_ascii_case(key) {
                Some(v.as_str())
            } else {
                None
            }
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct Metadata(OpusHeaders);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        opus_headers::parse_from_read(reader)
            .map(Self)
            .map_err(map_err)
    }

    pub fn find_embedded_artwork_image(&self) -> Option<(ApicType, String, Vec<u8>)> {
        let Self(OpusHeaders {
            id: _,
            comments: comment_header,
        }) = self;
        vorbis::find_embedded_artwork_image(vorbis_comments(comment_header))
    }

    pub fn import_into_track(&self, config: &ImportTrackConfig, track: &mut Track) -> Result<()> {
        let Self(OpusHeaders {
            id: id_header,
            comments: comment_header,
        }) = self;
        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            let channel_count = ChannelCount(id_header.channel_count.into());
            let channels = if channel_count.is_valid() {
                Some(channel_count.into())
            } else {
                tracing::warn!("Invalid channel count: {}", channel_count.0);
                None
            };
            let bitrate = None;
            let sample_rate = SampleRateHz::from_inner(id_header.input_sample_rate.into());
            let sample_rate = if sample_rate.is_valid() {
                Some(sample_rate)
            } else {
                tracing::warn!("Invalid sample rate: {}", sample_rate);
                None
            };
            let loudness = vorbis::import_loudness(comment_header);
            let encoder = vorbis::import_encoder(comment_header).map(Into::into);
            // TODO: The duration is not available from any header!?
            let duration = None;
            let audio_content = AudioContent {
                duration,
                channels,
                sample_rate,
                bitrate,
                loudness,
                encoder,
            };
            track.media_source.content = Content::Audio(audio_content);
        }

        // TODO: Move common code for importing track metadata from VorbisComments
        // into crate::fmt::vorbis

        if let Some(tempo_bpm) = vorbis::import_tempo_bpm(comment_header) {
            track.metrics.tempo_bpm = Some(tempo_bpm);
        }

        if let Some(key_signature) = vorbis::import_key_signature(comment_header) {
            track.metrics.key_signature = key_signature;
        }

        // Track titles
        let track_titles = vorbis::import_track_titles(comment_header);
        if !track_titles.is_empty() {
            track.titles = Canonical::tie(track_titles);
        }

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        for name in filter_vorbis_comment_values(comment_header, "ARTIST") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "ARRANGER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Arranger, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "COMPOSER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "CONDUCTOR") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "PRODUCER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Producer, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "REMIXER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "MIXER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Mixer, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "DJMIXER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::DjMixer, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "ENGINEER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Engineer, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "DIRECTOR") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "LYRICIST") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
        }
        for name in filter_vorbis_comment_values(comment_header, "WRITER") {
            push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie_replace(Default::default());

        // Album titles
        let album_titles = vorbis::import_album_titles(comment_header);
        if !album_titles.is_empty() {
            album.titles = Canonical::tie(album_titles);
        }

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in filter_vorbis_comment_values(comment_header, "ALBUMARTIST")
            .chain(filter_vorbis_comment_values(comment_header, "ALBUM_ARTIST"))
            .chain(filter_vorbis_comment_values(comment_header, "ALBUM ARTIST"))
            .chain(filter_vorbis_comment_values(comment_header, "ENSEMBLE"))
        {
            push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
        }
        let album_actors = album_actors.canonicalize_into();
        if !album_actors.is_empty() {
            album.actors = Canonical::tie(album_actors);
        }

        // Album properties
        if let Some(album_kind) = vorbis::import_album_kind(comment_header) {
            album.kind = album_kind;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(released_at) = vorbis::import_released_at(comment_header) {
            track.release.released_at = Some(released_at);
        }
        if let Some(released_by) = vorbis::import_released_by(comment_header) {
            track.release.released_by = Some(released_by);
        }
        if let Some(copyright) = vorbis::import_release_copyright(comment_header) {
            track.release.copyright = Some(copyright);
        }

        let mut tags_map = TagsMap::default();
        if config.flags.contains(ImportTrackFlags::AOIDE_TAGS) {
            // Pre-populate tags
            if let Some(tags) = vorbis::import_aoide_tags(comment_header) {
                debug_assert_eq!(0, tags_map.total_count());
                tags_map = tags.into();
            }
        }

        // Comment tag
        // The original specification only defines a "DESCRIPTION" field,
        // while MusicBrainz recommends to use "COMMENT".
        // http://www.xiph.org/vorbis/doc/v-comment.html
        // https://picard.musicbrainz.org/docs/mappings
        {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_COMMENT,
                filter_vorbis_comment_values(comment_header, "COMMENT")
                    .chain(filter_vorbis_comment_values(comment_header, "DESCRIPTION")),
            );
        }

        // Genre tags
        vorbis::import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_GENRE,
            filter_vorbis_comment_values(comment_header, "GENRE"),
        );

        // Mood tags
        vorbis::import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_MOOD,
            filter_vorbis_comment_values(comment_header, "MOOD"),
        );

        // Grouping tags
        vorbis::import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_GROUPING,
            filter_vorbis_comment_values(comment_header, "GROUPING"),
        );

        // ISRC tags
        vorbis::import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ISRC,
            filter_vorbis_comment_values(comment_header, "ISRC"),
        );

        if let Some(index) = vorbis::import_track_index(comment_header) {
            track.indexes.track = index;
        }
        if let Some(index) = vorbis::import_disc_index(comment_header) {
            track.indexes.disc = index;
        }
        if let Some(index) = vorbis::import_movement_index(comment_header) {
            track.indexes.movement = index;
        }

        if config.flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
            let artwork = if let Some((apic_type, media_type, image_data)) =
                self.find_embedded_artwork_image()
            {
                try_ingest_embedded_artwork_image(
                    &track.media_source.path,
                    apic_type,
                    &image_data,
                    None,
                    Some(media_type),
                    &mut config.flags.new_artwork_digest(),
                )
                .0
            } else {
                Artwork::Missing
            };
            track.media_source.artwork = Some(artwork);
        }

        // Serato Tags
        if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
            let mut serato_tags = SeratoTagContainer::new();
            vorbis::import_serato_markers2(comment_header, &mut serato_tags, SeratoTagFormat::Ogg);

            let track_cues = serato::read_cues(&serato_tags)?;
            if !track_cues.is_empty() {
                track.cues = Canonical::tie(track_cues);
            }

            track.color = serato::read_track_color(&serato_tags);
        }

        Ok(())
    }
}
