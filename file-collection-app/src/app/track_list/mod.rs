// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tabular layout of the track search results with selectable and movable columns.

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use egui::{
    Align, Align2, Button, Checkbox, DragAndDrop, Label, LayerId, Layout, OpenUrl, Order,
    PopupCloseBehavior, Rect, RichText, ScrollArea, Sense, Stroke, TextStyle, Ui, Vec2,
    containers::menu::{MenuButton, MenuConfig},
    load::SizedTexture,
};
use egui_extras::{Column, TableBuilder, TableRow};
use itertools::Itertools as _;

use aoide::track::AdvisoryRating;

use crate::library::{
    track_search,
    ui::{ARTWORK_THUMBNAIL_IMAGE_SIZE, TrackListItem},
};

use super::{
    MessageSender, TrackEditAction, TrackListColumnsAction, TrackSearchAction, TrackSelection,
    TrackSelectionAction, render::track_list_item_bpm,
};

/// Stores the identifiers of the selected columns in display order as a JSON array.
const COLUMNS_FILE_NAME: &str = "track_list_columns.json";

const UNSYNCHRONIZED_MARKER: &str = "*";

const COLUMN_MIN_WIDTH: f32 = 16.0;

const COLUMN_DROP_MARKER_WIDTH: f32 = 2.0;

const DRAGGED_COLUMN_LABEL_MARGIN: f32 = 4.0;

const DRAGGED_COLUMN_LABEL_POINTER_OFFSET: Vec2 = Vec2::new(16.0, 0.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TrackListColumn {
    Artwork,
    Unsynchronized,
    Artist,
    Title,
    AlbumTitle,
    AlbumSubtitle,
    AlbumArtist,
    Genres,
    Grouping,
    Comment,
    Year,
    Bpm,
    Key,
    AdvisoryRating,
    Copyright,
    FilePath,
}

impl TrackListColumn {
    /// All columns in the order of the selection menu.
    pub(crate) const ALL: [Self; 16] = [
        Self::Artwork,
        Self::Unsynchronized,
        Self::Artist,
        Self::Title,
        Self::AlbumTitle,
        Self::AlbumSubtitle,
        Self::AlbumArtist,
        Self::Genres,
        Self::Grouping,
        Self::Comment,
        Self::Year,
        Self::Bpm,
        Self::Key,
        Self::AdvisoryRating,
        Self::Copyright,
        Self::FilePath,
    ];

    /// Permanent identifier in the persistent settings.
    ///
    /// Independent of the enum variant name and must remain stable.
    #[must_use]
    const fn id(self) -> &'static str {
        match self {
            Self::Artwork => "artwork",
            Self::Unsynchronized => "unsynchronized",
            Self::Artist => "artist",
            Self::Title => "title",
            Self::AlbumTitle => "album_title",
            Self::AlbumSubtitle => "album_subtitle",
            Self::AlbumArtist => "album_artist",
            Self::Genres => "genres",
            Self::Grouping => "grouping",
            Self::Comment => "comment",
            Self::Year => "year",
            Self::Bpm => "bpm",
            Self::Key => "key",
            Self::AdvisoryRating => "advisory_rating",
            Self::Copyright => "copyright",
            Self::FilePath => "file_path",
        }
    }

    #[must_use]
    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|column| column.id() == id)
    }

    /// The name of the column in the selection menu.
    #[must_use]
    const fn title(self) -> &'static str {
        match self {
            Self::Artwork => "Artwork",
            Self::Unsynchronized => "Modified",
            Self::Artist => "Artist",
            Self::Title => "Title",
            Self::AlbumTitle => "Album",
            Self::AlbumSubtitle => "Album subtitle",
            Self::AlbumArtist => "Album artist",
            Self::Genres => "Genre",
            Self::Grouping => "Grouping",
            Self::Comment => "Comment",
            Self::Year => "Year",
            Self::Bpm => "BPM",
            Self::Key => "Key",
            Self::AdvisoryRating => "Advisory",
            Self::Copyright => "Copyright",
            Self::FilePath => "File",
        }
    }

    /// The text of the table header.
    #[must_use]
    const fn header(self) -> &'static str {
        match self {
            // Too narrow for any title.
            Self::Artwork => "",
            Self::Unsynchronized => UNSYNCHRONIZED_MARKER,
            _ => self.title(),
        }
    }

    #[must_use]
    fn table_column(self, row_height: f32, is_last: bool, width: Option<f32>) -> Column {
        let default_width = match self {
            Self::Artwork => {
                return Column::exact(row_height).resizable(false);
            }
            Self::Unsynchronized => {
                return Column::exact(COLUMN_MIN_WIDTH).resizable(false);
            }
            Self::Bpm | Self::Key => 48.0,
            Self::Year | Self::AdvisoryRating => 72.0,
            Self::Artist | Self::AlbumArtist | Self::Genres | Self::Grouping | Self::Copyright => {
                160.0
            }
            Self::Title | Self::AlbumTitle | Self::AlbumSubtitle | Self::Comment => 240.0,
            Self::FilePath => 400.0,
        };
        // The last column occupies the remaining width, even after the
        // available width has changed. Only non-resizable columns are updated.
        let column = if is_last {
            Column::remainder().resizable(false)
        } else {
            Column::initial(width.unwrap_or(default_width))
        };
        column.at_least(COLUMN_MIN_WIDTH).clip(true)
    }

    /// The text of the table cell.
    ///
    /// Returns `None` for columns that are not rendered as plain text.
    #[must_use]
    fn cell_text(self, item: &TrackListItem) -> Option<String> {
        match self {
            Self::Artwork | Self::Unsynchronized => None,
            Self::Artist => item.artist.clone(),
            Self::Title => item.title.clone(),
            Self::AlbumTitle => item.album_title.clone(),
            Self::AlbumSubtitle => item.album_subtitle.clone(),
            Self::AlbumArtist => item.album_artist.clone(),
            Self::Genres => Some(item.genres.iter().join(", ")),
            Self::Grouping => item.grouping.clone(),
            Self::Comment => item.comment.clone(),
            Self::Year => item.year.map(|year| {
                if year.min == year.max {
                    year.min.to_string()
                } else {
                    format!("{min}-{max}", min = year.min, max = year.max)
                }
            }),
            Self::Bpm => track_list_item_bpm(item).map(|bpm| bpm.to_string()),
            Self::Key => item.key.map(|key| key.code().as_lancelot_str().to_owned()),
            Self::AdvisoryRating => item.advisory_rating.and_then(|rating| match rating {
                AdvisoryRating::Unrated => None,
                AdvisoryRating::Explicit => Some("Explicit".to_owned()),
                AdvisoryRating::Clean => Some("Clean".to_owned()),
            }),
            Self::Copyright => item.copyright.clone(),
            Self::FilePath => item.content_url.as_ref().map(content_url_display),
        }
    }
}

