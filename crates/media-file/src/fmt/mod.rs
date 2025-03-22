// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, ops::Not as _};

use lofty::{
    config::ParseOptions,
    file::{AudioFile as _, TaggedFile, TaggedFileExt as _},
    picture::{MimeType, Picture, PictureType},
    properties::FileProperties,
    tag::{Accessor as _, ItemKey, ItemValue, MergeTag, SplitTag, Tag, TagItem, TagType},
};
use nonicle::{Canonical, CanonicalizeInto as _};
use semval::prelude::*;

use aoide_core::{
    audio::{
        BitrateBpsValue, ChannelFlags, Channels, DurationMs,
        channel::ChannelCount,
        signal::{BitrateBps, SampleRateHz},
    },
    media::{
        artwork::{ApicType, Artwork, ArtworkImage, EmbeddedArtwork},
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    music::tempo::TempoBpm,
    tag::{FacetId, FacetKey, FacetedTags, PlainTag, Tags, TagsMap},
    track::{
        AdvisoryRating, Track,
        actor::{Kind as ActorKind, Role as ActorRole},
        album::Kind as AlbumKind,
        index::Index,
        metric::MetricsFlags,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MBID_ARTIST, FACET_ID_MBID_RECORDING, FACET_ID_MBID_RELEASE,
            FACET_ID_MBID_RELEASE_ARTIST, FACET_ID_MBID_RELEASE_GROUP, FACET_ID_MBID_TRACK,
            FACET_ID_MBID_WORK, FACET_ID_MOOD, FACET_ID_XID,
        },
        title::{Kind as TitleKind, Titles},
    },
    util::string::trimmed_non_empty_from,
};

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    },
    util::{
        FormattedTempoBpm, TempoBpmFormat,
        artwork::{
            EditEmbeddedArtworkImage, EditOtherEmbeddedArtworkImages, RemoveEmbeddedArtworkImage,
            ReplaceEmbeddedArtworkImage, try_ingest_embedded_artwork_image,
        },
        digest::MediaDigest,
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from,
        key_signature_as_str, push_next_actor,
        tag::TagMappingConfig,
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

/// All facets that could be stored as native file tags.
const DEFAULT_FILE_TAG_MAPPING: [(&FacetId<'static>, &ItemKey); 14] = [
    (FACET_ID_COMMENT, &ItemKey::Comment),
    (FACET_ID_DESCRIPTION, &ItemKey::Description),
    (FACET_ID_GENRE, &ItemKey::Genre),
    (FACET_ID_GROUPING, &ItemKey::ContentGroup),
    (FACET_ID_ISRC, &ItemKey::Isrc),
    (FACET_ID_MBID_ARTIST, &ItemKey::MusicBrainzArtistId),
    (FACET_ID_MBID_RECORDING, &ItemKey::MusicBrainzRecordingId),
    (FACET_ID_MBID_RELEASE, &ItemKey::MusicBrainzReleaseId),
    (
        FACET_ID_MBID_RELEASE_ARTIST,
        &ItemKey::MusicBrainzReleaseArtistId,
    ),
    (
        FACET_ID_MBID_RELEASE_GROUP,
        &ItemKey::MusicBrainzReleaseGroupId,
    ),
    (FACET_ID_MBID_TRACK, &ItemKey::MusicBrainzTrackId),
    (FACET_ID_MBID_WORK, &ItemKey::MusicBrainzWorkId),
    (FACET_ID_MOOD, &ItemKey::Mood),
    (FACET_ID_XID, &ItemKey::AppleXid),
];

fn file_tag_facets_without<'a>(
    excluded_facet_id: &'a FacetId<'a>,
) -> impl Iterator<Item = &'a FacetId<'a>> + 'a + Clone {
    DEFAULT_FILE_TAG_MAPPING
        .iter()
        .copied()
        .filter_map(move |(facet_id, _)| {
            if facet_id == excluded_facet_id {
                None
            } else {
                Some(facet_id)
            }
        })
}

