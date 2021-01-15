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

///////////////////////////////////////////////////////////////////////

use super::*;

use std::io::SeekFrom;

use ::mp4::{ChannelConfig, MediaType, Mp4Reader, SampleFreqIndex, TrackType};
use aoide_core::{
    audio::{
        channel::{ChannelCount, ChannelLayout, Channels},
        signal::{BitRateBps, BitsPerSecond, SampleRateHz},
        AudioContent, Encoder,
    },
    track::{
        actor::{Actor, ActorKind, ActorRole},
        album::AlbumKind,
        tag::{FACET_CGROUP, FACET_COMMENT, FACET_GENRE},
        title::{Title, TitleKind},
    },
    util::{clock::DateTimeInner, Canonical, CanonicalizeInto as _},
};
use mp4ameta::{atom::read_tag_from, Ident, STANDARD_GENRES};

#[derive(Debug)]
pub struct ImportTrack;

fn read_sample_rate(sample_freq_idx: SampleFreqIndex) -> SampleRateHz {
    use SampleFreqIndex::*;
    SampleRateHz(match sample_freq_idx {
        Freq96000 => 96_000.0,
        Freq88200 => 88_200.0,
        Freq64000 => 64_000.0,
        Freq48000 => 48_000.0,
        Freq44100 => 44_100.0,
        Freq32000 => 32_000.0,
        Freq24000 => 24_000.0,
        Freq22050 => 22_500.0,
        Freq16000 => 16_000.0,
        Freq12000 => 12_000.0,
        Freq11025 => 11_025.0,
        Freq8000 => 8_000.0,
    })
}

fn read_bit_rate(bitrate: u32) -> Option<BitRateBps> {
    let bits_per_second = bitrate as BitsPerSecond;
    let bit_rate_bps = BitRateBps(bits_per_second);
    if bit_rate_bps >= BitRateBps::min() {
        Some(bit_rate_bps)
    } else {
        None
    }
}

fn read_channels(channel_config: ChannelConfig) -> Channels {
    use ChannelConfig::*;
    match channel_config {
        Mono => Channels::Layout(ChannelLayout::Mono),
        Stereo => Channels::Layout(ChannelLayout::Stereo),
        Three => Channels::Count(ChannelCount(3)),
        Four => Channels::Count(ChannelCount(4)),
        Five => Channels::Count(ChannelCount(5)),
        FiveOne => Channels::Layout(ChannelLayout::FiveOne),
        SevenOne => Channels::Layout(ChannelLayout::SevenOne),
    }
}

