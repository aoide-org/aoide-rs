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

use std::{iter::once, path::Path};

use image::ImageFormat;
use mp4ameta::{
    AdvisoryRating as Mp4AdvisoryRating, ChannelConfig, Data, DataIdent, Fourcc, FreeformIdent,
    Ident, ImgFmt, SampleRate as Mp4SampleRate, Tag as Mp4Tag, STANDARD_GENRES,
};
use semval::{IsValid as _, ValidatedFrom as _};
use triseratops::tag::{
    format::mp4::MP4Tag, Markers as SeratoMarkers, Markers2 as SeratoMarkers2,
    TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat,
};

use aoide_core::{
    audio::{
        channel::{ChannelCount, ChannelLayout, Channels},
        signal::{BitrateBps, BitsPerSecond, LoudnessLufs, SampleRateHz, SamplesPerSecond},
        AudioContent,
    },
    media::{AdvisoryRating, ApicType, Artwork, Content, ContentMetadataFlags},
    music::time::{Beats, TempoBpm},
    tag::{FacetedTags, PlainTag, Score as TagScore, Tags, TagsMap},
    track::{
        actor::ActorRole,
        album::AlbumKind,
        metric::MetricsFlags,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_MOOD, FACET_XID},
        title::{Title, TitleKind, Titles},
        Track,
    },
    util::{Canonical, CanonicalizeInto as _},
};

use aoide_core_serde::tag::Tags as SerdeTags;

use crate::{
    io::{
        export::{self, *},
        import::{self, *},
    },
    util::{
        digest::MediaDigest,
        format_parseable_value, format_replay_gain, parse_key_signature, parse_replay_gain,
        parse_tempo_bpm, parse_year_tag, push_next_actor_role_name, serato,
        tag::{
            import_faceted_tags_from_label_value_iter, import_plain_tags_from_joined_label_value,
            TagMappingConfig,
        },
        try_load_embedded_artwork,
    },
    Error, Result,
};

fn map_err(err: mp4ameta::Error) -> Error {
    let mp4ameta::Error { kind, description } = err;
    match kind {
        mp4ameta::ErrorKind::Io(err) => Error::Io(err),
        kind => Error::Other(anyhow::Error::from(mp4ameta::Error { kind, description })),
    }
}

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

const IDENT_ALBUM_ARTIST: Fourcc = Fourcc(*b"aART");

const IDENT_ARTIST: Fourcc = Fourcc(*b"\xA9ART");

const IDENT_COMMENT: Fourcc = Fourcc(*b"\xA9cmt");

const IDENT_COMPOSER: Fourcc = Fourcc(*b"\xA9wrt");

const IDENT_DIRECTOR: Fourcc = Fourcc(*b"\xA9dir");

const IDENT_GENRE: Fourcc = Fourcc(*b"\xA9gen");

const IDENT_GROUPING: Fourcc = Fourcc(*b"\xA9grp");

const IDENT_XID: Fourcc = Fourcc(*b"xid ");

const IDENT_BPM: FreeformIdent<'static> = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "BPM");

const IDENT_INITIAL_KEY: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "initialkey");
const KEY_IDENT: FreeformIdent<'static> = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "KEY");

const IDENT_REPLAYGAIN_TRACK_GAIN: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "replaygain_track_gain");

const IDENT_SUBTITLE: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "SUBTITLE");

const IDENT_CONDUCTOR: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "CONDUCTOR");

const IDENT_ENGINEER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "ENGINEER");

const IDENT_LYRICIST: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "LYRICIST");

const IDENT_MIXER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MIXER");

const IDENT_PRODUCER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "PRODUCER");

const IDENT_REMIXER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "REMIXER");

const IDENT_LABEL: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "LABEL");

const IDENT_MOOD: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MOOD");