#[cfg(feature = "gigtag")]
pub fn encode_gig_tags(
    tags: &mut Canonical<Tags<'_>>,
    encoded_tags: &mut Vec<PlainTag<'_>>,
    facet_id: &FacetId<'_>,
) -> std::fmt::Result {
    use nonicle::Canonical;

    let mut remaining_tags = std::mem::take(tags).untie();
    let facets = remaining_tags.split_off_faceted_tags(
        &file_tag_facets_without(facet_id),
        DEFAULT_FILE_TAG_MAPPING.len(),
    );
    let remaining_tags = Canonical::tie(remaining_tags);
    *tags = Canonical::tie(Tags {
        facets,
        plain: Default::default(),
    });
    crate::util::gigtag::export_and_encode_tags_into(
        remaining_tags.as_canonical_ref(),
        encoded_tags,
    )
}

pub(crate) fn parse_options() -> ParseOptions {
    ParseOptions::new().read_properties(true)
}

fn import_audio_content_from_file_properties(properties: &FileProperties) -> AudioContentMetadata {
    let bitrate = properties
        .audio_bitrate()
        .map(|kbps| BitrateBps::new(BitrateBpsValue::from(kbps) * 1000.0))
        .filter(IsValid::is_valid);
    let channel_count = properties
        .channels()
        .map(|count| ChannelCount::new(count.into()));
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
    let first_tag_type = tagged_file.first_tag().map(Tag::tag_type)?;
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

const fn picture_type_from_apic_type(apic_type: ApicType) -> PictureType {
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
        .find_map(|(apic_type, p)| Some((apic_type, p.mime_type()?.as_str(), p.data())))
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
            "Importing track metadata from {tag_type:?} tag in {file_type:?} file \
             \"{content_path}\": {tag_items:?}",
            tag_type = tag.tag_type(),
            file_type = tagged_file.file_type(),
            content_path = track.media_source.content.link.path,
            tag_items = tag.items().collect::<Vec<_>>(),
        );
        let file_properties = tagged_file.properties();
        import_file_tag_into_track(importer, config, file_properties, tag, track);
    }
}

// Compatibility hacks for mapping ItemKey::ContentGroup and ItemKey::Work
#[derive(Debug)]
struct Compatibility {
    primary_content_group: ItemKey,
    secondary_content_group: Option<ItemKey>,
    primary_work: ItemKey,
    secondary_work: Option<ItemKey>,
}

