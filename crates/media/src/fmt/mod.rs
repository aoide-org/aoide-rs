// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, ops::Not as _};

use lofty::{
    Accessor, AudioFile, FileProperties, ItemKey, ItemValue, MergeTag, MimeType, ParseOptions,
    Picture, PictureType, SplitTag, Tag, TagItem, TagType, TaggedFile, TaggedFileExt as _,
};

use aoide_core::{
    audio::{
        channel::ChannelCount,
        signal::{BitrateBps, SampleRateHz},
        ChannelFlags, Channels, DurationMs,
    },
    media::{
        artwork::{ApicType, Artwork, ArtworkImage, EmbeddedArtwork},
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    music::tempo::TempoBpm,
    prelude::*,
    tag::{FacetKey, FacetedTags, Label, PlainTag, Score as TagScore, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        index::Index,
        metric::MetricsFlags,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MBID_RECORDING, FACET_ID_MBID_RELEASE,
            FACET_ID_MBID_RELEASE_GROUP, FACET_ID_MBID_TRACK, FACET_ID_MOOD, FACET_ID_XID,
        },
        title::{Kind as TitleKind, Titles},
        Track,
    },
    util::string::trimmed_non_empty_from,
};

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    },
    util::{
        artwork::{
            try_ingest_embedded_artwork_image, EditEmbeddedArtworkImage,
            EditOtherEmbeddedArtworkImages, RemoveEmbeddedArtworkImage,
            ReplaceEmbeddedArtworkImage,
        },
        digest::MediaDigest,
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from,
        key_signature_as_str, push_next_actor_role_name_from,
        tag::TagMappingConfig,
        TempoBpmFormat,
    },
};

pub(crate) mod aiff;

pub(crate) mod flac;

pub(crate) mod id3v2;

pub(crate) mod mp4;

pub(crate) mod mpeg;

pub(crate) mod ogg;

pub(crate) mod opus;

pub(crate) mod vorbis;

const ENCODER_FIELD_SEPARATOR: &str = "|";

pub(crate) fn parse_options() -> ParseOptions {
    ParseOptions::new()
        .parsing_mode(lofty::ParsingMode::Relaxed)
        .read_properties(true)
}

fn import_audio_content_from_file_properties(properties: &FileProperties) -> AudioContentMetadata {
    let bitrate = properties
        .audio_bitrate()
        .map(|kbps| BitrateBps::new(f64::from(kbps) * 1000.0))
        .filter(IsValid::is_valid);
    let channel_count = properties
        .channels()
        .map(|count| ChannelCount(count.into()));
    let channel_flags = properties
        .channel_mask()
        .map(|mask| ChannelFlags::from_bits_truncate(mask.bits()));
    let channels = Channels::try_from_flags_or_count(channel_flags, channel_count);
    let duration_ms = properties.duration().as_secs_f64() * 1000.0;
    let duration = Some(DurationMs::new(duration_ms)).filter(IsValid::is_valid);
    let sample_rate = properties
        .sample_rate()
        .map(|hz| SampleRateHz::new(hz.into()))
        .filter(IsValid::is_valid);
    AudioContentMetadata {
        bitrate,
        channels,
        duration,
        sample_rate,
        encoder: None,
        loudness: None,
    }
}

pub(crate) fn take_primary_or_first_tag(tagged_file: &mut TaggedFile) -> Option<Tag> {
    if let Some(tag) = tagged_file.remove(tagged_file.primary_tag_type()) {
        return Some(tag);
    }
    let Some(first_tag_type) = tagged_file.first_tag().map(Tag::tag_type) else {
        return None;
    };
    tagged_file.remove(first_tag_type)
}

fn apic_type_from_picture_type(picture_type: PictureType) -> Option<ApicType> {
    let apic_type = match picture_type {
        PictureType::Artist => ApicType::Artist,
        PictureType::Band => ApicType::Band,
        PictureType::BandLogo => ApicType::BandLogo,
        PictureType::BrightFish => ApicType::BrightFish,
        PictureType::Composer => ApicType::Composer,
        PictureType::Conductor => ApicType::Conductor,
        PictureType::CoverBack => ApicType::CoverBack,
        PictureType::CoverFront => ApicType::CoverFront,
        PictureType::DuringPerformance => ApicType::DuringPerformance,
        PictureType::DuringRecording => ApicType::DuringRecording,
        PictureType::Icon => ApicType::Icon,
        PictureType::Illustration => ApicType::Illustration,
        PictureType::LeadArtist => ApicType::LeadArtist,
        PictureType::Leaflet => ApicType::Leaflet,
        PictureType::Lyricist => ApicType::Lyricist,
        PictureType::Media => ApicType::Media,
        PictureType::Other => ApicType::Other,
        PictureType::OtherIcon => ApicType::OtherIcon,
        PictureType::PublisherLogo => ApicType::PublisherLogo,
        PictureType::RecordingLocation => ApicType::RecordingLocation,
        PictureType::ScreenCapture => ApicType::ScreenCapture,
        PictureType::Undefined(_) => {
            return None;
        }
        _ => {
            // non-exhaustive enum
            log::error!("Unhandled picture type: {picture_type:?}");
            return None;
        }
    };
    Some(apic_type)
}

