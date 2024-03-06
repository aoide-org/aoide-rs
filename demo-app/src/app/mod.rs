// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, path::PathBuf, sync::mpsc};

use eframe::{CreationContext, Frame};
use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions};

use aoide::{
    desktop_app::fs::DirPath,
    media::{
        artwork::{Artwork, ArtworkImage, EmbeddedArtwork},
        content::ContentPath,
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::FacetId,
    track::{
        tag::{FACET_ID_COMMENT, FACET_ID_GENRE, FACET_ID_GROUPING},
        AdvisoryRating,
    },
    util::{clock::DateOrDateTime, color::RgbColor},
    TrackUid,
};
use itertools::Itertools as _;

use crate::{
    library::{self, track_search, Library},
    NoReceiverForEvent,
};

mod update;
use self::update::UpdateContext;

mod render;
use self::render::RenderContext;

#[allow(missing_debug_implementations)]
struct NoReceiverForMessage(Message);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct MessageSender {
    ctx: Context,
    msg_tx: mpsc::Sender<Message>,
}

impl MessageSender {
    const fn new(ctx: Context, msg_tx: mpsc::Sender<Message>) -> Self {
        Self { ctx, msg_tx }
    }

    fn send_action<T>(&self, action: T)
    where
        T: Into<Action>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Action(action.into())) {
            let Message::Action(action) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action {action:?}");
        }
    }

    #[allow(dead_code)] // TODO: Currently unused?
    fn emit_event<T>(&self, event: T) -> Result<(), NoReceiverForEvent>
    where
        T: Into<Event>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Event(event.into())) {
            let Message::Event(event) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for event: {event:?}");
            return Err(NoReceiverForEvent);
        }
        Ok(())
    }

    fn send_message(&self, msg: Message) -> Result<(), NoReceiverForMessage> {
        self.msg_tx.send(msg).map_err(|err| {
            log::warn!("Failed to send message: {err}");
            NoReceiverForMessage(err.0)
        })?;
        // Queued messages are consumed before rendering the next frame.
        self.ctx.request_repaint();
        Ok(())
    }
}

impl library::EventEmitter for MessageSender {
    fn emit_event(&self, event: library::Event) -> Result<(), NoReceiverForEvent> {
        let event: Event = Event::Library(event);
        self.send_message(Message::Event(event))
            .map_err(|NoReceiverForMessage(_)| NoReceiverForEvent)
    }
}

// Not cloneable so large enum variants should be fine.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum Message {
    Action(Action),
    Event(Event),
}

impl From<Action> for Message {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

impl From<Event> for Message {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

#[derive(Debug)]
enum Action {
    Library(LibraryAction),
}

#[derive(Debug)]
enum LibraryAction {
    MusicDirectory(MusicDirectoryAction),
    Collection(CollectionAction),
    TrackSearch(TrackSearchAction),
}

impl<T> From<T> for Action
where
    T: Into<LibraryAction>,
{
    fn from(action: T) -> Self {
        Self::Library(action.into())
    }
}

#[derive(Debug)]
enum MusicDirectoryAction {
    Reset,
    Select,
    Update(Option<DirPath<'static>>),
    SpawnSyncTask,
    AbortPendingSyncTask,
    FinishSync,
    OpenListView,
    CloseListView,
}

impl From<MusicDirectoryAction> for LibraryAction {
    fn from(action: MusicDirectoryAction) -> Self {
        Self::MusicDirectory(action)
    }
}

#[derive(Debug, Clone)]
enum CollectionAction {
    RefreshFromDb,
}

impl From<CollectionAction> for LibraryAction {
    fn from(action: CollectionAction) -> Self {
        Self::Collection(action)
    }
}

#[derive(Debug)]
enum TrackSearchAction {
    Search(String),
    FetchMore,
    AbortPendingStateChange,
    ApplyPendingStateChange {
        fetched_items: TrackSearchFetchedItems,
    },
}

#[derive(Debug)]
enum TrackSearchFetchedItems {
    Reset,
    Replace(Vec<TrackListItem>),
    Append(Vec<TrackListItem>),
}

impl From<TrackSearchAction> for LibraryAction {
    fn from(action: TrackSearchAction) -> Self {
        Self::TrackSearch(action)
    }
}

/// App-level event
///
/// Not cloneable to prevent unintended storage. Notifications are
/// supposed to be ephemeral and should disappear after being processed.
#[derive(Debug)]
enum Event {
    Library(library::Event),
}

impl From<library::Event> for Event {
    fn from(event: library::Event) -> Self {
        Self::Library(event)
    }
}

// Mutually exclusive modes of operation.
#[derive(Debug)]
enum ModelMode {
    TrackSearch(TrackSearchMode),
    MusicDirSync {
        last_progress: Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>,
        final_outcome:
            Option<Box<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>>,
    },
    MusicDirList {
        content_paths_with_count: Vec<(ContentPath<'static>, usize)>,
    },
}

#[derive(Debug, Default)]
struct TrackSearchMode {
    memo_state: track_search::MemoState,
    track_list: Option<Vec<TrackListItem>>,
}

#[derive(Debug)]
enum MusicDirSelection {
    Selecting,
    Selected,
}

/// Application model
///
/// Immutable during rendering.
#[allow(missing_debug_implementations)]
pub struct Model {
    library: Library,

