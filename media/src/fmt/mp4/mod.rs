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

use image::ImageFormat;
use mp4ameta::{
    AdvisoryRating as Mp4AdvisoryRating, ChannelConfig, Data, Fourcc, FreeformIdent, ImgFmt,
    SampleRate as Mp4SampleRate, Tag as Mp4Tag, STANDARD_GENRES,
};
use semval::IsValid as _;
use triseratops::tag::{
    format::mp4::MP4Tag, Markers as SeratoMarkers, Markers2 as SeratoMarkers2,
    TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat,
};

use aoide_core::{
    audio::{
        channel::{ChannelCount, ChannelLayout, Channels},
        signal::{BitrateBps, BitsPerSecond, SampleRateHz, SamplesPerSecond},
        AudioContent,
    },
    media::{AdvisoryRating, ApicType, Artwork, Content, ContentMetadataFlags},
    music::time::{Beats, TempoBpm},
    tag::{Score as TagScore, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        metric::MetricsFlags,
        tag::{FACET_CGROUP, FACET_COMMENT, FACET_GENRE, FACET_ISRC, FACET_MOOD, FACET_XID},
        title::{Title, TitleKind},
        Track,
    },
    util::{Canonical, CanonicalizeInto as _},
};

use aoide_core_serde::tag::Tags as SerdeTags;

use crate::{
    io::import::{self, *},
    util::{
        digest::MediaDigest, parse_key_signature, parse_replay_gain, parse_tempo_bpm,
        parse_year_tag, push_next_actor_role_name, serato, tag::import_faceted_tags,
        try_load_embedded_artwork,
    },
    Result,
};

#[derive(Debug)]
pub struct ImportTrack;

fn read_bitrate(bitrate: u32) -> Option<BitrateBps> {
    let bits_per_second = bitrate as BitsPerSecond;
    let bitrate_bps = BitrateBps::from_inner(bits_per_second);
    if bitrate_bps >= BitrateBps::min() {
        Some(bitrate_bps)
    } else {
        None
    }
}

fn read_channels(channel_config: ChannelConfig) -> Channels {
    use ChannelConfig::*;
    let channels = match channel_config {
        Mono => Channels::Layout(ChannelLayout::Mono),
        Stereo => Channels::Layout(ChannelLayout::Stereo),
        Three => Channels::Count(ChannelCount(3)),
        Four => Channels::Count(ChannelCount(4)),
        Five => Channels::Count(ChannelCount(5)),
        FiveOne => Channels::Layout(ChannelLayout::FiveOne),
        SevenOne => Channels::Layout(ChannelLayout::SevenOne),
    };
    // Discard the layout and only return the channel count.
    // In the database only the channel count will be stored.
    // Otherwise imported metadata would repeatedly be detected
    // as modified!
    channels.count().into()
}

const COM_APPLE_ITUNES_FREEFORM_MEAN: &str = "com.apple.iTunes";

const BPM_IDENT: FreeformIdent<'static> = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "BPM");

const INITIAL_KEY_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "initialkey");
const KEY_IDENT: FreeformIdent<'static> = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "KEY");

const REPLAYGAIN_TRACK_GAIN_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "replaygain_track_gain");

const SUBTITLE_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "SUBTITLE");

const CONDUCTOR_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "CONDUCTOR");

const PRODUCER_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "PRODUCER");

const REMIXER_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "REMIXER");

const ENGINEER_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "ENGINEER");

const MIXER_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MIXER");

const LABEL_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "LABEL");

const MOOD_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MOOD");

const XID_IDENT: Fourcc = Fourcc(*b"xid ");

const ORG_MIXXX_DJ_FREEFORM_MEAN: &str = "org.mixxx.dj";

const MIXXX_CUSTOM_TAGS_IDENT: FreeformIdent<'static> =
    FreeformIdent::new(ORG_MIXXX_DJ_FREEFORM_MEAN, "CustomTags");

const SERATO_MARKERS_IDENT: FreeformIdent<'static> = FreeformIdent::new(
    SeratoMarkers::MP4_ATOM_FREEFORM_MEAN,
    SeratoMarkers::MP4_ATOM_FREEFORM_NAME,
);

const SERATO_MARKERS2_IDENT: FreeformIdent<'static> = FreeformIdent::new(
    SeratoMarkers2::MP4_ATOM_FREEFORM_MEAN,
    SeratoMarkers2::MP4_ATOM_FREEFORM_NAME,
);