fn picture_type_from_apic_type(apic_type: ApicType) -> PictureType {
    match apic_type {
        ApicType::Artist => PictureType::Artist,
        ApicType::Band => PictureType::Band,
        ApicType::BandLogo => PictureType::BandLogo,
        ApicType::BrightFish => PictureType::BrightFish,
        ApicType::Composer => PictureType::Composer,
        ApicType::Conductor => PictureType::Conductor,
        ApicType::CoverBack => PictureType::CoverBack,
        ApicType::CoverFront => PictureType::CoverFront,
        ApicType::DuringPerformance => PictureType::DuringPerformance,
        ApicType::DuringRecording => PictureType::DuringRecording,
        ApicType::Icon => PictureType::Icon,
        ApicType::Illustration => PictureType::Illustration,
        ApicType::LeadArtist => PictureType::LeadArtist,
        ApicType::Leaflet => PictureType::Leaflet,
        ApicType::Lyricist => PictureType::Lyricist,
        ApicType::Media => PictureType::Media,
        ApicType::Other => PictureType::Other,
        ApicType::OtherIcon => PictureType::OtherIcon,
        ApicType::PublisherLogo => PictureType::PublisherLogo,
        ApicType::RecordingLocation => PictureType::RecordingLocation,
        ApicType::ScreenCapture => PictureType::ScreenCapture,
    }
}