    mode: Option<ModelMode>,

    music_dir_selection: Option<MusicDirSelection>,
}

impl Model {
    #[must_use]
    pub const fn new(library: Library) -> Self {
        Self {
            library,
            mode: None,
            music_dir_selection: None,
        }
    }
}

/// UI data bindings
///
/// Stores user input and other mutable UI state that needs to be preserved
/// between frames.
#[derive(Debug, Default)]
struct UiData {
    track_search_input: String,
}

#[allow(missing_debug_implementations)]
pub struct App {
    rt: tokio::runtime::Handle,

    msg_rx: mpsc::Receiver<Message>,
    msg_tx: MessageSender,

    mdl: Model,

    ui_data: UiData,
}

impl App {
    #[must_use]
    pub fn new(
        ctx: &CreationContext<'_>,
        rt: tokio::runtime::Handle,
        mdl: Model,
        settings_dir: PathBuf,
    ) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx.egui_ctx.clone(), msg_tx);
        mdl.library.spawn_background_tasks(&rt, settings_dir);
        mdl.library.spawn_event_tasks(&rt, &msg_tx);
        Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ui_data: Default::default(),
        }
    }

    fn update(&mut self) -> (&mpsc::Receiver<Message>, UpdateContext<'_>) {
        let Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ..
        } = self;
        let ctx = UpdateContext { rt, msg_tx, mdl };
        (msg_rx, ctx)
    }

