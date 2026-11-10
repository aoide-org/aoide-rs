// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tabular layout of the track search results with selectable and movable columns.

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use egui::{
    Align, Align2, Button, Checkbox, CornerRadius, CursorIcon, DragAndDrop, Frame, Key, Label,
    LayerId, Layout, OpenUrl, Order, PopupCloseBehavior, Rect, RichText, ScrollArea, Sense, Stroke,
    TextEdit, TextStyle, Ui, Vec2,
    containers::menu::{MenuButton, MenuConfig},
    load::SizedTexture,
};
use egui_extras::{Column, TableBuilder, TableRow};
use itertools::Itertools as _;

use aoide::{TrackUid, track::AdvisoryRating};

use crate::library::{
    track_search,
    ui::{
        ARTWORK_THUMBNAIL_IMAGE_SIZE, CustomTagsDelta, MultiTrackEdit, MultiTrackFields,
        TrackListItem,
    },
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

const GENRE_BUBBLE_HORIZONTAL_PADDING: f32 = 6.0;

const GENRE_BUBBLE_CORNER_RADIUS: u8 = 8;

const LINE_SEPARATOR: &str = "\n";

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

    /// The fields for replacing the contents of the column.
    ///
    /// Returns `None` if the column could not be edited as plain text.
    #[must_use]
    fn edited_fields(self, text: String) -> Option<MultiTrackFields> {
        let mut fields = MultiTrackFields::default();
        let field = match self {
            Self::Artist => &mut fields.artist,
            Self::Title => &mut fields.title,
            Self::AlbumTitle => &mut fields.album_title,
            Self::AlbumSubtitle => &mut fields.album_subtitle,
            Self::AlbumArtist => &mut fields.album_artist,
            Self::Grouping => &mut fields.grouping,
            Self::Comment => &mut fields.comment,
            Self::Copyright => &mut fields.copyright,
            Self::Artwork
            | Self::Unsynchronized
            | Self::Genres
            | Self::Year
            | Self::Bpm
            | Self::Key
            | Self::AdvisoryRating
            | Self::FilePath => return None,
        };
        *field = Some(text);
        Some(fields)
    }

    /// The text for editing the column directly in the table cell.
    ///
    /// Returns `None` if the column is not editable. This also applies to
    /// multiple values, which could not be edited on a single line.
    #[must_use]
    fn editable_text(self, item: &TrackListItem) -> Option<String> {
        self.edited_fields(String::new())?;
        let text = self.cell_text(item).unwrap_or_default();
        (!text.contains(LINE_SEPARATOR)).then_some(text)
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
        move_target_index(index, insertion_index, self.selected.len())
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

/// The final index of an element when moving it within a list.
///
/// The `insertion_index` is the position in the current order, i.e.
/// the element is inserted before the element at this index.
///
/// Returns `None` if moving the element would not have any effect.
#[must_use]
fn move_target_index(index: usize, insertion_index: usize, len: usize) -> Option<usize> {
    let target_index = if insertion_index > index {
        insertion_index - 1
    } else {
        insertion_index
    };
    (target_index != index && target_index < len).then_some(target_index)
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
    /// The table cell that is currently edited.
    cell_edit: Option<CellEdit>,
}

impl TrackListTableState {
    /// Finish editing the current table cell and apply the text if it has been modified.
    pub(super) fn finish_cell_edit(&mut self, msg_tx: &MessageSender) {
        if let Some(cell_edit) = self.cell_edit.take() {
            cell_edit.apply(msg_tx);
        }
    }
}

/// Editing the text of a table cell.
#[derive(Debug)]
struct CellEdit {
    entity_uid: TrackUid,
    column: TrackListColumn,
    original_text: String,
    text: String,
    is_focus_requested: bool,
}

impl CellEdit {
    #[must_use]
    fn is_cell(&self, item: &TrackListItem, column: TrackListColumn) -> bool {
        self.entity_uid == item.entity_uid && self.column == column
    }

    /// Finish editing and apply the text if it has been modified.
    fn apply(self, msg_tx: &MessageSender) {
        let Self {
            entity_uid,
            column,
            original_text,
            text,
            is_focus_requested: _,
        } = self;
        if text.trim() == original_text.trim() {
            return;
        }
        let Some(fields) = column.edited_fields(text) else {
            return;
        };
        send_track_edit(msg_tx, entity_uid, fields);
    }
}

fn send_track_edit(msg_tx: &MessageSender, entity_uid: TrackUid, fields: MultiTrackFields) {
    match MultiTrackEdit::new(fields, CustomTagsDelta::default()) {
        Ok(edit) => msg_tx.send_action(TrackEditAction::Apply {
            entity_uid,
            edit: Box::new(edit),
        }),
        Err(errors) => log::error!("Invalid edit of track {entity_uid}: {errors:?}"),
    }
}

/// A genre bubble that is dragged to another position within its table cell.
#[derive(Debug, Clone)]
struct DraggedGenre {
    entity_uid: TrackUid,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TrackListInteraction<'a> {
    pub(super) edited_entity_uid: Option<&'a aoide::TrackUid>,
    pub(super) track_selection: Option<&'a TrackSelection>,
    pub(super) is_track_editor_draft_modified: bool,
    /// The error of the last edit that was applied directly in the table.
    pub(super) edit_error: Option<&'a str>,
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
        super::render::help_icon(
            ui,
            "Click text to edit it (Enter applies, Esc aborts).\n\
             Click other cells or right-click to open the track editor.\n\
             Ctrl+click to select multiple tracks, Shift+click to select a range.\n\
             Drag genres or column headers to reorder them.",
        );
    });
    if let Some(edit_error) = interaction.edit_error {
        ui.colored_label(ui.visuals().error_fg_color, edit_error);
    }
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
                    &mut table_state.cell_edit,
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
/// could be cancelled by pressing the escape key.
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
    cell_edit: &mut Option<CellEdit>,
) {
    let TrackListInteraction {
        edited_entity_uid,
        track_selection,
        is_track_editor_draft_modified,
        edit_error: _,
    } = interaction;
    let is_edited = edited_entity_uid == Some(&item.entity_uid);
    let is_selected = track_selection.is_some_and(|selection| selection.contains(&item.entity_uid));
    // Prevent losing unsaved changes unintentionally and modifying
    // tracks while they are updated.
    let is_editable = !is_track_editor_draft_modified
        && !track_selection.is_some_and(TrackSelection::is_updating);
    row.set_selected(is_edited || is_selected);
    let mut is_cell_edit_started = false;
    // The contents of a cell might receive the click instead of the cell.
    let mut is_contents_clicked = false;
    for &column in &columns.selected {
        let (_, response) = row.col(|ui| {
            is_contents_clicked |= render_track_list_cell(
                ui,
                msg_tx,
                column,
                item,
                row_height,
                is_editable,
                cell_edit,
            );
        });
        if !is_editable || !response.clicked() || response.ctx.input(|i| !i.modifiers.is_none()) {
            continue;
        }
        let Some(text) = column.editable_text(item) else {
            continue;
        };
        is_cell_edit_started = true;
        // The previously edited cell might have been rendered before this one.
        // Otherwise it would be discarded before noticing that it lost the focus.
        if let Some(previous_cell_edit) = cell_edit.take() {
            previous_cell_edit.apply(msg_tx);
        }
        *cell_edit = Some(CellEdit {
            entity_uid: item.entity_uid.clone(),
            column,
            original_text: text.clone(),
            text,
            is_focus_requested: false,
        });
    }
    let response = row.response();
    if is_track_editor_draft_modified {
        if !is_edited {
            response.on_hover_text("Save or revert the changes of the edited track first.");
        }
        return;
    }
    if !is_editable {
        return;
    }
    if !is_edited {
        response.context_menu(|ui| {
            if ui.button("Open track editor").clicked() {
                msg_tx.send_action(TrackEditAction::Open(item.entity_uid.clone()));
                ui.close();
            }
        });
    }
    if is_cell_edit_started || !(response.clicked() || is_contents_clicked) {
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
    msg_tx: &MessageSender,
    column: TrackListColumn,
    item: &TrackListItem,
    row_height: f32,
    is_editable: bool,
    cell_edit: &mut Option<CellEdit>,
) -> bool {
    if cell_edit
        .as_ref()
        .is_some_and(|cell_edit| cell_edit.is_cell(item, column))
    {
        render_cell_edit(ui, msg_tx, cell_edit);
        return false;
    }
    match column {
        TrackListColumn::Artwork => render_artwork_cell(ui, item, row_height),
        TrackListColumn::Unsynchronized => {
            if !item.is_synchronized {
                let text = RichText::new(UNSYNCHRONIZED_MARKER).color(ui.visuals().warn_fg_color);
                ui.add(Label::new(text).selectable(false))
                    .on_hover_text("Modified, but not written into the file yet.");
            }
        }
        TrackListColumn::Genres => return render_genres_cell(ui, msg_tx, item, is_editable),
        _ => {
            if let Some(text) = column.cell_text(item) {
                // Selectable labels would swallow the clicks on the row.
                // Truncated text is available as a tooltip.
                ui.add(Label::new(text).selectable(false).truncate());
            }
        }
    }
    false
}

/// Render the text field of the edited table cell.
///
/// The text is applied when the text field loses the focus, i.e. by pressing
/// enter, clicking elsewhere, or scrolling the table cell out of view.
/// Pressing escape discards the text.
fn render_cell_edit(ui: &mut Ui, msg_tx: &MessageSender, cell_edit: &mut Option<CellEdit>) {
    let Some(edit) = cell_edit else {
        return;
    };
    let response = ui.add(
        TextEdit::singleline(&mut edit.text)
            .frame(Frame::new().fill(ui.visuals().extreme_bg_color))
            .margin(0)
            .desired_width(f32::INFINITY),
    );
    if !edit.is_focus_requested {
        response.request_focus();
        edit.is_focus_requested = true;
        return;
    }
    // The text field also loses the focus when pressing escape.
    if ui.input(|input| input.key_pressed(Key::Escape)) {
        *cell_edit = None;
        return;
    }
    if !response.has_focus()
        && let Some(edit) = cell_edit.take()
    {
        edit.apply(msg_tx);
    }
}

/// Render the genres as bubbles that could be dragged to reorder them.
///
/// Returns `true` if a bubble has been clicked.
fn render_genres_cell(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    item: &TrackListItem,
    is_editable: bool,
) -> bool {
    let genres = &item.genres;
    let is_movable = is_editable && genres.len() > 1;
    let dragged_index = DragAndDrop::payload::<DraggedGenre>(ui.ctx())
        .filter(|dragged| dragged.entity_uid == item.entity_uid)
        .map(|dragged| dragged.index);
    let font_id = TextStyle::Body.resolve(ui.style());
    let text_color = ui.visuals().text_color();
    let mut is_clicked = false;
    let bubble_rects = genres
        .iter()
        .enumerate()
        .map(|(index, genre)| {
            let galley = ui
                .painter()
                .layout_no_wrap(genre.clone(), font_id.clone(), text_color);
            let padding = Vec2::new(GENRE_BUBBLE_HORIZONTAL_PADDING, 0.0);
            // Only sensing drags would prevent the table cell from being clicked.
            let sense = if is_movable {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            };
            let (rect, response) = ui.allocate_exact_size(galley.size() + 2.0 * padding, sense);
            if is_movable {
                response.dnd_set_drag_payload(DraggedGenre {
                    entity_uid: item.entity_uid.clone(),
                    index,
                });
                is_clicked |= response.clicked();
                if response.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
            }
            let fill = if dragged_index == Some(index) {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.inactive.bg_fill
            };
            ui.painter()
                .rect_filled(rect, CornerRadius::same(GENRE_BUBBLE_CORNER_RADIUS), fill);
            ui.painter().galley(rect.min + padding, galley, text_color);
            rect
        })
        .collect::<Vec<_>>();
    let Some(dragged_index) = dragged_index else {
        return is_clicked;
    };
    let Some(pointer_pos) = ui.input(|input| input.pointer.interact_pos()) else {
        return is_clicked;
    };
    let insertion_index = column_insertion_index(&bubble_rects, pointer_pos.x);
    let Some(target_index) = move_target_index(dragged_index, insertion_index, genres.len()) else {
        return is_clicked;
    };
    let marker_x = bubble_rects.get(insertion_index).map_or_else(
        || {
            bubble_rects.last().map_or(pointer_pos.x, |rect| {
                rect.right() + ui.spacing().item_spacing.x / 2.0
            })
        },
        |rect| rect.left() - ui.spacing().item_spacing.x / 2.0,
    );
    ui.painter().vline(
        marker_x,
        ui.max_rect().y_range(),
        Stroke::new(COLUMN_DROP_MARKER_WIDTH, ui.visuals().selection.bg_fill),
    );
    if ui.input(|input| input.pointer.any_released()) {
        let mut reordered = genres.iter().map(String::as_str).collect::<Vec<_>>();
        let genre = reordered.remove(dragged_index);
        reordered.insert(target_index, genre);
        let fields = MultiTrackFields {
            genres: Some(reordered.join(LINE_SEPARATOR)),
            ..Default::default()
        };
        send_track_edit(msg_tx, item.entity_uid.clone(), fields);
    }
    is_clicked
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