/// The selected columns of the track list.
///
/// Contains at least one column. Otherwise no tracks could be clicked
/// and there would be no table to display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrackListColumns {
    /// Unique and in display order.
    selected: Vec<TrackListColumn>,
}

impl Default for TrackListColumns {
    /// Displays the same information as the former, unstructured list.
    fn default() -> Self {
        Self {
            selected: vec![
                TrackListColumn::Artwork,
                TrackListColumn::Unsynchronized,
                TrackListColumn::Artist,
                TrackListColumn::Title,
                TrackListColumn::AlbumTitle,
                TrackListColumn::AlbumArtist,
                TrackListColumn::Bpm,
                TrackListColumn::Key,
            ],
        }
    }
}

impl TrackListColumns {
    #[must_use]
    pub(crate) fn contains(&self, column: TrackListColumn) -> bool {
        self.selected.contains(&column)
    }

    /// Select or deselect a column.
    ///
    /// Newly selected columns are appended. The last selected column
    /// cannot be deselected.
    ///
    /// Returns `true` if the selection has changed.
    pub(crate) fn toggle(&mut self, column: TrackListColumn) -> bool {
        if let Some(index) = self.selected.iter().position(|&other| other == column) {
            if self.selected.len() == 1 {
                return false;
            }
            self.selected.remove(index);
        } else {
            self.selected.push(column);
        }
        true
    }

    /// The final index of a selected column when moving it.
    ///
    /// The `insertion_index` is the position in the current order, i.e.
    /// the column is inserted before the column at this index.
    ///
    /// Returns `None` if moving the column would not have any effect.
    #[must_use]
    fn move_target_index(&self, column: TrackListColumn, insertion_index: usize) -> Option<usize> {
        let index = self.selected.iter().position(|&other| other == column)?;
        let target_index = if insertion_index > index {
            insertion_index - 1
        } else {
            insertion_index
        };
        (target_index != index && target_index < self.selected.len()).then_some(target_index)
    }