    fn render(&mut self) -> RenderContext<'_> {
        let Self {
            msg_tx,
            mdl,
            ui_data,
            ..
        } = self;
        RenderContext {
            msg_tx,
            mdl,
            ui_data,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frm: &mut Frame) {
        let (msg_rx, mut update_ctx) = self.update();
        let msg_count = msg_rx
            .try_iter()
            .map(|msg| {
                update_ctx.on_message(ctx, msg);
            })
            .count();
        if msg_count > 0 {
            log::debug!("Processed {msg_count} message(s) before rendering frame");
        }

        let mut render_ctx = self.render();
        render_ctx.render_frame(ctx, frm);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackYear {
    pub min: i16,
    pub max: i16,
}

/// Simplified, pre-rendered track data
pub struct TrackListItem {
    pub entity_uid: TrackUid,
    pub artwork_thumbnail_texture: TextureHandle,

    pub artist: Option<String>,
    pub title: Option<String>,
    pub album_artist: Option<String>,
    pub album_title: Option<String>,
    pub album_subtitle: Option<String>,
    pub copyright: Option<String>,
    pub advisory_rating: Option<AdvisoryRating>,
    pub grouping: Option<String>,
    pub comment: Option<String>,
    pub genres: Vec<String>,
    pub year: Option<TrackYear>,
    pub bpm: Option<TempoBpm>,
    pub key: Option<KeySignature>,
}

const MULTI_VALUED_TAG_LABEL_SEPARATOR: &str = "\n";

impl TrackListItem {
    #[must_use]
    pub fn new(ctx: &Context, entity_uid: aoide::TrackUid, track: &aoide::Track) -> Self {
        let artwork_thumbnail_image = track
            .media_source
            .artwork
            .as_ref()
            .and_then(|artwork| {
                let default_color = match track.color {
                    Some(aoide::util::color::Color::Rgb(rgb_color)) => Some(rgb_color),
                    _ => None,
                };
                artwork_thumbnail_image(artwork, default_color)
            })
            .unwrap_or_else(|| {
                // TODO: Use a single, shared, transparent texture for all tracks without artwork.
                artwork_thumbnail_image_placeholder()
            });
        // TODO: Only load the texture once for each distinct image -> hash the image data.
        let artwork_thumbnail_texture = ctx.load_texture(
            "", // anonymous
            artwork_thumbnail_image,
            TextureOptions::LINEAR,
        );
        let artist = track.track_artist().map(ToOwned::to_owned);
        let title = track.track_title().map(ToOwned::to_owned);
        let album_artist = track.album_artist().map(ToOwned::to_owned);
        let album_title = track.album_title().map(ToOwned::to_owned);
        let album_subtitle = track.album_subtitle().map(ToOwned::to_owned);
        let copyright = track.copyright.clone();
        let advisory_rating = track.advisory_rating;
        let genres = filter_faceted_track_tag_labels(track, FACET_ID_GENRE)
            .map(ToString::to_string)
            .collect();
        let grouping = concat_faceted_track_tag_labels(
            track,
            FACET_ID_GROUPING,
            MULTI_VALUED_TAG_LABEL_SEPARATOR,
        );
        let comment = concat_faceted_track_tag_labels(
            track,
            FACET_ID_COMMENT,
            MULTI_VALUED_TAG_LABEL_SEPARATOR,
        );
        let dates = track
            .recorded_at
            .into_iter()
            .chain(track.released_at)
            .chain(track.released_orig_at);
        let year_min = dates.clone().map(DateOrDateTime::year).min();
        let year_max = dates.map(DateOrDateTime::year).max();
        let year = match (year_min, year_max) {
            (Some(min), Some(max)) => Some(TrackYear { min, max }),
            (None, None) => None,
            _ => unreachable!(),
        };
        let bpm = track.metrics.tempo_bpm;
        let key = track.metrics.key_signature;
        Self {
            entity_uid,
            artwork_thumbnail_texture,
            artist,
            title,
            album_artist,
            album_title,
            album_subtitle,
            copyright,
            advisory_rating,
            grouping,
            comment,
            genres,
            year,
            bpm,
            key,
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for TrackListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrackListItem")
            .field("entity_uid", &self.entity_uid)
            .field(
                "artwork_thumbnail_texture",
                &self.artwork_thumbnail_texture.id(),
            )
            .finish()
    }
}

fn filter_faceted_track_tag_labels<'a>(
    track: &'a aoide::Track,
    facet_id: &'a FacetId<'a>,
) -> impl Iterator<Item = &'a aoide::tag::Label<'a>> {
    track
        .tags
        .facets
        .iter()
        .filter_map(|faceted_tags| {
            if faceted_tags.facet_id == *facet_id {
                Some(faceted_tags.tags.iter())
            } else {
                None
            }
        })
        .flatten()
        .filter_map(|tag| tag.label.as_ref())
}

#[must_use]
#[allow(unstable_name_collisions)] // Itertools::intersperse()
fn concat_faceted_track_tag_labels(
    track: &aoide::Track,
    facet_id: &FacetId<'_>,
    separator: &str,
) -> Option<String> {
    let concat = filter_faceted_track_tag_labels(track, facet_id)
        .map(aoide::tag::Label::as_str)
        .intersperse(separator)
        .collect::<String>();
    if concat.is_empty()
        && filter_faceted_track_tag_labels(track, facet_id)
            .next()
            .is_none()
    {
        None
    } else {
        Some(concat)
    }
}

const ARTWORK_THUMBNAIL_SIZE: usize = 4;
const ARTWORK_THUMBNAIL_IMAGE_SIZE: usize = 6;
const ARTWORK_THUMBNAIL_BORDER_SIZE: usize = 1;

#[must_use]
const fn solid_rgb_color(color: RgbColor) -> Color32 {
    Color32::from_rgb(color.red(), color.green(), color.blue())
}

#[must_use]
fn artwork_thumbnail_image_with_solid_color(color: Color32) -> ColorImage {
    ColorImage {
        size: [ARTWORK_THUMBNAIL_IMAGE_SIZE, ARTWORK_THUMBNAIL_IMAGE_SIZE],
        pixels: [color; ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_IMAGE_SIZE].to_vec(),
    }
}

#[must_use]
fn artwork_thumbnail_image_placeholder() -> ColorImage {
    artwork_thumbnail_image_with_solid_color(Color32::TRANSPARENT)
}

#[must_use]
fn artwork_thumbnail_image_from_rgb_pixels(
    thumbnail: &[u8; ARTWORK_THUMBNAIL_SIZE * ARTWORK_THUMBNAIL_SIZE * 3],
    border_color: Color32,
) -> ColorImage {
    let pixels = thumbnail
        .chunks_exact(3)
        .map(|rgb| Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
    artwork_thumbnail_image_from_pixels(pixels, border_color)
}

#[must_use]
#[allow(clippy::similar_names)]
fn artwork_thumbnail_image_from_pixels(
    pixels: impl IntoIterator<Item = Color32>,
    border_color: Color32,
) -> ColorImage {
    // TODO: Avoid temporary allocation.
    let pixels = pixels.into_iter().collect::<Vec<_>>();
    let mut pixels_rows = pixels.chunks_exact(4);
    let pixels_row0 = pixels_rows.next().unwrap();
    let pixels_row1 = pixels_rows.next().unwrap();
    let pixels_row2 = pixels_rows.next().unwrap();
    let pixels_row3 = pixels_rows.next().unwrap();
    let pixels = std::iter::repeat(border_color)
        .take(
            ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_BORDER_SIZE
                + ARTWORK_THUMBNAIL_BORDER_SIZE,
        )
        .chain(pixels_row0.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row1.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row2.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row3.iter().copied())
        .chain(std::iter::repeat(border_color).take(
            ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_BORDER_SIZE
                + ARTWORK_THUMBNAIL_BORDER_SIZE,
        ))
        .collect::<Vec<_>>();
    debug_assert_eq!(
        pixels.len(),
        ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_IMAGE_SIZE
    );
    ColorImage {
        size: [ARTWORK_THUMBNAIL_IMAGE_SIZE, ARTWORK_THUMBNAIL_IMAGE_SIZE],
        pixels,
    }
}

#[must_use]
#[allow(clippy::similar_names)]
fn artwork_thumbnail_image(
    artwork: &Artwork,
    default_color: Option<RgbColor>,
) -> Option<ColorImage> {
    let Artwork::Embedded(EmbeddedArtwork {
        image: ArtworkImage {
            thumbnail, color, ..
        },
        ..
    }) = artwork
    else {
        return None;
    };
    let color = color.or(default_color);
    let Some(thumbnail) = thumbnail else {
        return color
            .map(solid_rgb_color)
            .map(artwork_thumbnail_image_with_solid_color);
    };
    let color = color.map(solid_rgb_color);
    let border_color = color.unwrap_or(Color32::TRANSPARENT);
    Some(artwork_thumbnail_image_from_rgb_pixels(
        thumbnail,
        border_color,
    ))
}