impl import::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        config: &ImportTrackConfig,
        flags: ImportTrackFlags,
        mut track: Track,
        reader: &mut Box<dyn Reader>,
    ) -> Result<Track> {
        // Extract metadata with mp4ameta
        let mut mp4_tag = match Mp4Tag::read_from(reader) {
            Ok(mp4_tag) => mp4_tag,
            Err(err) => {
                tracing::warn!(
                    "Failed to parse metadata from media source '{}': {}",
                    track.media_source.path,
                    err
                );
                return Ok(track);
            }
        };

        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::UNRELIABLE)
        {
            let duration = mp4_tag.duration().map(Into::into);
            let channels = mp4_tag.channel_config().map(read_channels);
            let sample_rate = mp4_tag
                .sample_rate()
                .as_ref()
                .map(Mp4SampleRate::hz)
                .map(|hz| SampleRateHz::from_inner(hz as SamplesPerSecond));
            let bitrate = mp4_tag.avg_bitrate().and_then(read_bitrate);
            let loudness = mp4_tag
                .strings_of(&REPLAYGAIN_TRACK_GAIN_IDENT)
                .next()
                .and_then(parse_replay_gain);
            let encoder = mp4_tag.take_encoder();
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

        if let Some(advisory_rating) = mp4_tag.advisory_rating() {
            let advisory_rating = match advisory_rating {
                Mp4AdvisoryRating::Inoffensive => AdvisoryRating::Unrated,
                Mp4AdvisoryRating::Clean => AdvisoryRating::Clean,
                Mp4AdvisoryRating::Explicit => AdvisoryRating::Explicit,
            };
            debug_assert!(track.media_source.advisory_rating.is_none());
            track.media_source.advisory_rating = Some(advisory_rating);
        }

        let mut tempo_bpm_non_fractional = false;
        let tempo_bpm = mp4_tag
            .strings_of(&BPM_IDENT)
            .flat_map(parse_tempo_bpm)
            .next()
            .or_else(|| {
                mp4_tag.bpm().and_then(|bpm| {
                    tempo_bpm_non_fractional = true;
                    let bpm = TempoBpm(Beats::from(bpm));
                    bpm.is_valid().then(|| bpm)
                })
            });
        if let Some(tempo_bpm) = tempo_bpm {
            debug_assert!(tempo_bpm.is_valid());
            track.metrics.tempo_bpm = Some(tempo_bpm);
            track.metrics.flags.set(
                MetricsFlags::TEMPO_BPM_NON_FRACTIONAL,
                tempo_bpm_non_fractional,
            );
        }

        let key_signature = mp4_tag
            .strings_of(&INITIAL_KEY_IDENT)
            // alternative name (conforms to Rapid Evolution)
            .chain(mp4_tag.strings_of(&KEY_IDENT))
            .flat_map(parse_key_signature)
            .next();
        if let Some(key_signature) = key_signature {
            track.metrics.key_signature = key_signature;
        }

        // Track titles
        let mut track_titles = Vec::with_capacity(4);
        if let Some(name) = mp4_tag.take_title() {
            let title = Title {
                name,
                kind: TitleKind::Main,
            };
            track_titles.push(title);
        }
        if let Some(name) = mp4_tag.take_work() {
            let title = Title {
                name,
                kind: TitleKind::Work,
            };
            track_titles.push(title);
        }
        if let Some(name) = mp4_tag.take_movement() {
            let title = Title {
                name,
                kind: TitleKind::Movement,
            };
            track_titles.push(title);
        }
        if let Some(name) = mp4_tag.take_strings_of(&SUBTITLE_IDENT).next() {
            let title = Title {
                name,
                kind: TitleKind::Sub,
            };
            track_titles.push(title);
        }
        let track_titles = track_titles.canonicalize_into();
        if !track_titles.is_empty() {
            track.titles = Canonical::tie(track_titles);
        }

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        for name in mp4_tag.take_artists() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name);
        }
        for name in mp4_tag.take_composers() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name);
        }
        for name in mp4_tag.take_strings_of(&PRODUCER_IDENT) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name);
        }
        for name in mp4_tag.take_strings_of(&REMIXER_IDENT) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name);
        }
        for name in mp4_tag.take_strings_of(&MIXER_IDENT) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Mixer, name);
        }
        for name in mp4_tag.take_strings_of(&ENGINEER_IDENT) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Engineer, name);
        }
        for name in mp4_tag.take_lyricists() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name);
        }
        for name in mp4_tag.take_strings_of(&CONDUCTOR_IDENT) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name);
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie();

        // Album titles
        let mut album_titles = Vec::with_capacity(1);
        if let Some(name) = mp4_tag.take_album() {
            let title = Title {
                name,
                kind: TitleKind::Main,
            };
            album_titles.push(title);
        }
        let album_titles = album_titles.canonicalize_into();
        if !album_titles.is_empty() {
            album.titles = Canonical::tie(album_titles);
        }

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in mp4_tag.take_album_artists() {
            push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name);
        }
        let album_actors = album_actors.canonicalize_into();
        if !album_actors.is_empty() {
            album.actors = Canonical::tie(album_actors);
        }

        // Album properties
        if mp4_tag.compilation() {
            album.kind = AlbumKind::Compilation;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(year) = mp4_tag.year() {
            if let Some(released_at) = parse_year_tag(year) {
                track.release.released_at = Some(released_at);
            }
        }
        if let Some(copyright) = mp4_tag.take_copyright() {
            track.release.copyright = Some(copyright);
        }
        if let Some(label) = mp4_tag.take_strings_of(&LABEL_IDENT).next() {
            track.release.released_by = Some(label);
        }

        let mut tags_map = TagsMap::default();

        // Mixxx CustomTags
        if flags.contains(ImportTrackFlags::MIXXX_CUSTOM_TAGS) {
            if let Some(data) = mp4_tag.data_of(&MIXXX_CUSTOM_TAGS_IDENT).next() {
                if let Some(custom_tags) = match data {
                    Data::Utf8(input) => serde_json::from_str::<SerdeTags>(input)
                        .map_err(|err| {
                            tracing::warn!("Failed to parse Mixxx custom tags: {}", err);
                            err
                        })
                        .ok(),
                    data => {
                        tracing::warn!("Unexpected data for Mixxx custom tags: {:?}", data);
                        None
                    }
                }
                .map(Tags::from)
                {
                    // Initialize map with all existing custom tags as starting point
                    tags_map = custom_tags.into();
                }
            }
        }

        // Comment tag
        if let Some(comment) = mp4_tag.take_comment() {
            tags_map.remove_faceted_tags(&FACET_COMMENT);
            let mut next_score_value = TagScore::default_value();
            import_faceted_tags(
                &mut tags_map,
                &mut next_score_value,
                &FACET_COMMENT,
                None,
                comment,
            );
        }

        // Genre tags
        let mut genre_count = 0;
        if mp4_tag.custom_genres().next().is_some() {
            tags_map.remove_faceted_tags(&FACET_GENRE);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_GENRE.value());
            let mut next_score_value = TagScore::max_value();
            for genre in mp4_tag.take_custom_genres() {
                genre_count += import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_GENRE,
                    tag_mapping_config,
                    genre,
                );
            }
        }
        if genre_count == 0 && mp4_tag.standard_genres().next().is_some() {
            // Import legacy/standard genres instead
            tags_map.remove_faceted_tags(&FACET_GENRE);
            let mut next_score_value = TagScore::max_value();
            for genre_id in mp4_tag.standard_genres() {
                let genre_id = usize::from(genre_id);
                if genre_id < STANDARD_GENRES.len() {
                    genre_count += import_faceted_tags(
                        &mut tags_map,
                        &mut next_score_value,
                        &FACET_GENRE,
                        None,
                        STANDARD_GENRES[genre_id],
                    );
                }
            }
        }

        // Mood tags
        if mp4_tag.strings_of(&MOOD_IDENT).next().is_some() {
            tags_map.remove_faceted_tags(&FACET_MOOD);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_MOOD.value());
            let mut next_score_value = TagScore::max_value();
            for mood in mp4_tag.take_strings_of(&MOOD_IDENT) {
                import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_MOOD,
                    tag_mapping_config,
                    mood,
                );
            }
        }

        // Grouping tags
        if mp4_tag.groupings().next().is_some() {
            tags_map.remove_faceted_tags(&FACET_CGROUP);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_CGROUP.value());
            let mut next_score_value = TagScore::max_value();
            for grouping in mp4_tag.take_groupings() {
                import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_CGROUP,
                    tag_mapping_config,
                    grouping,
                );
            }
        }

        // ISRC tag
        if let Some(isrc) = mp4_tag.take_isrc() {
            tags_map.remove_faceted_tags(&FACET_ISRC);
            let mut next_score_value = TagScore::default_value();
            import_faceted_tags(
                &mut tags_map,
                &mut next_score_value,
                &FACET_ISRC,
                None,
                isrc,
            );
        }

        // iTunes XID tags
        if mp4_tag.strings_of(&XID_IDENT).next().is_some() {
            tags_map.remove_faceted_tags(&FACET_XID);
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_XID.value());
            let mut next_score_value = TagScore::max_value();
            for xid in mp4_tag.take_strings_of(&XID_IDENT) {
                import_faceted_tags(
                    &mut tags_map,
                    &mut next_score_value,
                    &FACET_XID,
                    tag_mapping_config,
                    xid,
                );
            }
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        // Indexes (in pairs)
        // Import both values consistently if any of them is available!
        if mp4_tag.track_number().is_some() || mp4_tag.total_tracks().is_some() {
            track.indexes.track.number = mp4_tag.track_number();
            track.indexes.track.total = mp4_tag.total_tracks();
        }
        if mp4_tag.disc_number().is_some() || mp4_tag.total_discs().is_some() {
            track.indexes.disc.number = mp4_tag.disc_number();
            track.indexes.disc.total = mp4_tag.total_discs();
        }
        if mp4_tag.movement_index().is_some() || mp4_tag.movement_count().is_some() {
            track.indexes.movement.number = mp4_tag.movement_index();
            track.indexes.movement.total = mp4_tag.movement_count();
        }

        // Artwork
        if flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
            let mut image_digest = if flags.contains(ImportTrackFlags::ARTWORK_DIGEST) {
                if flags.contains(ImportTrackFlags::ARTWORK_DIGEST_SHA256) {
                    // Compatibility
                    MediaDigest::sha256()
                } else {
                    // Default
                    MediaDigest::new()
                }
            } else {
                Default::default()
            };
            track.media_source.artwork = Some(Artwork::Missing);
            for image in mp4_tag.artworks() {
                let (image_data, image_format) = match image.fmt {
                    ImgFmt::Jpeg => (image.data, Some(ImageFormat::Jpeg)),
                    ImgFmt::Png => (image.data, Some(ImageFormat::Png)),
                    ImgFmt::Bmp => (image.data, Some(ImageFormat::Bmp)),
                };
                let artwork = try_load_embedded_artwork(
                    &track.media_source.path,
                    ApicType::Other,
                    image_data,
                    image_format,
                    &mut image_digest,
                )
                .map(Artwork::Embedded);
                if artwork.is_some() {
                    track.media_source.artwork = artwork;
                    break;
                }
            }
        }

        // Serato Tags
        if flags.contains(ImportTrackFlags::SERATO_TAGS) {
            let mut serato_tags = SeratoTagContainer::new();

            if let Some(data) = mp4_tag.data_of(&SERATO_MARKERS_IDENT).next() {
                match data {
                    Data::Utf8(input) => {
                        serato_tags
                            .parse_markers(input.as_bytes(), SeratoTagFormat::MP4)
                            .map_err(|err| {
                                tracing::warn!("Failed to parse Serato Markers: {}", err);
                            })
                            .ok();
                    }
                    data => {
                        tracing::warn!("Unexpected data for Serato Markers: {:?}", data);
                    }
                }
            }

            if let Some(data) = mp4_tag.data_of(&SERATO_MARKERS2_IDENT).next() {
                match data {
                    Data::Utf8(input) => {
                        serato_tags
                            .parse_markers2(input.as_bytes(), SeratoTagFormat::MP4)
                            .map_err(|err| {
                                tracing::warn!("Failed to parse Serato Markers2: {}", err);
                            })
                            .ok();
                    }
                    data => {
                        tracing::warn!("Unexpected data for Serato Markers2: {:?}", data);
                    }
                }
            }

            let track_cues = serato::read_cues(&serato_tags)?;
            if !track_cues.is_empty() {
                track.cues = Canonical::tie(track_cues);
            }

            track.color = serato::read_track_color(&serato_tags);
        }

        Ok(track)
    }
}