    /// Move a selected column.
    ///
    /// See [`Self::move_target_index`] for the semantics of `insertion_index`.
    ///
    /// Returns `true` if the order has changed.
    pub(crate) fn move_column(&mut self, column: TrackListColumn, insertion_index: usize) -> bool {
        let Some(target_index) = self.move_target_index(column, insertion_index) else {
            return false;
        };
        self.selected.retain(|&other| other != column);
        self.selected.insert(target_index, column);
        true
    }

    /// Identifiers that are unknown, e.g. written by a different version
    /// of the app, are ignored.
    ///
    /// Returns `None` if the contents are invalid or no column is selected.
    #[must_use]
    fn from_json(json: &[u8]) -> Option<Self> {
        let ids = serde_json::from_slice::<Vec<String>>(json).ok()?;
        let selected = ids
            .iter()
            .filter_map(|id| TrackListColumn::from_id(id))
            .unique()
            .collect::<Vec<_>>();
        (!selected.is_empty()).then_some(Self { selected })
    }

    #[must_use]
    fn to_json(&self) -> Vec<u8> {
        let ids = self
            .selected
            .iter()
            .map(|column| column.id())
            .collect::<Vec<_>>();
        serde_json::to_vec_pretty(&ids).expect("infallible")
    }

    #[must_use]
    fn file_path(settings_dir: &Path) -> PathBuf {
        settings_dir.join(COLUMNS_FILE_NAME)
    }

    /// Load the selected columns.
    ///
    /// Falls back to the default columns if the selection has not been saved
    /// yet or could not be restored.
    #[must_use]
    pub(crate) fn restore(settings_dir: &Path) -> Self {
        let file_path = Self::file_path(settings_dir);
        let json = match fs::read(&file_path) {
            Ok(json) => json,
            Err(err) => {
                if err.kind() != io::ErrorKind::NotFound {
                    log::warn!(
                        "Failed to read track list columns from file '{file_path}': {err}",
                        file_path = file_path.display(),
                    );
                }
                return Self::default();
            }
        };
        Self::from_json(&json).unwrap_or_else(|| {
            log::warn!(
                "Ignoring invalid track list columns in file '{file_path}'",
                file_path = file_path.display(),
            );
            Self::default()
        })
    }

    pub(crate) fn save(&self, settings_dir: &Path) -> io::Result<()> {
        fs::write(Self::file_path(settings_dir), self.to_json())
    }
}

#[must_use]
fn content_url_display(content_url: &url::Url) -> String {
    content_url.to_file_path().map_or_else(
        |()| content_url.to_string(),
        |path| path.display().to_string(),
    )
}

/// Open the menu for selecting the columns of the track list.
pub(super) fn render_columns_menu(ui: &mut Ui, msg_tx: &MessageSender, columns: &TrackListColumns) {
    // Keep the menu open for toggling multiple columns.
    let config = MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside);
    MenuButton::new("Columns").config(config).ui(ui, |ui| {
        for column in TrackListColumn::ALL {
            let mut is_selected = columns.contains(column);
            let is_only_selected = is_selected && columns.selected.len() == 1;
            let response = ui.add_enabled(
                !is_only_selected,
                Checkbox::new(&mut is_selected, column.title()),
            );
            if response.changed() {
                msg_tx.send_action(TrackListColumnsAction::Toggle(column));
            }
        }
    });
}

/// Column widths that are preserved when the columns change.
///
/// The table stores the widths by index, i.e. they would be applied
/// to the wrong columns after moving or toggling columns.
#[derive(Debug, Default)]
pub(super) struct TrackListTableState {
    /// The columns that the current state of the table belongs to.
    columns: Vec<TrackListColumn>,
    widths: HashMap<TrackListColumn, f32>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TrackListInteraction<'a> {
    pub(super) edited_entity_uid: Option<&'a aoide::TrackUid>,
    pub(super) track_selection: Option<&'a TrackSelection>,
    pub(super) is_track_editor_draft_modified: bool,
}