impl Compatibility {
    const fn import(tage_type: TagType, flags: ImportTrackFlags) -> Self {
        Self::new(
            tage_type,
            flags.contains(ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        )
    }

    const fn export(tage_type: TagType, flags: ExportTrackFlags) -> Self {
        Self::new(
            tage_type,
            flags.contains(ExportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        )
    }

    const fn new(tage_type: TagType, apple_grp1: bool) -> Self {
        let primary_content_group;
        let secondary_content_group;
        let primary_work;
        let secondary_work;
        if matches!(tage_type, TagType::Id3v2) {
            primary_content_group = ItemKey::AppleId3v2ContentGroup; // GRP1
            primary_work = ItemKey::Work; // TXXX:WORK
            if apple_grp1 {
                secondary_content_group = None;
                secondary_work = Some(ItemKey::ContentGroup); // TIT1
            } else {
                secondary_content_group = Some(ItemKey::ContentGroup); // TIT1
                secondary_work = None;
            }
        } else {
            primary_content_group = ItemKey::ContentGroup;
            secondary_content_group = None;
            primary_work = ItemKey::Work;
            secondary_work = None;
        }
        Self {
            primary_content_group,
            secondary_content_group,
            primary_work,
            secondary_work,
        }
    }

    #[must_use]
    fn primary_item_key<'a>(&'a self, item_key: &'a ItemKey) -> &'a ItemKey {
        let Compatibility {
            primary_content_group,
            secondary_content_group,
            primary_work,
            secondary_work,
        } = self;
        if let Some(secondary_content_group) = secondary_content_group {
            if secondary_content_group == item_key {
                return primary_content_group;
            }
        }
        if let Some(secondary_work) = secondary_work {
            if secondary_work == item_key {
                return primary_work;
            }
        }
        item_key
    }
}

fn try_parse_boolean_flag(input: &str) -> Option<bool> {
    let input = input.trim();
    input
        .parse::<u8>()
        .ok()
        .and_then(|value| match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })
        .or_else(|| {
            // Fallback: Parse "true" or "false" as boolean
            input.to_ascii_lowercase().parse::<bool>().ok()
        })
}

fn tag_take_strings<'a>(tag: &'a mut Tag, key: &'a ItemKey) -> impl Iterator<Item = String> + 'a {
    // Retain all items with a non-empty description.
    tag.take_filter(key, |item| item.description().is_empty())
        .filter_map(|item| item.into_value().into_string())
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
    let mut tempo_bpm_strings = tag_take_strings(&mut tag, &ItemKey::Bpm)
        .map(|input| (false, input))
        .collect::<Vec<_>>();
    tempo_bpm_strings
        .extend(tag_take_strings(&mut tag, &ItemKey::IntegerBpm).map(|input| (true, input)));
    for (is_integer, imported_tempo_bpm) in
        tempo_bpm_strings
            .into_iter()
            .filter_map(|(is_integer, input)| {
                importer.import_tempo_bpm(&input).map(|bpm| {
                    // The file might still contain a fractional value even if the tag
                    // is supposed to contain only an integer value!
                    let is_integer = is_integer && bpm.is_integer();
                    (is_integer, bpm)
                })
            })
    {
        if is_integer
            && track.metrics.tempo_bpm.is_some()
            && !track
                .metrics
                .flags
                .contains(MetricsFlags::TEMPO_BPM_INTEGER)
        {
            // Preserve the existing fractional bpm and do not overwrite it with
            // the imprecise integer value. Instead continue and try to import
            // a more precise, fractional bpm from another tag field.
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
            .set(MetricsFlags::TEMPO_BPM_INTEGER, is_integer);
        if !is_integer {
            // Abort after importing the first fractional bpm
            break;
        }
        // Continue and try to import a more precise, fractional bpm.
    }

    // Musical metrics: key signature
    let new_key_signature = tag_take_strings(&mut tag, &ItemKey::InitialKey)
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
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::TrackTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Main))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::TrackTitleSortOrder)
        .find_map(|name| ingest_title_from(name, TitleKind::Sorting))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::TrackSubtitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Sub))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::Movement)
        .find_map(|name| ingest_title_from(name, TitleKind::Movement))
    {
        track_titles.push(title);
    }
    let primary_work_title = tag_take_strings(&mut tag, &compatibility.primary_work)
        .find_map(|name| ingest_title_from(name, TitleKind::Work));
    if let Some(work_title) = primary_work_title.or_else(|| {
        compatibility.secondary_work.and_then(|secondary_work| {
            tag_take_strings(&mut tag, &secondary_work)
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
    for name in tag_take_strings(&mut tag, &ItemKey::TrackArtist) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Artist,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::TrackArtistSortOrder) {
        push_next_actor(
            &mut track_actors,
            name,
            ActorKind::Sorting,
            ActorRole::Artist,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Arranger) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Arranger,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Composer) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Composer,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::ComposerSortOrder) {
        push_next_actor(
            &mut track_actors,
            name,
            ActorKind::Sorting,
            ActorRole::Composer,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Conductor) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Conductor,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Director) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Director,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Engineer) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Engineer,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Lyricist) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Lyricist,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::MixDj) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::MixDj,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::MixEngineer) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::MixEngineer,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Performer) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Performer,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Producer) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Producer,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::Writer) {
        push_next_actor(
            &mut track_actors,
            name,
            Default::default(),
            ActorRole::Writer,
        );
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
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::AlbumTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Main))
    {
        album_titles.push(title);
    }
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::SetSubtitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Sub))
    {
        album_titles.push(title);
    }
    if let Some(title) = tag_take_strings(&mut tag, &ItemKey::AlbumTitleSortOrder)
        .find_map(|name| ingest_title_from(name, TitleKind::Sorting))
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
    for name in tag_take_strings(&mut tag, &ItemKey::AlbumArtist) {
        push_next_actor(
            &mut album_actors,
            name,
            Default::default(),
            ActorRole::Artist,
        );
    }
    for name in tag_take_strings(&mut tag, &ItemKey::AlbumArtistSortOrder) {
        push_next_actor(
            &mut album_actors,
            name,
            ActorKind::Sorting,
            ActorRole::Artist,
        );
    }
    let new_album_actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);
    let old_album_actors = &mut album.actors;
    if !old_album_actors.is_empty() && *old_album_actors != new_album_actors {
        log::debug!("Replacing album actors: {old_album_actors:?} -> {new_album_actors:?}");
    }
    *old_album_actors = new_album_actors;

    if let Some(item) = tag.take(&ItemKey::FlagCompilation).next() {
        if let Some(kind) =
            item.value()
                .text()
                .and_then(try_parse_boolean_flag)
                .map(|compilation| {
                    if compilation {
                        AlbumKind::Compilation
                    } else {
                        AlbumKind::NoCompilation
                    }
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

    let new_copyright = tag_take_strings(&mut tag, &ItemKey::CopyrightMessage)
        .find_map(trimmed_non_empty_from)
        .map(Cow::into_owned);
    let old_copyright = &mut track.copyright;
    if old_copyright.is_some() && *old_copyright != new_copyright {
        log::debug!("Replacing copyright: {old_copyright:?} -> {new_copyright:?}");
    }
    *old_copyright = new_copyright;

    let old_publisher = &mut track.publisher;
    let mut new_publisher = tag_take_strings(&mut tag, &ItemKey::Label)
        .find_map(trimmed_non_empty_from)
        .map(Cow::into_owned);
    if new_publisher.is_none() {
        new_publisher = tag_take_strings(&mut tag, &ItemKey::Publisher)
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
        .map(TryInto::try_into)
        .transpose()
        .ok()
        .flatten();
    let track_total = tag
        .track_total()
        .map(TryInto::try_into)
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
    let old_disc_index = &mut track.indexes.disc;
    let disc_number = tag.disk().map(TryInto::try_into).transpose().ok().flatten();
    let disc_total = tag
        .disk_total()
        .map(TryInto::try_into)
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
    let mut new_recorded_at = tag_take_strings(&mut tag, &ItemKey::RecordingDate)
        .find_map(|input| importer.import_year_tag_from_field("RecordingDate", &input));
    if new_recorded_at.is_none() {
        new_recorded_at = tag_take_strings(&mut tag, &ItemKey::Year)
            .find_map(|input| importer.import_year_tag_from_field("Year", &input));
    }
    if old_recorded_at.is_some() && *old_recorded_at != new_recorded_at {
        log::debug!("Replacing recorded at: {old_recorded_at:?} -> {new_recorded_at:?}");
    }
    *old_recorded_at = new_recorded_at;

    let old_released_at = &mut track.released_at;
    let new_released_at = tag_take_strings(&mut tag, &ItemKey::ReleaseDate)
        .find_map(|input| importer.import_year_tag_from_field("ReleaseDate", &input));
    if old_released_at.is_some() && *old_released_at != new_released_at {
        log::debug!("Replacing released at: {old_released_at:?} -> {new_released_at:?}");
    }
    *old_released_at = new_released_at;

    let old_released_orig_at = &mut track.released_orig_at;
    let new_released_orig_at = tag_take_strings(&mut tag, &ItemKey::OriginalReleaseDate)
        .find_map(|input| importer.import_year_tag_from_field("OriginalReleaseDate", &input));
    if old_released_orig_at.is_some() && *old_released_orig_at != new_released_orig_at {
        log::debug!(
            "Replacing original released at: {old_released_orig_at:?} -> {new_released_orig_at:?}"
        );
    }
    *old_released_orig_at = new_released_orig_at;

    let old_advisory_rating = &mut track.advisory_rating;
    let new_advisory_rating = tag_take_strings(&mut tag, &ItemKey::ParentalAdvisory)
        .find_map(|input| input.parse::<u8>().ok().and_then(AdvisoryRating::from_repr));
    if old_advisory_rating.is_some() && *old_advisory_rating != new_advisory_rating {
        log::debug!(
            "Replacing advisory rating: {old_advisory_rating:?} -> {new_advisory_rating:?}"
        );
    }
    *old_advisory_rating = new_advisory_rating;

    let mut tags_map: TagsMap<'static> = Default::default();

    // Grouping tags
    debug_assert!(tags_map.get_faceted_plain_tags(FACET_ID_GROUPING).is_none());
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_GROUPING,
        tag_take_strings(&mut tag, &compatibility.primary_content_group).map(Into::into),
    );
    if let Some(secondary_content_group) = compatibility.secondary_content_group {
        if tags_map.get_faceted_plain_tags(FACET_ID_GROUPING).is_none() {
            importer.import_faceted_tags_from_label_values(
                &mut tags_map,
                &config.faceted_tag_mapping,
                FACET_ID_GROUPING,
                tag_take_strings(&mut tag, &secondary_content_group).map(Into::into),
            );
        }
    }

    // Import gig tags from raw grouping tags before any other tags.
    #[cfg(feature = "gigtag")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS_CGRP) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(FACET_ID_GROUPING) {
            tags_map.merge(crate::util::gigtag::import_from_faceted_tags(faceted_tags));
        }
    }

    // Comment tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_COMMENT,
        tag_take_strings(&mut tag, &ItemKey::Comment).map(Into::into),
    );

    // Import additional gig tags from the raw comment tag.
    #[cfg(feature = "gigtag")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS_COMM) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(FACET_ID_COMMENT) {
            tags_map.merge(crate::util::gigtag::import_from_faceted_tags(faceted_tags));
        }
    }

    // Genre tags
    {
        let tag_mapping_config = config.faceted_tag_mapping.get(FACET_ID_GENRE.as_str());
        let mut next_score_value = PlainTag::DEFAULT_SCORE.value();
        let mut plain_tags = Vec::with_capacity(8);
        for genre in tag_take_strings(&mut tag, &ItemKey::Genre) {
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
        tag_take_strings(&mut tag, &ItemKey::Mood).map(Into::into),
    );

    // Description tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_DESCRIPTION,
        tag_take_strings(&mut tag, &ItemKey::Description).map(Into::into),
    );

    // Isrc tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_ISRC,
        tag_take_strings(&mut tag, &ItemKey::Isrc).map(Into::into),
    );

    // XID tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_XID,
        tag_take_strings(&mut tag, &ItemKey::AppleXid).map(Into::into),
    );

    // MusicBrainz tags
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_RECORDING,
        tag_take_strings(&mut tag, &ItemKey::MusicBrainzRecordingId).map(Into::into),
    );
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_TRACK,
        tag_take_strings(&mut tag, &ItemKey::MusicBrainzTrackId).map(Into::into),
    );
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_RELEASE,
        tag_take_strings(&mut tag, &ItemKey::MusicBrainzReleaseId).map(Into::into),
    );
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MBID_RELEASE_GROUP,
        tag_take_strings(&mut tag, &ItemKey::MusicBrainzReleaseGroupId).map(Into::into),
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
    actor_names: Option<FilteredActorNames<'_>>,
) {
    let Some(actor_names) = actor_names else {
        tag.remove_key(&item_key);
        return;
    };
    match actor_names {
        FilteredActorNames::Summary(name) | FilteredActorNames::Sorting(name) => {
            tag.insert_text(item_key, name.to_owned());
        }
        FilteredActorNames::Individual(names) => {
            for name in names {
                let item_key = item_key.clone();
                let item_val = ItemValue::Text(name.to_owned());
                let pushed = tag.push(TagItem::new(item_key, item_val));
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
                .filter_map(|PlainTag { label, score: _ }| label.map(Into::into)),
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
            let item_val = ItemValue::Text(Cow::from(label).into_owned());
            let pushed = tag.push(TagItem::new(item_key, item_val));
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
    if let Some(secondary_content_group) = &compatibility.secondary_content_group {
        tag.remove_key(secondary_content_group);
    }
    if let Some(secondary_work) = &compatibility.secondary_work {
        tag.remove_key(secondary_work);
    }

    // Music: Tempo/Bpm
    // Write both the non-fractional and the fractional Bpm value. Depending on the tag
    // format either one or both of them will be written.
    if let Some(formatted) =
        format_validated_tempo_bpm(&mut track.metrics.tempo_bpm, TempoBpmFormat::Integer)
    {
        tag.insert_text(ItemKey::IntegerBpm, formatted.into());
    } else {
        tag.remove_key(&ItemKey::IntegerBpm);
    }
    if let Some(formatted) = format_validated_tempo_bpm(
        &mut track.metrics.tempo_bpm,
        crate::util::TempoBpmFormat::Float,
    ) {
        if matches!(formatted, FormattedTempoBpm::Fractional(_)) {
            // Reset non-fractional flag if the actual bpm is fractional.
            track.metrics.flags.remove(MetricsFlags::TEMPO_BPM_INTEGER);
        }
        tag.insert_text(ItemKey::Bpm, formatted.into());
    } else {
        tag.remove_key(&ItemKey::Bpm);
    }

    // Music: Key
    if let Some(key_signature) = track.metrics.key_signature {
        let key_text = key_signature_as_str(key_signature).to_owned();
        tag.insert_text(ItemKey::InitialKey, key_text);
    } else {
        tag.remove_key(&ItemKey::InitialKey);
    }

    // Track titles
    if let Some(main_track_title) = Titles::main_title(track.titles.iter()) {
        tag.set_title(main_track_title.name.clone());
    } else {
        tag.remove_title();
    }
    if let Some(sort_track_title) = Titles::sort_title(track.titles.iter()) {
        tag.insert_text(ItemKey::TrackTitleSortOrder, sort_track_title.name.clone());
    } else {
        tag.remove_key(&ItemKey::TrackTitleSortOrder);
    }
    tag.remove_key(&ItemKey::TrackSubtitle);
    for track_subtitle in Titles::filter_kind(track.titles.iter(), TitleKind::Sub).peekable() {
        let item_val = ItemValue::Text(track_subtitle.name.clone());
        let pushed = tag.push(TagItem::new(ItemKey::TrackSubtitle, item_val));
        debug_assert!(pushed);
    }
    for movement_title in Titles::filter_kind(track.titles.iter(), TitleKind::Movement).peekable() {
        let item_val = ItemValue::Text(movement_title.name.clone());
        let pushed = tag.push(TagItem::new(ItemKey::Movement, item_val));
        debug_assert!(pushed);
    }
    for work_title in Titles::filter_kind(track.titles.iter(), TitleKind::Work).peekable() {
        let item_val = ItemValue::Text(work_title.name.clone());
        let pushed = tag.push(TagItem::new(compatibility.primary_work.clone(), item_val));
        debug_assert!(pushed);
    }

    // Track actors
    export_filtered_actor_names(
        tag,
        ItemKey::TrackArtist,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Artist, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::TrackArtistSortOrder,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Artist, ActorKind::Sorting),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Arranger,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Arranger, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Composer,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Composer, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::ComposerSortOrder,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Composer, ActorKind::Sorting),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Conductor,
        FilteredActorNames::filter(
            track.actors.iter(),
            ActorRole::Conductor,
            Default::default(),
        ),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Director,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Director, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Engineer,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Engineer, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::MixDj,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::MixDj, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::MixEngineer,
        FilteredActorNames::filter(
            track.actors.iter(),
            ActorRole::MixEngineer,
            Default::default(),
        ),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Lyricist,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Lyricist, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Performer,
        FilteredActorNames::filter(
            track.actors.iter(),
            ActorRole::Performer,
            Default::default(),
        ),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Producer,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Producer, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Remixer,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Remixer, Default::default()),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Writer,
        FilteredActorNames::filter(track.actors.iter(), ActorRole::Writer, Default::default()),
    );

    // Album
    if let Some(main_album_title) = Titles::main_title(track.album.titles.iter()) {
        tag.set_album(main_album_title.name.clone());
    } else {
        tag.remove_album();
    }
    if let Some(sort_album_title) = Titles::sort_title(track.album.titles.iter()) {
        tag.insert_text(ItemKey::AlbumTitleSortOrder, sort_album_title.name.clone());
    } else {
        tag.remove_key(&ItemKey::AlbumTitleSortOrder);
    }
    for album_subtitle in Titles::filter_kind(track.album.titles.iter(), TitleKind::Sub).peekable()
    {
        let item_val = ItemValue::Text(album_subtitle.name.clone());
        let pushed = tag.push(TagItem::new(ItemKey::SetSubtitle, item_val));
        debug_assert!(pushed);
    }
    export_filtered_actor_names(
        tag,
        ItemKey::AlbumArtist,
        FilteredActorNames::filter(
            track.album.actors.iter(),
            ActorRole::Artist,
            Default::default(),
        ),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::AlbumArtistSortOrder,
        FilteredActorNames::filter(
            track.album.actors.iter(),
            ActorRole::Artist,
            ActorKind::Sorting,
        ),
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

    if let Some(recorded_at) = &track.recorded_at {
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
    if let Some(released_at) = &track.released_at {
        let released_at_text = released_at.to_string();
        tag.insert_text(ItemKey::ReleaseDate, released_at_text);
    } else {
        tag.remove_key(&ItemKey::ReleaseDate);
    }
    if let Some(released_orig_at) = &track.released_orig_at {
        let released_orig_at_text = released_orig_at.to_string();
        tag.insert_text(ItemKey::OriginalReleaseDate, released_orig_at_text);
    } else {
        tag.remove_key(&ItemKey::OriginalReleaseDate);
    }

    {
        let mut tags_map = TagsMap::from(track.tags.clone().untie());

        for (facet_id, item_key) in DEFAULT_FILE_TAG_MAPPING {
            #[cfg(feature = "gigtag")]
            if config.encode_gigtags.as_ref() == Some(facet_id) {
                // Defer until later (see below).
            }
            let item_key = compatibility.primary_item_key(item_key).clone();
            if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_GENRE)
            {
                export_faceted_tags(
                    tag,
                    item_key,
                    config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
                    tags,
                );
            } else {
                tag.remove_key(&item_key);
            }
        }

        #[cfg(feature = "gigtag")]
        if let Some(facet_id) = &config.encode_gigtags {
            if let Some(item_key) =
                DEFAULT_FILE_TAG_MAPPING
                    .iter()
                    .copied()
                    .find_map(|(file_tag_id, item_key)| {
                        if file_tag_id == facet_id {
                            Some(item_key)
                        } else {
                            None
                        }
                    })
            {
                let item_key = compatibility.primary_item_key(item_key).clone();
                let mut tags = tags_map
                    .take_faceted_tags(facet_id)
                    .map(|FacetedTags { facet_id: _, tags }| tags)
                    .unwrap_or_default();
                // Verify that the map does not contain any tags that have already been exported.
                debug_assert!(tags_map.facet_keys().all(|facet_key| {
                    let Some(key_facet_id) = facet_key.as_ref() else {
                        // Plain tag.
                        return true;
                    };
                    file_tag_facets_without(facet_id)
                        .all(|other_facet_id| other_facet_id != key_facet_id)
                }));
                let remaining_tags = tags_map.canonicalize_into();
                if let Err(err) = crate::util::gigtag::export_and_encode_tags_into(
                    remaining_tags.as_canonical_ref(),
                    &mut tags,
                ) {
                    log::error!("Failed to export gig tags: {err}");
                }
                if tags.is_empty() {
                    tag.remove_key(&item_key);
                } else {
                    export_faceted_tags(
                        tag,
                        item_key,
                        config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
                        tags,
                    );
                }
            } else {
                log::error!("Cannot export gig tags through facet \"{facet_id}\": no file tag");
            }
        }
    }

    // Advisory rating
    if let Some(advisory_rating) = track.advisory_rating {
        tag.insert_text(
            ItemKey::ParentalAdvisory,
            (advisory_rating as u8).to_string(),
        );
    } else {
        tag.remove_key(&ItemKey::ParentalAdvisory);
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
                let picture = Picture::new_unchecked(pic_type, Some(mime_type), None, image_data);
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