const IDENT_ISRC: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "ISRC");

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
        reader: &mut Box<dyn Reader>,
        config: &ImportTrackConfig,
        track: &mut Track,
    ) -> Result<()> {
        // Extract metadata with mp4ameta
        let mut mp4_tag = match Mp4Tag::read_from(reader) {
            Ok(mp4_tag) => mp4_tag,
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
                .strings_of(&IDENT_REPLAYGAIN_TRACK_GAIN)
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
            .strings_of(&IDENT_BPM)
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
            .strings_of(&IDENT_INITIAL_KEY)
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
        if let Some(name) = mp4_tag.take_strings_of(&IDENT_SUBTITLE).next() {
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
        for name in mp4_tag.take_strings_of(&IDENT_PRODUCER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_REMIXER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_MIXER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Mixer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_ENGINEER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Engineer, name);
        }
        for name in mp4_tag.take_lyricists() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_CONDUCTOR) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_DIRECTOR) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Director, name);
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie_replace(Default::default());

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
        if let Some(label) = mp4_tag.take_strings_of(&IDENT_LABEL).next() {
            track.release.released_by = Some(label);
        }

        let mut tags_map = TagsMap::default();

        // Mixxx CustomTags
        if config.flags.contains(ImportTrackFlags::MIXXX_CUSTOM_TAGS) {
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

        // Genre tags (custom + standard)
        {
            // Prefer custom genre tags
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_GENRE.value());
            let mut next_score_value = TagScore::default_value();
            let mut plain_tags = Vec::with_capacity(8);
            if mp4_tag.custom_genres().next().is_some() {
                for genre in mp4_tag.take_custom_genres() {
                    import_plain_tags_from_joined_label_value(
                        tag_mapping_config,
                        &mut next_score_value,
                        &mut plain_tags,
                        genre,
                    );
                }
            }
            if plain_tags.is_empty() {
                // Import legacy/standard genres only as a fallback
                for genre_id in mp4_tag.standard_genres() {
                    let genre_id = usize::from(genre_id);
                    if genre_id < STANDARD_GENRES.len() {
                        let genre = STANDARD_GENRES[genre_id];
                        import_plain_tags_from_joined_label_value(
                            tag_mapping_config,
                            &mut next_score_value,
                            &mut plain_tags,
                            genre,
                        );
                    }
                }
            }
            tags_map.update_faceted_plain_tags_by_label_ordering(&FACET_GENRE, plain_tags);
        }

        // Mood tags
        import_faceted_tags_from_label_value_iter(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_MOOD,
            mp4_tag.take_strings_of(&IDENT_MOOD),
        );

        // Comment tag
        import_faceted_tags_from_label_value_iter(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_COMMENT,
            mp4_tag.take_comment().into_iter(),
        );

        // Grouping tags
        import_faceted_tags_from_label_value_iter(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_GROUPING,
            mp4_tag.take_groupings(),
        );

        // ISRC tag
        import_faceted_tags_from_label_value_iter(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ISRC,
            mp4_tag.take_isrc().into_iter(),
        );

        // iTunes XID tags
        import_faceted_tags_from_label_value_iter(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_XID,
            mp4_tag.take_strings_of(&IDENT_XID),
        );

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
        if config.flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
            let mut image_digest = if config.flags.contains(ImportTrackFlags::ARTWORK_DIGEST) {
                if config
                    .flags
                    .contains(ImportTrackFlags::ARTWORK_DIGEST_SHA256)
                {
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
        if config.flags.contains(ImportTrackFlags::SERATO_TAGS) {
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

        Ok(())
    }
}

fn export_filtered_actor_names(
    mp4_tag: &mut Mp4Tag,
    ident: impl Ident + Into<DataIdent>,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            mp4_tag.set_all_data(ident, once(Data::Utf8(name.to_owned())));
        }
        FilteredActorNames::Primary(names) => {
            mp4_tag.set_all_data(
                ident,
                names.into_iter().map(|name| Data::Utf8(name.to_owned())),
            );
        }
    }
}