/// Render the column selection and the table with all tracks.
pub(super) fn render_track_list(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    columns: &TrackListColumns,
    table_state: &mut TrackListTableState,
    track_list: &[TrackListItem],
    could_fetch_more: bool,
    interaction: TrackListInteraction<'_>,
) {
    ui.horizontal(|ui| {
        render_columns_menu(ui, msg_tx, columns);
        ui.weak("Click to edit a track. Ctrl+click to select multiple tracks, Shift+click to select a range. Drag column headers to move columns.");
    });
    ui.separator();

    let text_height = ui.text_style_height(&TextStyle::Body);
    let row_height = if columns.contains(TrackListColumn::Artwork) {
        text_height.max(ARTWORK_THUMBNAIL_IMAGE_SIZE as _)
    } else {
        text_height
    };
    let total_rows = track_list.len();
    // An empty list is always considered as scrolled to the end.
    let mut is_last_row_visible = total_rows == 0;
    ScrollArea::horizontal().auto_shrink(false).show(ui, |ui| {
        let mut table = TableBuilder::new(ui)
            .id_salt("track-list")
            .striped(true)
            .resizable(true)
            .auto_shrink(false)
            .sense(Sense::click())
            .cell_layout(Layout::left_to_right(Align::Center));
        if table_state.columns != columns.selected {
            // Re-initialize the table from the widths of the columns.
            table.reset();
            table_state.columns.clone_from(&columns.selected);
        }
        for (index, column) in columns.selected.iter().enumerate() {
            let is_last = index + 1 == columns.selected.len();
            let width = table_state.widths.get(column).copied();
            table = table.column(column.table_column(row_height, is_last, width));
        }
        let mut header_rects = Vec::with_capacity(columns.selected.len());
        let mut table = table.header(text_height, |mut header| {
            for &column in &columns.selected {
                let (_, response) = header.col(|ui| render_column_header(ui, column));
                header_rects.push(response.rect);
            }
        });
        render_dragged_column(table.ui_mut(), msg_tx, columns, &header_rects);
        table.body(|body| {
            // The width of the last column depends on the available width.
            let resized_columns = columns.selected.len() - 1;
            table_state.widths.extend(
                columns.selected[..resized_columns]
                    .iter()
                    .copied()
                    .zip(body.widths().iter().copied()),
            );
            body.rows(row_height, total_rows, |mut row| {
                let index = row.index();
                is_last_row_visible |= index + 1 == total_rows;
                render_track_list_row(
                    &mut row,
                    msg_tx,
                    columns,
                    &track_list[index],
                    row_height,
                    interaction,
                );
            });
        });
    });
    if is_last_row_visible && could_fetch_more {
        log::debug!("Trying to fetch more track search results");
        msg_tx.send_action(TrackSearchAction::FetchMore);
    }
}

/// Check if more results should be fetched when reaching the end of the list.
#[must_use]
pub(super) fn could_fetch_more(
    library: &crate::library::CurrentState<'_>,
    memo_state: &track_search::MemoState,
    total_rows: usize,
) -> bool {
    // Prevent eagerly fetching more results repeatedly.
    Some(total_rows) == library.track_search.fetched_entities_len()
        && matches!(memo_state, track_search::MemoState::Ready(_))
        && library.could_fetch_more_track_search_results()
}

/// Render a table header that could be dragged to move the column.
fn render_column_header(ui: &mut Ui, column: TrackListColumn) {
    ui.add(
        Label::new(RichText::new(column.header()).strong())
            .selectable(false)
            .truncate(),
    );
    ui.interact(
        ui.max_rect(),
        ui.scope_id().with(("column-drag", column)),
        Sense::drag(),
    )
    .dnd_set_drag_payload(column);
}

/// The position for inserting a column that is dropped at the given position.
///
/// Only depends on the horizontal position. The header is too small for
/// reliably dropping a column on it.
#[must_use]
fn column_insertion_index(header_rects: &[Rect], pointer_x: f32) -> usize {
    header_rects
        .iter()
        .position(|rect| pointer_x < rect.center().x)
        .unwrap_or(header_rects.len())
}