#[must_use]
pub(crate) fn find_embedded_artwork_image(tag: &Tag) -> Option<(ApicType, &str, &[u8])> {
    tag.pictures()
        .iter()
        .filter_map(|p| {
            if p.pic_type() == PictureType::CoverFront {
                Some((ApicType::CoverFront, p))
            } else {
                None
            }
        })
        .chain(tag.pictures().iter().filter_map(|p| {
            if p.pic_type() == PictureType::Media {
                Some((ApicType::Media, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().iter().filter_map(|p| {
            if p.pic_type() == PictureType::Leaflet {
                Some((ApicType::Leaflet, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().iter().filter_map(|p| {
            if p.pic_type() == PictureType::Other {
                Some((ApicType::Other, p))
            } else {
                None
            }
        }))
        // otherwise take the first picture that could be parsed
        .chain(tag.pictures().iter().map(|p| {
            (
                apic_type_from_picture_type(p.pic_type()).unwrap_or(ApicType::Other),
                p,
            )
        }))
        .map(|(apic_type, p)| (apic_type, p.mime_type().as_str(), p.data()))
        .next()
}

pub(crate) fn import_embedded_artwork(
    importer: &mut Importer,
    tag: &Tag,
    mut media_digest: MediaDigest,
) -> Artwork {
    let artwork = if let Some((apic_type, mime_type, image_data)) = find_embedded_artwork_image(tag)
    {
        let (artwork, _, issues) = try_ingest_embedded_artwork_image(
            apic_type,
            image_data,
            None,
            Some(mime_type),
            &mut media_digest,
        );
        for issue in issues {
            importer.add_issue(issue);
        }
        artwork
    } else {
        Artwork::Missing
    };
    artwork
}

pub(crate) fn import_tagged_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    mut tagged_file: TaggedFile,
    track: &mut Track,
) {
    let tag = take_primary_or_first_tag(&mut tagged_file);
    if let Some(tag) = tag {
        log::debug!(
            "Importing track metadata from {tag_type:?} tag in {file_type:?} file",
            tag_type = tag.tag_type(),
            file_type = tagged_file.file_type(),
        );
        let file_properties = tagged_file.properties();
        import_file_tag_into_track(importer, config, file_properties, tag, track);
    }
}

// Compatibility hacks for mapping ItemKey::ContentGroup and ItemKey::Work
#[derive(Debug)]
struct Compatibility {
    primary_content_group_item_key: ItemKey,
    secondary_content_group_item_key: Option<ItemKey>,
    primary_work_item_key: ItemKey,
    secondary_work_item_key: Option<ItemKey>,
}

impl Compatibility {
    fn import(tage_type: TagType, flags: ImportTrackFlags) -> Self {
        Self::new(
            tage_type,
            flags.contains(ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        )
    }

    fn export(tage_type: TagType, flags: ExportTrackFlags) -> Self {
        Self::new(
            tage_type,
            flags.contains(ExportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        )
    }

    fn new(tage_type: TagType, apple_grp1: bool) -> Self {
        let primary_content_group_item_key;
        let secondary_content_group_item_key;
        let primary_work_item_key;
        let secondary_work_item_key;
        if matches!(tage_type, TagType::ID3v2) {
            primary_content_group_item_key = ItemKey::AppleId3v2ContentGroup; // GRP1
            primary_work_item_key = ItemKey::Work; // TXXX:WORK
            if apple_grp1 {
                secondary_content_group_item_key = None;
                secondary_work_item_key = Some(ItemKey::ContentGroup); // TIT1
            } else {
                secondary_content_group_item_key = Some(ItemKey::ContentGroup); // TIT1
                secondary_work_item_key = None;
            }
        } else {
            primary_content_group_item_key = ItemKey::ContentGroup;
            secondary_content_group_item_key = None;
            primary_work_item_key = ItemKey::Work;
            secondary_work_item_key = None;
        }
        Self {
            primary_content_group_item_key,
            secondary_content_group_item_key,
            primary_work_item_key,
            secondary_work_item_key,
        }
    }
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn import_file_tag_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    file_properties: &FileProperties,
    mut tag: Tag,
    track: &mut Track,
) {
    let audio_content = track
        .media_source
        .content
        .metadata_flags
        .update(ContentMetadataFlags::UNRELIABLE)
        .then(|| import_audio_content_from_file_properties(file_properties));
    if let Some(mut audio_content) = audio_content {
        // Import the remaining audio content properties
        debug_assert!(audio_content.encoder.is_none());
        let encoder_info: Vec<_> = [
            tag.get_string(&ItemKey::EncodedBy),
            tag.get_string(&ItemKey::EncoderSoftware),
            tag.get_string(&ItemKey::EncoderSettings),
        ]
        .into_iter()
        .flatten()
        .collect();
        audio_content.encoder = encoder_info
            .is_empty()
            .not()
            .then(|| encoder_info.join(ENCODER_FIELD_SEPARATOR));
        debug_assert!(audio_content.loudness.is_none());
        audio_content.loudness = tag
            .get_string(&ItemKey::ReplayGainTrackGain)
            .and_then(|input| importer.import_loudness_from_replay_gain(input));
        let new_metadata = ContentMetadata::Audio(audio_content);
        let old_metadata = &mut track.media_source.content.metadata;
        if *old_metadata != new_metadata {
            log::debug!("Updating content metadata: {old_metadata:?} -> {new_metadata:?}");
        }
        *old_metadata = new_metadata;
    }

    if !config.flags.contains(ImportTrackFlags::METADATA) {
        log::debug!("Skipping import of metadata");
        return;
    }

    let compatibility = Compatibility::import(tag.tag_type(), config.flags);

    // Musical metrics: tempo (bpm)
    for imported_tempo_bpm in tag
        .take_strings(&ItemKey::BPM)
        .filter_map(|input| importer.import_tempo_bpm(&input))
    {
        let is_non_fractional = imported_tempo_bpm.is_non_fractional();
        if is_non_fractional
            && track.metrics.tempo_bpm.is_some()
            && !track
                .metrics
                .flags
                .contains(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL)
        {
            // Preserve the existing fractional bpm and continue, trying
            // to import a more precise, fractional bpm.
            continue;
        }
        let old_tempo_bpm = &mut track.metrics.tempo_bpm;
        let new_tempo_bpm = TempoBpm::from(imported_tempo_bpm);
        if let Some(old_tempo_bpm) = old_tempo_bpm {
            if *old_tempo_bpm != new_tempo_bpm {
                log::debug!("Replacing tempo: {old_tempo_bpm} -> {new_tempo_bpm}");
            }
        }
        *old_tempo_bpm = Some(new_tempo_bpm);
        track
            .metrics
            .flags
            .set(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL, is_non_fractional);
        if !is_non_fractional {
            // Abort after importing the first fractional bpm
            break;
        }
        // Continue and try to import a more precise, fractional bpm.
    }

    // Musical metrics: key signature
    let new_key_signature = tag
        .take_strings(&ItemKey::InitialKey)
        .find_map(|input| importer.import_key_signature(&input));
    if let Some(old_key_signature) = track.metrics.key_signature {
        if let Some(new_key_signature) = new_key_signature {
            if old_key_signature != new_key_signature {
                log::debug!("Replacing key signature: {old_key_signature} -> {new_key_signature}");
            }
        } else {
            log::debug!("Removing key signature: {old_key_signature}");
        }
    }
    track.metrics.key_signature = new_key_signature;

    // Track titles
    let mut track_titles = Vec::with_capacity(4);
    if let Some(title) = tag
        .take_strings(&ItemKey::TrackTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Main))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag
        .take_strings(&ItemKey::TrackTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Sub))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag
        .take_strings(&ItemKey::Movement)
        .find_map(|name| ingest_title_from(name, TitleKind::Movement))
    {
        track_titles.push(title);
    }
    let primary_work_title = tag
        .take_strings(&compatibility.primary_work_item_key)
        .find_map(|name| ingest_title_from(name, TitleKind::Work));
    if let Some(work_title) = primary_work_title.or_else(|| {
        compatibility
            .secondary_work_item_key
            .and_then(|secondary_work_item_key| {
                tag.take_strings(&secondary_work_item_key)
                    .find_map(|name| ingest_title_from(name, TitleKind::Work))
            })
    }) {
        track_titles.push(work_title);
    }
    let new_track_titles = importer.finish_import_of_titles(TrackScope::Track, track_titles);
    let old_track_titles = &mut track.titles;
    if !old_track_titles.is_empty() && *old_track_titles != new_track_titles {
        log::debug!("Replacing track titles: {old_track_titles:?} -> {new_track_titles:?}");
    }
    *old_track_titles = new_track_titles;

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    for name in tag.take_strings(&ItemKey::TrackArtist) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
    }
    for name in tag.take_strings(&ItemKey::Arranger) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Arranger, name);
    }
    for name in tag.take_strings(&ItemKey::Composer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
    }
    for name in tag.take_strings(&ItemKey::Conductor) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
    }
    for name in tag.take_strings(&ItemKey::Director) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
    }
    for name in tag.take_strings(&ItemKey::Engineer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Engineer, name);
    }
    for name in tag.take_strings(&ItemKey::Lyricist) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
    }
    for name in tag.take_strings(&ItemKey::MixDj) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::MixDj, name);
    }
    for name in tag.take_strings(&ItemKey::MixEngineer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::MixEngineer, name);
    }
    for name in tag.take_strings(&ItemKey::Performer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Performer, name);
    }
    for name in tag.take_strings(&ItemKey::Producer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Producer, name);
    }
    for name in tag.take_strings(&ItemKey::Writer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
    }
    let new_track_actors = importer.finish_import_of_actors(TrackScope::Track, track_actors);
    let old_track_actors = &mut track.actors;
    if !old_track_actors.is_empty() && *old_track_actors != new_track_actors {
        log::debug!("Replacing track actors: {old_track_actors:?} -> {new_track_actors:?}");
    }
    *old_track_actors = new_track_actors;

    let mut album = std::mem::take(&mut track.album).untie();

    // Album titles
    let mut album_titles = Vec::with_capacity(1);
    if let Some(title) = tag
        .take_strings(&ItemKey::AlbumTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Main))
    {
        album_titles.push(title);
    }
    let new_album_titles = importer.finish_import_of_titles(TrackScope::Album, album_titles);
    let old_album_titles = &mut album.titles;
    if !old_album_titles.is_empty() && *old_album_titles != new_album_titles {
        log::debug!("Replacing album titles: {old_album_titles:?} -> {new_album_titles:?}");
    }
    *old_album_titles = new_album_titles;

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    for name in tag.take_strings(&ItemKey::AlbumArtist) {
        push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
    }
    let new_album_actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);
    let old_album_actors = &mut album.actors;
    if !old_album_actors.is_empty() && *old_album_actors != new_album_actors {
        log::debug!("Replacing album actors: {old_album_actors:?} -> {new_album_actors:?}");
    }
    *old_album_actors = new_album_actors;

    if let Some(item) = tag.take(&ItemKey::FlagCompilation).next() {
        if let Some(kind) = item
            .value()
            .text()
            .and_then(|input| input.parse::<u8>().ok())
            .and_then(|value| match value {
                0 => Some(AlbumKind::NoCompilation),
                1 => Some(AlbumKind::Compilation),
                _ => None,
            })
        {
            album.kind = Some(kind);
        } else {
            importer.add_issue(format!("Unexpected compilation flag item: {item:?}"));
        }
    }

    let new_album = Canonical::tie(album);
    let old_album = &mut track.album;
    if *old_album != Default::default() && *old_album != new_album {
        log::debug!("Replacing album: {old_album:?} -> {new_album:?}");
    }
    *old_album = new_album;

    let new_copyright = tag
        .take_strings(&ItemKey::CopyrightMessage)
        .find_map(trimmed_non_empty_from)
        .map(Cow::into_owned);
    let old_copyright = &mut track.copyright;
    if old_copyright.is_some() && *old_copyright != new_copyright {
        log::debug!("Replacing copyright: {old_copyright:?} -> {new_copyright:?}");
    }
    *old_copyright = new_copyright;

    let old_publisher = &mut track.publisher;
    let mut new_publisher = tag
        .take_strings(&ItemKey::Label)
        .find_map(trimmed_non_empty_from)
        .map(Cow::into_owned);
    if new_publisher.is_none() {
        new_publisher = tag
            .take_strings(&ItemKey::Publisher)
            .find_map(trimmed_non_empty_from)
            .map(Cow::into_owned);
    }
    if old_publisher.is_some() && *old_publisher != new_publisher {
        log::debug!("Replacing publisher: {old_publisher:?} -> {new_publisher:?}");
    }
    *old_publisher = new_publisher;

    // Index pairs
    // Import both values consistently if any of them is available!
    // TODO: Verify u32 -> u16 conversions
    let old_track_index = &mut track.indexes.track;
    let track_number = tag
        .track()
        .map(TryFrom::try_from)
        .transpose()
        .ok()
        .flatten();
    let track_total = tag
        .track_total()
        .map(TryFrom::try_from)
        .transpose()
        .ok()
        .flatten();
    if track_number.is_some() || track_total.is_some() {
        let new_track_index = Index {
            number: track_number,
            total: track_total,
        };
        if *old_track_index != Default::default() && *old_track_index != new_track_index {
            log::debug!("Replacing track index: {old_track_index:?} -> {new_track_index:?}");
        }
        *old_track_index = new_track_index;
    } else {
        if *old_track_index != Default::default() {
            log::debug!("Resetting track index: {old_track_index:?}");
        }
        *old_track_index = Default::default();
    }
    let old_disc_index = &mut track.indexes.track;
    let disc_number = tag.disk().map(TryFrom::try_from).transpose().ok().flatten();
    let disc_total = tag
        .disk_total()
        .map(TryFrom::try_from)
        .transpose()
        .ok()
        .flatten();
    if disc_number.is_some() || disc_total.is_some() {
        let new_disc_index = Index {
            number: disc_number,
            total: disc_total,
        };
        if *old_disc_index != Default::default() && *old_disc_index != new_disc_index {
            log::debug!("Replacing disc index: {old_disc_index:?} -> {new_disc_index:?}");
        }
        *old_disc_index = new_disc_index;
    } else {
        if *old_disc_index != Default::default() {
            log::debug!("Resetting disc index: {old_disc_index:?}");
        }
        *old_disc_index = Default::default();
    }
    let old_movement_index = &mut track.indexes.movement;
    let movement_number =
        tag.get_items(&ItemKey::MovementNumber)
            .find_map(|item| match item.value() {
                ItemValue::Text(number) => number.parse::<u16>().ok(),
                _ => None,
            });
    let movement_total =
        tag.get_items(&ItemKey::MovementNumber)
            .find_map(|item| match item.value() {
                ItemValue::Text(number) => number.parse::<u16>().ok(),
                _ => None,
            });
    if movement_number.is_some() || movement_total.is_some() {
        let new_movement_index = Index {
            number: movement_number,
            total: movement_total,
        };
        if *old_movement_index != Default::default() && *old_movement_index != new_movement_index {
            log::debug!(
                "Replacing movement index: {old_movement_index:?} -> {new_movement_index:?}"
            );
        }
        *old_movement_index = new_movement_index;
    } else {
        if *old_movement_index != Default::default() {
            log::debug!("Resetting movement index: {old_movement_index:?}");
        }
        *old_movement_index = Default::default();
    }

    let old_recorded_at = &mut track.recorded_at;
    let mut new_recorded_at = tag
        .take_strings(&ItemKey::RecordingDate)
        .find_map(|input| importer.import_year_tag_from_field("RecordingDate", &input));
    if new_recorded_at.is_none() {
        new_recorded_at = tag
            .take_strings(&ItemKey::Year)
            .find_map(|input| importer.import_year_tag_from_field("Year", &input));
    }
    if old_recorded_at.is_some() && *old_recorded_at != new_recorded_at {
        log::debug!("Replacing recorded at: {old_recorded_at:?} -> {new_recorded_at:?}");
    }
    *old_recorded_at = new_recorded_at;

    let old_released_at = &mut track.released_at;
    let new_released_at = tag
        .take_strings(&ItemKey::PodcastReleaseDate)
        .find_map(|input| importer.import_year_tag_from_field("PodcastReleaseDate", &input));
    if old_released_at.is_some() && *old_released_at != new_released_at {
        log::debug!("Replacing released at: {old_released_at:?} -> {new_released_at:?}");
    }
    *old_released_at = new_released_at;

    let old_released_orig_at = &mut track.released_orig_at;
    let new_released_orig_at = tag
        .take_strings(&ItemKey::OriginalReleaseDate)
        .find_map(|input| importer.import_year_tag_from_field("OriginalReleaseDate", &input));
    if old_released_orig_at.is_some() && *old_released_orig_at != new_released_orig_at {
        log::debug!(
            "Replacing original released at: {old_released_orig_at:?} -> {new_released_orig_at:?}"
        );
    }
    *old_released_orig_at = new_released_orig_at;

    let mut tags_map: TagsMap<'static> = Default::default();

    // Grouping tags
    debug_assert!(tags_map.get_faceted_plain_tags(FACET_ID_GROUPING).is_none());
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_GROUPING,
        tag.take_strings(&compatibility.primary_content_group_item_key)
            .map(Into::into),
    );
    if let Some(secondary_content_group_item_key) = compatibility.secondary_content_group_item_key {
        if tags_map.get_faceted_plain_tags(FACET_ID_GROUPING).is_none() {
            importer.import_faceted_tags_from_label_values(
                &mut tags_map,
                &config.faceted_tag_mapping,
                FACET_ID_GROUPING,
                tag.take_strings(&secondary_content_group_item_key)
                    .map(Into::into),
            );
        }
    }

    // Import gig tags from raw grouping tags before any other tags.
    #[cfg(feature = "gigtag")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(FACET_ID_GROUPING) {
            tags_map = crate::util::gigtag::import_from_faceted_tags(faceted_tags);
        }
    }

    // Genre tags
    {
        let tag_mapping_config = config.faceted_tag_mapping.get(FACET_ID_GENRE.as_str());
        let mut next_score_value = TagScore::default_value();
        let mut plain_tags = Vec::with_capacity(8);
        for genre in tag.take_strings(&ItemKey::Genre) {
            importer.import_plain_tags_from_joined_label_value(
                tag_mapping_config,
                &mut next_score_value,
                &mut plain_tags,
                genre,
            );
        }
        tags_map.update_faceted_plain_tags_by_label_ordering(FACET_ID_GENRE, plain_tags);
    }

    // Mood tags
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MOOD,
        tag.take_strings(&ItemKey::Mood).map(Into::into),
    );

    // Comment tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_COMMENT,
        tag.take_strings(&ItemKey::Comment).map(Into::into),
    );

    // Description tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_DESCRIPTION,
        tag.take_strings(&ItemKey::Description).map(Into::into),
    );

    // ISRC tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_ISRC,
        tag.take_strings(&ItemKey::ISRC).map(Into::into),
    );

    // XID tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_XID,
        tag.take_strings(&ItemKey::AppleXid).map(Into::into),
    );

    // MusicBrainz tags
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_RECORDING,
        tag.take_strings(&ItemKey::MusicBrainzRecordingId)
            .map(Into::into),
    );
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_TRACK,
        tag.take_strings(&ItemKey::MusicBrainzTrackId)
            .map(Into::into),
    );
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_RELEASE,
        tag.take_strings(&ItemKey::MusicBrainzReleaseId)
            .map(Into::into),
    );
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_RELEASE_GROUP,
        tag.take_strings(&ItemKey::MusicBrainzReleaseGroupId)
            .map(Into::into),
    );

    let old_tags = &mut track.tags;
    let new_tags = tags_map.canonicalize_into();
    if !old_tags.is_empty() && *old_tags != new_tags {
        log::debug!("Replacing tags: {old_tags:?} -> {new_tags:?}");
    }
    *old_tags = new_tags;

    // Artwork
    if config
        .flags
        .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
    {
        let new_artwork =
            import_embedded_artwork(importer, &tag, config.flags.new_artwork_digest());
        if let Some(old_artwork) = &track.media_source.artwork {
            if *old_artwork != new_artwork {
                log::debug!("Replacing artwork: {old_artwork:?} -> {new_artwork:?}");
            }
        }
        track.media_source.artwork = Some(new_artwork);
    } else {
        log::debug!("Skipping import of embedded artwork");
    }
}