impl super::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        url: &Url,
        mime: &Mime,
        config: &ImportTrackConfig,
        options: ImportTrackOptions,
        input: ImportTrackInput,
        reader: &mut Box<dyn Reader>,
        size: u64,
    ) -> Result<Track> {
        let mut track = input.try_from_url_into_new_track(url, mime)?;

        // Extract metadata with mp4ameta
        let mut mp4_tag = read_tag_from(reader).map_err(anyhow::Error::from)?;

        // Restart reader to decode basic audio properties with mp4
        // that are not supported by mp4ameta.
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);
        let reader = Mp4Reader::read_header(reader, size).map_err(anyhow::Error::from)?;
        let audio_track = if let Some(audio_track) = reader
            .tracks()
            .iter()
            .find(|t| t.track_type().ok() == Some(TrackType::Audio))
        {
            audio_track
        } else {
            return Err(Error::Other(anyhow!("No audio track found")));
        };
        debug_assert_eq!(Some(MediaType::AAC), audio_track.media_type().ok());
        debug_assert_eq!(
            Content::Audio(Default::default()),
            track.media_source.content
        );

        // TODO: Remove debug logs
        log::debug!("filetype = {}", mp4_tag.filetype().unwrap());
        for ext_data_atom in mp4_tag.take_data(Ident(*b"----")) {
            log::debug!("ext_data_atom = {:?}", ext_data_atom);
        }

        let audio_content = AudioContent {
            duration: Some(audio_track.duration().into()),
            sample_rate: Some(
                audio_track
                    .sample_freq_index()
                    .map(read_sample_rate)
                    .map_err(anyhow::Error::from)?,
            ),
            bit_rate: read_bit_rate(audio_track.bitrate()),
            channels: Some(
                audio_track
                    .channel_config()
                    .map(read_channels)
                    .map_err(anyhow::Error::from)?,
            ),
            encoder: mp4_tag.take_encoder().map(|name| Encoder {
                name,
                settings: None,
            }),
            // TODO: Parse loudness from "----:com.apple.iTunes:replaygain_track_gain"
            ..Default::default()
        };
        track.media_source.content = Content::Audio(audio_content);

        // Track titles
        let mut track_titles = Vec::with_capacity(3);
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
        debug_assert!(track.titles.is_empty());
        track.titles = Canonical::tie(track_titles.canonicalize_into());

        // Track actors
        let mut track_actors = Vec::with_capacity(4);
        for name in mp4_tag.take_artists() {
            let role = ActorRole::Artist;
            let kind = adjust_last_actor_kind(&mut track_actors, role);
            let actor = Actor {
                name,
                kind,
                role,
                role_notes: None,
            };
            track_actors.push(actor);
        }
        for name in mp4_tag.take_composers() {
            let role = ActorRole::Composer;
            let kind = adjust_last_actor_kind(&mut track_actors, role);
            let actor = Actor {
                name,
                kind,
                role,
                role_notes: None,
            };
            track_actors.push(actor);
        }
        debug_assert!(track.actors.is_empty());
        track.actors = Canonical::tie(track_actors.canonicalize_into());

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
        debug_assert!(album.titles.is_empty());
        album.titles = Canonical::tie(album_titles.canonicalize_into());

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        if mp4_tag.album_artists().count() > 1 {
            for name in mp4_tag.take_album_artists() {
                let actor = Actor {
                    name,
                    kind: ActorKind::Primary,
                    role: ActorRole::Artist,
                    role_notes: None,
                };
                album_actors.push(actor);
            }
        } else if let Some(name) = mp4_tag.take_album_artist() {
            debug_assert!(mp4_tag.take_album_artist().is_none());
            let actor = Actor {
                name,
                kind: ActorKind::Summary,
                role: ActorRole::Artist,
                role_notes: None,
            };
            album_actors.push(actor);
        }
        debug_assert!(album.actors.is_empty());
        album.actors = Canonical::tie(album_actors.canonicalize_into());

        // Album properties
        if mp4_tag.compilation() {
            debug_assert_eq!(album.kind, AlbumKind::Unknown);
            album.kind = AlbumKind::Compilation;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(year) = mp4_tag.year() {
            debug_assert!(track.release.released_at.is_none());
            if let Ok(released_at) = year.parse::<DateTimeInner>() {
                track.release.released_at = Some(DateTime::from(released_at).into());
            } else {
                log::warn!("Release date not recognized: {}", year);
            }
        }
        debug_assert!(track.release.copyright.is_none());
        track.release.copyright = mp4_tag.take_copyright();

        let mut tags_map = TagsMap::default();

        // Genres
        let mut genre_count = 0;
        if mp4_tag.custom_genres().next().is_some() {
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
        if genre_count == 0 {
            // Import legacy/standard genres instead
            let mut next_score_value = TagScore::max_value();
            for genre_id in mp4_tag.standard_genres() {
                let genre = STANDARD_GENRES
                    .iter()
                    .filter_map(|(id, genre)| if *id == genre_id { Some(*genre) } else { None })
                    .next();
                if let Some(genre) = genre {
                    genre_count += import_faceted_tags(
                        &mut tags_map,
                        &mut next_score_value,
                        &FACET_GENRE,
                        None,
                        genre,
                    );
                }
            }
        }

        // Groupings
        if mp4_tag.groupings().next().is_some() {
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

        // Comment
        if let Some(comment) = mp4_tag.take_comment() {
            let mut next_score_value = TagScore::default_value();
            import_faceted_tags(
                &mut tags_map,
                &mut next_score_value,
                &FACET_COMMENT,
                None,
                comment,
            );
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        // Indexes
        debug_assert!(track.indexes.track.number.is_none());
        track.indexes.track.number = mp4_tag.track_number();
        debug_assert!(track.indexes.track.total.is_none());
        track.indexes.track.total = mp4_tag.total_tracks();
        debug_assert!(track.indexes.disc.number.is_none());
        track.indexes.disc.number = mp4_tag.disc_number();
        debug_assert!(track.indexes.disc.total.is_none());
        track.indexes.disc.total = mp4_tag.total_discs();
        debug_assert!(track.indexes.movement.number.is_none());
        track.indexes.movement.number = mp4_tag.movement_index();
        debug_assert!(track.indexes.movement.total.is_none());
        track.indexes.movement.total = mp4_tag.movement_count();

        Ok(track)
    }
}