/// Visualize and finish dragging a column.
///
/// The column is dropped wherever the mouse button is released. Dragging
/// could be canceled by pressing the escape key.
fn render_dragged_column(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    columns: &TrackListColumns,
    header_rects: &[Rect],
) {
    let Some(column) = DragAndDrop::payload::<TrackListColumn>(ui.ctx()).map(|column| *column)
    else {
        return;
    };
    let Some(pointer_pos) = ui.input(|input| input.pointer.interact_pos()) else {
        return;
    };
    let insertion_index = column_insertion_index(header_rects, pointer_pos.x);
    let is_move = columns.move_target_index(column, insertion_index).is_some();

    // Painted on top of the table, which has not been rendered yet.
    let painter = ui.ctx().layer_painter(LayerId::new(
        Order::Foreground,
        ui.scope_id().with("track-list-column-drag"),
    ));
    if is_move {
        let marker_x = header_rects.get(insertion_index).map_or_else(
            || header_rects.last().map_or(pointer_pos.x, Rect::right),
            Rect::left,
        );
        let clip_rect = ui.clip_rect();
        let marker_top = header_rects.first().map_or(clip_rect.top(), Rect::top);
        painter.with_clip_rect(clip_rect).vline(
            marker_x,
            marker_top..=clip_rect.bottom(),
            Stroke::new(COLUMN_DROP_MARKER_WIDTH, ui.visuals().selection.bg_fill),
        );
    }
    let text_color = ui.visuals().strong_text_color();
    let galley = painter.layout_no_wrap(
        column.title().to_owned(),
        TextStyle::Body.resolve(ui.style()),
        text_color,
    );
    let text_rect = Align2::LEFT_CENTER.anchor_size(
        pointer_pos + DRAGGED_COLUMN_LABEL_POINTER_OFFSET,
        galley.size(),
    );
    painter.rect_filled(
        text_rect.expand(DRAGGED_COLUMN_LABEL_MARGIN),
        ui.visuals().menu_corner_radius,
        ui.visuals().window_fill,
    );
    painter.galley(text_rect.min, galley, text_color);

    if is_move && ui.input(|input| input.pointer.any_released()) {
        msg_tx.send_action(TrackListColumnsAction::Move {
            column,
            insertion_index,
        });
    }
}

fn render_track_list_row(
    row: &mut TableRow<'_, '_>,
    msg_tx: &MessageSender,
    columns: &TrackListColumns,
    item: &TrackListItem,
    row_height: f32,
    interaction: TrackListInteraction<'_>,
) {
    let TrackListInteraction {
        edited_entity_uid,
        track_selection,
        is_track_editor_draft_modified,
    } = interaction;
    let is_edited = edited_entity_uid == Some(&item.entity_uid);
    let is_selected = track_selection.is_some_and(|selection| selection.contains(&item.entity_uid));
    row.set_selected(is_edited || is_selected);
    for &column in &columns.selected {
        row.col(|ui| render_track_list_cell(ui, column, item, row_height));
    }
    let response = row.response();
    if is_track_editor_draft_modified {
        // Prevent losing unsaved changes unintentionally.
        if !is_edited {
            response.on_hover_text("Save or revert the changes of the edited track first.");
        }
        return;
    }
    if track_selection.is_some_and(TrackSelection::is_updating) || !response.clicked() {
        return;
    }
    let modifiers = response.ctx.input(|input| input.modifiers);
    if modifiers.shift {
        msg_tx.send_action(TrackSelectionAction::SelectRange(item.entity_uid.clone()));
    } else if modifiers.command {
        msg_tx.send_action(TrackSelectionAction::Toggle(item.entity_uid.clone()));
    } else if !is_edited {
        msg_tx.send_action(TrackEditAction::Open(item.entity_uid.clone()));
    }
}

fn render_track_list_cell(
    ui: &mut Ui,
    column: TrackListColumn,
    item: &TrackListItem,
    row_height: f32,
) {
    match column {
        TrackListColumn::Artwork => render_artwork_cell(ui, item, row_height),
        TrackListColumn::Unsynchronized => {
            if !item.is_synchronized {
                let text = RichText::new(UNSYNCHRONIZED_MARKER).color(ui.visuals().warn_fg_color);
                ui.add(Label::new(text).selectable(false))
                    .on_hover_text("Modified, but not written into the file yet.");
            }
        }
        _ => {
            if let Some(text) = column.cell_text(item) {
                // Selectable labels would swallow the clicks on the row.
                // Truncated text is available as a tooltip.
                ui.add(Label::new(text).selectable(false).truncate());
            }
        }
    }
}

fn render_artwork_cell(ui: &mut Ui, item: &TrackListItem, row_height: f32) {
    debug_assert_eq!(
        item.artwork_thumbnail_texture.size_vec2().x,
        item.artwork_thumbnail_texture.size_vec2().y
    );
    let artwork_texture = SizedTexture {
        id: item.artwork_thumbnail_texture.id(),
        size: egui::Vec2::new(row_height, row_height),
    };
    let mut response = ui.add(Button::image(artwork_texture).frame(false));
    if let Some(content_url) = &item.content_url {
        response = response.on_hover_text_at_pointer(content_url_display(content_url));
        // Demo interaction handler that simply opens the content URL in a new (browser) tab.
        if response.clicked() || response.middle_clicked() {
            ui.open_url(OpenUrl {
                url: content_url.to_string(),
                new_tab: true,
            });
        }
    }
}

#[cfg(test)]
mod tests;