#[cfg(feature = "serato-markers")]
pub(crate) fn import_serato_tags(track: &mut Track, serato_tags: &triseratops::tag::TagContainer) {
    let old_cues = &mut track.cues;
    let new_cues = crate::util::serato::import_cues(serato_tags);
    if !old_cues.is_empty() && *old_cues != new_cues {
        log::debug!("Replacing cues from Serato tags: {old_cues:?} -> {new_cues:?}");
    }
    *old_cues = new_cues;

    let old_color = &mut track.color;
    let new_color = crate::util::serato::import_track_color(serato_tags);
    if old_color.is_some() && *old_color != new_color {
        log::debug!("Replacing color from Serato tags: {old_color:?} -> {new_color:?}");
    }
    *old_color = new_color;
}

fn export_filtered_actor_names(
    tag: &mut Tag,
    item_key: ItemKey,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            tag.insert_text(item_key, name.to_owned());
        }
        FilteredActorNames::Individual(names) => {
            for name in names {
                let item_key = item_key.clone();
                let item_val = ItemValue::Text(name.to_owned());
                let pushed = tag.push_item(TagItem::new(item_key, item_val));
                if !pushed {
                    // Unsupported key
                    break;
                }
            }
        }
    }
}

fn export_faceted_tags(
    tag: &mut Tag,
    item_key: ItemKey,
    config: Option<&TagMappingConfig>,
    tags: impl IntoIterator<Item = PlainTag<'static>>,
) {
    if let Some(config) = config {
        let joined_labels = config.join_labels(
            tags.into_iter()
                .filter_map(|PlainTag { label, score: _ }| label.map(Label::into_inner)),
        );
        if let Some(joined_labels) = joined_labels {
            let inserted = tag.insert_text(item_key, joined_labels.into_owned());
            debug_assert!(inserted);
        } else {
            tag.remove_key(&item_key);
        }
    } else {
        tag.remove_key(&item_key);
        for label in tags
            .into_iter()
            .filter_map(|PlainTag { label, score: _ }| label)
        {
            let item_key = item_key.clone();
            let item_val = ItemValue::Text(label.into_inner().into_owned());
            let pushed = tag.push_item(TagItem::new(item_key, item_val));
            if !pushed {
                // Unsupported key
                break;
            }
        }
    }
}