fn export_faceted_tags(
    mp4_tag: &mut Mp4Tag,
    ident: impl Ident + Into<DataIdent>,
    config: Option<&TagMappingConfig>,
    tags: Vec<PlainTag>,
) {
    if let Some(config) = config {
        let joined_labels = config
            .join_labels_str_iter(
                tags.iter()
                    .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
            )
            .to_owned();
        mp4_tag.set_all_data(ident, joined_labels.map(|s| Data::Utf8(s.into())));
    } else {
        mp4_tag.set_all_data(
            ident,
            tags.into_iter().filter_map(|PlainTag { label, score: _ }| {
                label.map(|label| Data::Utf8(label.into()))
            }),
        );
    }
}

#[derive(Debug)]
pub struct ExportTrack;

impl export::ExportTrack for ExportTrack {
    fn export_track_to_path(
        &self,
        config: &ExportTrackConfig,
        track: &Track,
        path: &Path,
    ) -> Result<bool> {
        let mp4_tag_orig = Mp4Tag::read_from_path(path).map_err(map_err)?;

        let mut mp4_tag = mp4_tag_orig.clone();

        // Audio properties
        match &track.media_source.content {
            Content::Audio(audio) => {
                if let Some(loudness) = audio
                    .loudness
                    .map(LoudnessLufs::validated_from)
                    .transpose()
                    .ok()
                    .flatten()
                {
                    mp4_tag.set_all_data(
                        IDENT_REPLAYGAIN_TRACK_GAIN,
                        once(Data::Utf8(format_replay_gain(loudness))),
                    );
                } else {
                    mp4_tag.remove_data_of(&IDENT_REPLAYGAIN_TRACK_GAIN);
                }
                if let Some(encoder) = &audio.encoder {
                    mp4_tag.set_encoder(encoder);
                }
            }
        }

        // Music: Tempo/BPM
        if let Some(tempo_bpm) = track
            .metrics
            .tempo_bpm
            .map(TempoBpm::validated_from)
            .transpose()
            .ok()
            .flatten()
        {
            let mut bpm_value = tempo_bpm.0;
            mp4_tag.set_all_data(
                IDENT_BPM,
                once(Data::Utf8(format_parseable_value(&mut bpm_value))),
            );
            mp4_tag.set_bpm(bpm_value.round().max(u16::MAX as Beats) as u16);
        } else {
            mp4_tag.remove_bpm();
            mp4_tag.remove_data_of(&IDENT_BPM);
        }

        // Music: Key
        if track.metrics.key_signature.is_unknown() {
            mp4_tag.remove_data_of(&IDENT_INITIAL_KEY);
            mp4_tag.remove_data_of(&KEY_IDENT);
        } else {
            // TODO: Write a custom key code string according to config
            mp4_tag.set_all_data(
                IDENT_INITIAL_KEY,
                once(Data::Utf8(track.metrics.key_signature.to_string())),
            );
            if mp4_tag.data_of(&KEY_IDENT).next().is_some() {
                // Write non-standard key atom only if already present
                mp4_tag.set_all_data(
                    KEY_IDENT,
                    once(Data::Utf8(track.metrics.key_signature.to_string())),
                );
            }
        }

        // Track titles
        if let Some(title) = Titles::main_title(track.titles.iter()) {
            mp4_tag.set_title(title.name.to_owned());
        } else {
            mp4_tag.remove_title();
        }
        let track_subtitles = Titles::filter_kind(track.titles.iter(), TitleKind::Sub).peekable();
        mp4_tag.set_all_data(
            IDENT_SUBTITLE,
            track_subtitles.map(|subtitle| Data::Utf8(subtitle.name.to_owned())),
        );
        let mut track_movements =
            Titles::filter_kind(track.titles.iter(), TitleKind::Movement).peekable();
        if track_movements.peek().is_some() {
            let movement = track_movements.next().unwrap();
            // Only a single movement is supported
            debug_assert!(track_movements.peek().is_none());
            mp4_tag.set_movement(movement.name.to_owned());
        } else {
            mp4_tag.remove_movement();
        }
        let mut track_works = Titles::filter_kind(track.titles.iter(), TitleKind::Work).peekable();
        if track_works.peek().is_some() {
            let work = track_works.next().unwrap();
            // Only a single work is supported
            debug_assert!(track_works.peek().is_none());
            mp4_tag.set_work(work.name.to_owned());
        } else {
            mp4_tag.remove_work();
        }

        // Track actors
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_ARTIST,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_COMPOSER,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_DIRECTOR,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_LYRICIST,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_CONDUCTOR,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_ENGINEER,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Engineer),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_MIXER,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Mixer),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_PRODUCER,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Producer),
        );
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_REMIXER,
            FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
        );

        // Album
        if let Some(title) = Titles::main_title(track.album.titles.iter()) {
            mp4_tag.set_album(title.name.to_owned());
        } else {
            mp4_tag.remove_album();
        }
        export_filtered_actor_names(
            &mut mp4_tag,
            IDENT_ALBUM_ARTIST,
            FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
        );
        match track.album.kind {
            AlbumKind::Unknown => {
                mp4_tag.remove_compilation();
            }
            AlbumKind::Compilation => {
                mp4_tag.set_compilation();
            }
            AlbumKind::Album | AlbumKind::Single => {
                // TODO: Set compilation flag to false!?
                mp4_tag.remove_compilation();
            }
        }

        // Release
        if let Some(copyright) = &track.release.copyright {
            mp4_tag.set_copyright(copyright);
        } else {
            mp4_tag.remove_copyright();
        }
        if let Some(released_by) = &track.release.released_by {
            mp4_tag.set_all_data(IDENT_LABEL, once(Data::Utf8(released_by.to_owned())));
        } else {
            mp4_tag.remove_data_of(&IDENT_LABEL);
        }
        if let Some(released_at) = &track.release.released_at {
            mp4_tag.set_year(released_at.to_string());
        } else {
            mp4_tag.remove_year();
        }

        // Numbers
        if let Some(track_number) = track.indexes.track.number {
            mp4_tag.set_track_number(track_number);
        } else {
            mp4_tag.remove_track_number();
        }
        if let Some(track_total) = track.indexes.track.total {
            mp4_tag.set_total_tracks(track_total);
        } else {
            mp4_tag.remove_total_tracks();
        }
        if let Some(disc_number) = track.indexes.disc.number {
            mp4_tag.set_disc_number(disc_number);
        } else {
            mp4_tag.remove_disc_number();
        }
        if let Some(disc_total) = track.indexes.disc.total {
            mp4_tag.set_total_discs(disc_total);
        } else {
            mp4_tag.remove_total_discs();
        }
        if let Some(movement_number) = track.indexes.movement.number {
            mp4_tag.set_movement_index(movement_number);
        } else {
            mp4_tag.remove_movement_index();
        }
        if let Some(movement_total) = track.indexes.movement.total {
            mp4_tag.set_movement_count(movement_total);
        } else {
            mp4_tag.remove_movement_count();
        }

        let mut tags_map = TagsMap::from(track.tags.clone().untie());

        // Genre(s)
        if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_GENRE) {
            // Overwrite standard genres with custom genres
            mp4_tag.remove_standard_genres();
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_GENRE,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        } else {
            // Preserve standard genres until overwritten by custom genres
            mp4_tag.remove_data_of(&IDENT_GENRE);
        }

        // Comment(s)
        if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_COMMENT) {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_COMMENT,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        } else {
            mp4_tag.remove_data_of(&IDENT_COMMENT);
        }

        // Grouping(s)
        if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_GROUPING) {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_GROUPING,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        } else {
            mp4_tag.remove_data_of(&IDENT_GROUPING);
        }

        // Mood(s)
        if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_MOOD) {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_MOOD,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        } else {
            mp4_tag.remove_data_of(&IDENT_MOOD);
        }

        // ISRC(s)
        if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ISRC) {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_ISRC,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        } else {
            mp4_tag.remove_data_of(&IDENT_ISRC);
        }

        // XID(s)
        if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_XID) {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_XID,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        } else {
            mp4_tag.remove_data_of(&IDENT_XID);
        }

        if mp4_tag == mp4_tag_orig {
            // Unmodified
            return Ok(false);
        }
        mp4_tag.write_to_path(path).map_err(map_err)?;
        // Modified
        Ok(true)
    }
}