fn split_export_merge_track_to_tag<T>(
    tag_repr: T,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> <T::Remainder as MergeTag>::Merged
where
    T: SplitTag,
{
    // Split the generic tag contents from the underlying representation.
    // The remainder will remain untouched while modifying the generic tag.
    let (tag_remainder, mut tag) = tag_repr.split_tag();
    // Export the metadata of the given track into the generic tag, i.e.
    // add, modify, or delete items and pictures accordingly.
    export_track_to_tag(&mut tag, config, track, edit_embedded_artwork_image);
    // Merge the generic tag contents back into the remainder of the underlying
    // representation.
    // Depending on `T` some post-processing might be required in the outer context
    // to update contents in `tag_repr` that are not (yet) supported by the generic
    // `lofty::Tag` representation.
    tag_remainder.merge_tag(tag)
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn export_track_to_tag(
    tag: &mut Tag,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) {
    // Audio properties
    match &track.media_source.content.metadata {
        ContentMetadata::Audio(audio) => {
            if let Some(track_gain_text) = audio.loudness.and_then(format_valid_replay_gain) {
                tag.insert_text(ItemKey::ReplayGainTrackGain, track_gain_text);
            } else {
                tag.remove_key(&ItemKey::ReplayGainTrackGain);
            }
            // The encoder is a read-only property.
        }
    }

    let compatibility = Compatibility::export(tag.tag_type(), config.flags);
    // Prevent inconsistencies by removing ambiguous keys.
    if let Some(secondary_content_group_item_key) = compatibility.secondary_content_group_item_key {
        tag.remove_key(&secondary_content_group_item_key);
    }
    if let Some(secondary_work_item_key) = compatibility.secondary_work_item_key {
        tag.remove_key(&secondary_work_item_key);
    }

    // Music: Tempo/BPM
    // Write the BPM rounded to an integer value as the least common denominator.
    // The precise BPM could be written into custom tag fields during post-processing.
    if let Some(bpm_text) =
        format_validated_tempo_bpm(&mut track.metrics.tempo_bpm, TempoBpmFormat::Integer)
    {
        tag.insert_text(ItemKey::BPM, bpm_text);
    } else {
        tag.remove_key(&ItemKey::BPM);
    }

    // Music: Key
    if let Some(key_signature) = track.metrics.key_signature {
        let key_text = key_signature_as_str(key_signature).to_owned();
        tag.insert_text(ItemKey::InitialKey, key_text);
    } else {
        tag.remove_key(&ItemKey::InitialKey);
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        tag.set_title(title.name.clone());
    } else {
        tag.remove_title();
    }
    tag.remove_key(&ItemKey::TrackSubtitle);
    for track_subtitle in Titles::filter_kind(track.titles.iter(), TitleKind::Sub).peekable() {
        let item_val = ItemValue::Text(track_subtitle.name.clone());
        let pushed = tag.push_item(TagItem::new(ItemKey::TrackSubtitle, item_val));
        debug_assert!(pushed);
    }
    for movement_title in Titles::filter_kind(track.titles.iter(), TitleKind::Movement).peekable() {
        let item_val = ItemValue::Text(movement_title.name.clone());
        let pushed = tag.push_item(TagItem::new(ItemKey::Movement, item_val));
        debug_assert!(pushed);
    }
    for work_title in Titles::filter_kind(track.titles.iter(), TitleKind::Work).peekable() {
        let item_val = ItemValue::Text(work_title.name.clone());
        let pushed = tag.push_item(TagItem::new(
            compatibility.primary_work_item_key.clone(),
            item_val,
        ));
        debug_assert!(pushed);
    }

    // Track actors
    export_filtered_actor_names(
        tag,
        ItemKey::TrackArtist,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Arranger,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Arranger),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Composer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Conductor,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Director,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Engineer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Engineer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::MixDj,
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixDj),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::MixEngineer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixEngineer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Lyricist,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Performer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Performer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Producer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Producer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Remixer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Writer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Writer),
    );

    // Album
    if let Some(title) = Titles::main_title(track.album.titles.iter()) {
        tag.set_album(title.name.clone());
    } else {
        tag.remove_album();
    }
    export_filtered_actor_names(
        tag,
        ItemKey::AlbumArtist,
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    if let Some(kind) = track.album.kind {
        match kind {
            AlbumKind::NoCompilation | AlbumKind::Album | AlbumKind::Single => {
                tag.insert_text(ItemKey::FlagCompilation, "0".to_owned());
            }
            AlbumKind::Compilation => {
                tag.insert_text(ItemKey::FlagCompilation, "1".to_owned());
            }
        }
    } else {
        tag.remove_key(&ItemKey::FlagCompilation);
    }

    if let Some(publisher) = &track.publisher {
        tag.insert_text(ItemKey::Label, publisher.clone());
    } else {
        tag.remove_key(&ItemKey::Label);
    }
    if let Some(copyright) = &track.copyright {
        tag.insert_text(ItemKey::CopyrightMessage, copyright.clone());
    } else {
        tag.remove_key(&ItemKey::CopyrightMessage);
    }

    // Index pairs
    if let Some(track_number) = track.indexes.track.number {
        tag.set_track(track_number.into());
    } else {
        tag.remove_track();
    }
    if let Some(track_total) = track.indexes.track.total {
        tag.set_track_total(track_total.into());
    } else {
        tag.remove_track_total();
    }
    if let Some(disc_number) = track.indexes.disc.number {
        tag.set_disk(disc_number.into());
    } else {
        tag.remove_disk();
    }
    if let Some(disc_total) = track.indexes.disc.total {
        tag.set_disk_total(disc_total.into());
    } else {
        tag.remove_disk_total();
    }
    if let Some(movement_number) = track.indexes.movement.number {
        tag.insert_text(ItemKey::MovementNumber, movement_number.to_string());
    } else {
        tag.remove_key(&ItemKey::MovementNumber);
    }
    if let Some(movement_total) = track.indexes.movement.total {
        tag.insert_text(ItemKey::MovementTotal, movement_total.to_string());
    } else {
        tag.remove_key(&ItemKey::MovementTotal);
    }

    if let Some(recorded_at) = track.recorded_at {
        // Modify ItemKey::Year before ItemKey::RecordingDate
        // because they may end up in the same tag field. The
        // year number should not overwrite the more specific
        // time stamp if available.
        let year = recorded_at.year();
        if year >= 0 {
            tag.set_year(year as _);
        } else {
            tag.remove_year();
        }
        let recorded_at_text = recorded_at.to_string();
        tag.insert_text(ItemKey::RecordingDate, recorded_at_text);
    } else {
        tag.remove_year();
        tag.remove_key(&ItemKey::RecordingDate);
    }
    if let Some(released_at) = track.released_at {
        let released_at_text = released_at.to_string();
        tag.insert_text(ItemKey::PodcastReleaseDate, released_at_text);
    } else {
        tag.remove_key(&ItemKey::PodcastReleaseDate);
    }
    if let Some(released_orig_at) = track.released_orig_at {
        let released_orig_at_text = released_orig_at.to_string();
        tag.insert_text(ItemKey::OriginalReleaseDate, released_orig_at_text);
    } else {
        tag.remove_key(&ItemKey::OriginalReleaseDate);
    }

    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_GENRE) {
        export_faceted_tags(
            tag,
            ItemKey::Genre,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags.into_iter(),
        );
    } else {
        tag.remove_key(&ItemKey::Genre);
    }

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_COMMENT) {
        export_faceted_tags(
            tag,
            ItemKey::Comment,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::Comment);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_DESCRIPTION) {
        export_faceted_tags(
            tag,
            ItemKey::Description,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::Description);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MOOD) {
        export_faceted_tags(
            tag,
            ItemKey::Mood,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::Mood);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_ISRC) {
        export_faceted_tags(
            tag,
            ItemKey::ISRC,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::ISRC);
    }

    // XID(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_XID) {
        export_faceted_tags(
            tag,
            ItemKey::AppleXid,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::AppleXid);
    }

    // MusicBrainz tags
    if let Some(FacetedTags { facet_id, tags }) =
        tags_map.take_faceted_tags(FACET_ID_MBID_RECORDING)
    {
        export_faceted_tags(
            tag,
            ItemKey::MusicBrainzRecordingId,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::MusicBrainzRecordingId);
    }
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MBID_TRACK) {
        export_faceted_tags(
            tag,
            ItemKey::MusicBrainzTrackId,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::MusicBrainzTrackId);
    }
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MBID_RELEASE)
    {
        export_faceted_tags(
            tag,
            ItemKey::MusicBrainzReleaseId,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::MusicBrainzReleaseId);
    }
    if let Some(FacetedTags { facet_id, tags }) =
        tags_map.take_faceted_tags(FACET_ID_MBID_RELEASE_GROUP)
    {
        export_faceted_tags(
            tag,
            ItemKey::MusicBrainzReleaseGroupId,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::MusicBrainzReleaseGroupId);
    }

    // Grouping(s)
    {
        let facet_id = FACET_ID_GROUPING;
        let mut tags = tags_map
            .take_faceted_tags(facet_id)
            .map(|FacetedTags { facet_id: _, tags }| tags)
            .unwrap_or_default();
        #[cfg(feature = "gigtag")]
        if config.flags.contains(ExportTrackFlags::GIGTAGS) {
            let remaining_tags = tags_map.canonicalize_into();
            if let Err(err) = crate::util::gigtag::export_and_encode_remaining_tags_into(
                remaining_tags.as_canonical_ref(),
                &mut tags,
            ) {
                log::error!("Failed to export gig tags: {err}");
            }
        }
        if tags.is_empty() {
            tag.remove_key(&compatibility.primary_content_group_item_key);
        } else {
            export_faceted_tags(
                tag,
                compatibility.primary_content_group_item_key,
                config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
                tags,
            );
        }
    }

    if let Some(edit_embedded_artwork_image) = edit_embedded_artwork_image {
        match edit_embedded_artwork_image {
            EditEmbeddedArtworkImage::Replace(replace_embedded_artwork_image) => {
                let ReplaceEmbeddedArtworkImage {
                    artwork_image,
                    image_data,
                    others,
                } = replace_embedded_artwork_image;
                let ArtworkImage {
                    apic_type,
                    media_type,
                    ..
                } = &artwork_image;
                let pic_type = picture_type_from_apic_type(*apic_type);
                let mime_type = match media_type.essence_str() {
                    "image/bmp" => MimeType::Bmp,
                    "image/gif" => MimeType::Gif,
                    "image/jpeg" => MimeType::Jpeg,
                    "image/png" => MimeType::Png,
                    "image/tiff" => MimeType::Tiff,
                    _ => MimeType::Unknown(media_type.to_string()),
                };
                track.media_source.artwork = Some(Artwork::Embedded(EmbeddedArtwork {
                    image: artwork_image,
                }));
                let picture = Picture::new_unchecked(pic_type, mime_type, None, image_data);
                match others {
                    EditOtherEmbeddedArtworkImages::Keep => {
                        tag.remove_picture_type(pic_type);
                    }
                    EditOtherEmbeddedArtworkImages::Remove => {
                        while !tag.pictures().is_empty() {
                            tag.remove_picture(tag.pictures().len() - 1);
                        }
                    }
                }
                tag.push_picture(picture);
            }
            EditEmbeddedArtworkImage::Remove(remove_embedded_artwork_image) => {
                let RemoveEmbeddedArtworkImage { apic_type, others } =
                    remove_embedded_artwork_image;
                let pic_type = picture_type_from_apic_type(apic_type);
                match others {
                    EditOtherEmbeddedArtworkImages::Keep => {
                        tag.remove_picture_type(pic_type);
                        if matches!(track.media_source.artwork, Some(Artwork::Embedded(EmbeddedArtwork { image: ArtworkImage { apic_type: old_apic_type, .. } })) if old_apic_type == apic_type)
                        {
                            track.media_source.artwork = None;
                        }
                    }
                    EditOtherEmbeddedArtworkImages::Remove => {
                        while !tag.pictures().is_empty() {
                            tag.remove_picture(tag.pictures().len() - 1);
                        }
                        if matches!(track.media_source.artwork, Some(Artwork::Embedded(_))) {
                            track.media_source.artwork = None;
                        }
                    }
                }
            }
        }
    }
}
