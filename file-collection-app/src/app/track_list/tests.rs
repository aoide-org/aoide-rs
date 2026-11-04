// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{cell::RefCell, sync::mpsc};

use aoide::{EntityUid, TrackUid};
use egui::{
    CentralPanel, ColorImage, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect,
    TextureOptions, Vec2,
};

use crate::app::{Action, LibraryAction, Message};

use super::*;

#[test]
fn every_column_has_a_unique_id_that_can_be_parsed() {
    for column in TrackListColumn::ALL {
        assert_eq!(Some(column), TrackListColumn::from_id(column.id()));
    }
    let ids = TrackListColumn::ALL.map(TrackListColumn::id);
    assert_eq!(ids.len(), ids.iter().unique().count());
}

#[test]
fn newly_selected_columns_are_appended() {
    let mut columns = TrackListColumns {
        selected: vec![TrackListColumn::Key, TrackListColumn::Artwork],
    };
    assert!(columns.toggle(TrackListColumn::Title));
    assert_eq!(
        vec![
            TrackListColumn::Key,
            TrackListColumn::Artwork,
            TrackListColumn::Title
        ],
        columns.selected
    );
    assert!(columns.toggle(TrackListColumn::Artwork));
    assert_eq!(
        vec![TrackListColumn::Key, TrackListColumn::Title],
        columns.selected
    );
}

#[test]
fn move_column() {
    use TrackListColumn::{Artist as A, Bpm as B, Key as C, Title as D};
    let new_columns = || TrackListColumns {
        selected: vec![A, B, C, D],
    };
    let moved = |column, insertion_index| {
        let mut columns = new_columns();
        assert!(columns.move_column(column, insertion_index));
        columns.selected
    };
    assert_eq!(vec![B, C, A, D], moved(A, 3));
    assert_eq!(vec![B, C, D, A], moved(A, 4));
    assert_eq!(vec![D, A, B, C], moved(D, 0));
    assert_eq!(vec![A, C, B, D], moved(C, 1));
}

#[test]
fn moving_column_without_effect() {
    let mut columns = TrackListColumns {
        selected: vec![TrackListColumn::Artist, TrackListColumn::Title],
    };
    let unmodified = columns.clone();
    // Inserting before or after itself.
    assert!(!columns.move_column(TrackListColumn::Artist, 0));
    assert!(!columns.move_column(TrackListColumn::Artist, 1));
    assert!(!columns.move_column(TrackListColumn::Title, 2));
    // Out of bounds.
    assert!(!columns.move_column(TrackListColumn::Artist, 3));
    // Not selected.
    assert!(!columns.move_column(TrackListColumn::Key, 0));
    assert_eq!(unmodified, columns);
}

#[test]
fn last_selected_column_cannot_be_deselected() {
    let mut columns = TrackListColumns {
        selected: vec![TrackListColumn::Title],
    };
    assert!(!columns.toggle(TrackListColumn::Title));
    assert_eq!(vec![TrackListColumn::Title], columns.selected);
}

#[test]
fn json_roundtrip() {
    let columns = TrackListColumns {
        selected: vec![TrackListColumn::Title, TrackListColumn::FilePath],
    };
    assert_eq!(
        Some(columns.clone()),
        TrackListColumns::from_json(&columns.to_json())
    );
}

#[test]
fn from_json_sanitizes_ids() {
    assert_eq!(
        Some(TrackListColumns {
            selected: vec![TrackListColumn::Key, TrackListColumn::Title],
        }),
        TrackListColumns::from_json(br#"["key", "unknown", "title", "key"]"#)
    );
}

#[test]
fn from_json_rejects_contents_without_any_column() {
    assert_eq!(None, TrackListColumns::from_json(b"[]"));
    assert_eq!(None, TrackListColumns::from_json(br#"["unknown"]"#));
    assert_eq!(None, TrackListColumns::from_json(b"not json"));
}

fn new_item(ctx: &Context, title: &str) -> TrackListItem {
    TrackListItem {
        entity_uid: TrackUid::from_untyped(EntityUid::random()),
        content_url: None,
        is_synchronized: true,
        artist: None,
        title: Some(title.to_owned()),
        album_artist: None,
        album_title: None,
        album_subtitle: None,
        copyright: None,
        advisory_rating: None,
        grouping: None,
        comment: None,
        genres: Vec::new(),
        year: None,
        bpm: None,
        key: None,
        artwork_thumbnail_texture: ctx.load_texture(
            "",
            ColorImage::filled([4, 4], egui::Color32::BLACK),
            TextureOptions::LINEAR,
        ),
    }
}

struct Harness {
    ctx: Context,
    msg_tx: MessageSender,
    msg_rx: mpsc::Receiver<Message>,
    columns: TrackListColumns,
    table_state: RefCell<TrackListTableState>,
    track_list: Vec<TrackListItem>,
}

impl Harness {
    fn new(columns: TrackListColumns, titles: &[&str]) -> Self {
        let ctx = Context::default();
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx.clone(), msg_tx);
        let track_list = titles.iter().map(|title| new_item(&ctx, title)).collect();
        Self {
            ctx,
            msg_tx,
            msg_rx,
            columns,
            table_state: RefCell::default(),
            track_list,
        }
    }

    /// Render a single frame.
    fn frame(&self, mut events: Vec<Event>, modifiers: Modifiers, could_fetch_more: bool) {
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 600.0))),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| {
            CentralPanel::default().show(ui, |ui| {
                render_track_list(
                    ui,
                    &self.msg_tx,
                    &self.columns,
                    &mut self.table_state.borrow_mut(),
                    &self.track_list,
                    could_fetch_more,
                    TrackListInteraction {
                        edited_entity_uid: None,
                        track_selection: None,
                        is_track_editor_draft_modified: false,
                    },
                );
            });
        });
        // No renderer is available for applying the textures.
        output.textures_delta.clear();
    }

    fn click(&self, pos: Pos2, modifiers: Modifiers) {
        let pointer_event = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers,
        };
        self.frame(vec![Event::PointerMoved(pos)], modifiers, false);
        self.frame(vec![pointer_event(true)], modifiers, false);
        self.frame(vec![pointer_event(false)], modifiers, false);
        self.frame(Vec::new(), modifiers, false);
    }

    fn drag(&self, from: Pos2, to: Pos2) {
        let pointer_event = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let frame = |event| self.frame(vec![event], Modifiers::NONE, false);
        frame(Event::PointerMoved(from));
        frame(pointer_event(from, true));
        // Exceed the drag threshold in a separate step.
        frame(Event::PointerMoved(from + Vec2::new(10.0, 0.0)));
        frame(Event::PointerMoved(to));
        frame(pointer_event(to, false));
        self.frame(Vec::new(), Modifiers::NONE, false);
    }

    fn actions(&self) -> Vec<LibraryAction> {
        self.msg_rx
            .try_iter()
            .map(|msg| {
                let Message::Action(Action::Library(action)) = msg else {
                    panic!("unexpected message: {msg:?}");
                };
                action
            })
            .collect()
    }
}

/// A position inside the title cell of the first row, right on top of the
/// text of the label.
const FIRST_ROW_LABEL_POS: Pos2 = Pos2::new(20.0, 62.0);

fn title_only_harness() -> Harness {
    Harness::new(
        TrackListColumns {
            selected: vec![TrackListColumn::Title],
        },
        &["First", "Second", "Third"],
    )
}

#[test]
fn clicking_on_a_row_opens_the_track_editor() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    let actions = harness.actions();
    let [LibraryAction::TrackEdit(TrackEditAction::Open(entity_uid))] = actions.as_slice() else {
        panic!("unexpected actions: {actions:?}");
    };
    assert_eq!(harness.track_list[0].entity_uid, *entity_uid);
}

#[test]
fn clicking_on_a_row_with_modifiers_updates_the_selection() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::COMMAND);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSelection(TrackSelectionAction::Toggle(
                _
            ))]
        ),
        "unexpected actions: {actions:?}"
    );
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::SHIFT);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSelection(
                TrackSelectionAction::SelectRange(_)
            )]
        ),
        "unexpected actions: {actions:?}"
    );
}

#[test]
fn fetches_more_only_if_the_end_of_the_list_is_visible() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, true);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSearch(TrackSearchAction::FetchMore)]
        ),
        "unexpected actions: {actions:?}"
    );

    let titles = vec!["Title"; 1000];
    let harness = Harness::new(TrackListColumns::default(), &titles);
    harness.frame(Vec::new(), Modifiers::NONE, true);
    assert!(harness.actions().is_empty());
}

#[test]
fn renders_all_columns_without_panicking() {
    let harness = Harness::new(
        TrackListColumns {
            selected: TrackListColumn::ALL.to_vec(),
        },
        &["First", "Second"],
    );
    // The initial sizing pass is followed by regular frames.
    for _ in 0..3 {
        harness.frame(Vec::new(), Modifiers::NONE, false);
    }
}

#[test]
fn switching_columns_does_not_panic() {
    let mut harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.columns = TrackListColumns::default();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.columns.toggle(TrackListColumn::Year);
    harness.frame(Vec::new(), Modifiers::NONE, false);
}

#[test]
fn columns_menu_stays_open_for_toggling_multiple_columns() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    // Open the menu that is located in the top left corner.
    harness.click(Pos2::new(24.0, 16.0), Modifiers::NONE);
    assert!(harness.actions().is_empty());
    // Click repeatedly while moving downwards along the entries of the menu.
    let mut toggled = Vec::new();
    for y in (24..200).step_by(4) {
        harness.click(Pos2::new(30.0, y as f32), Modifiers::NONE);
        for action in harness.msg_rx.try_iter() {
            let Message::Action(Action::TrackListColumns(TrackListColumnsAction::Toggle(column))) =
                action
            else {
                panic!("unexpected message: {action:?}");
            };
            toggled.push(column);
        }
    }
    assert!(toggled.contains(&TrackListColumn::Artwork), "{toggled:?}");
    assert!(
        toggled.contains(&TrackListColumn::Unsynchronized),
        "{toggled:?}"
    );
    // The only selected column cannot be deselected.
    assert!(!toggled.contains(&TrackListColumn::Title), "{toggled:?}");
}

#[test]
fn saved_columns_are_restored() {
    let settings_dir = std::env::temp_dir().join(format!(
        "aoide_track_list_columns_test_{pid}",
        pid = std::process::id()
    ));
    fs::create_dir_all(&settings_dir).unwrap();
    assert_eq!(
        TrackListColumns::default(),
        TrackListColumns::restore(&settings_dir)
    );
    let mut columns = TrackListColumns::default();
    assert!(columns.toggle(TrackListColumn::Year));
    assert!(columns.toggle(TrackListColumn::Artwork));
    assert!(columns.move_column(TrackListColumn::Year, 0));
    columns.save(&settings_dir).unwrap();
    assert_eq!(columns, TrackListColumns::restore(&settings_dir));
    // Invalid contents are replaced by the defaults.
    fs::write(TrackListColumns::file_path(&settings_dir), b"invalid").unwrap();
    assert_eq!(
        TrackListColumns::default(),
        TrackListColumns::restore(&settings_dir)
    );
    fs::remove_dir_all(&settings_dir).unwrap();
}

#[test]
fn rows_remain_clickable_if_the_columns_exceed_the_available_width() {
    let harness = Harness::new(
        TrackListColumns {
            selected: TrackListColumn::ALL.to_vec(),
        },
        &["First"],
    );
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(Pos2::new(900.0, FIRST_ROW_LABEL_POS.y), Modifiers::NONE);
    assert_eq!(1, harness.actions().len());
}

/// The vertical center of the table header.
const HEADER_Y: f32 = 44.0;

/// Cells at x = [8..168], [176..416], and [424..].
fn three_columns_harness() -> Harness {
    let harness = Harness::new(
        TrackListColumns {
            selected: vec![
                TrackListColumn::Artist,
                TrackListColumn::Title,
                TrackListColumn::Key,
            ],
        },
        &["First"],
    );
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness
}

fn drag_slowly(harness: &Harness, from: Pos2, to: Pos2) {
    let pointer_event = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    let frame = |event| harness.frame(vec![event], Modifiers::NONE, false);
    frame(Event::PointerMoved(from));
    frame(pointer_event(from, true));
    for step in 1..=20 {
        frame(Event::PointerMoved(
            from + (to - from) * (step as f32 / 20.0),
        ));
    }
    frame(pointer_event(to, false));
    harness.frame(Vec::new(), Modifiers::NONE, false);
}

fn move_actions(harness: &Harness) -> Vec<(TrackListColumn, usize)> {
    harness
        .msg_rx
        .try_iter()
        .filter_map(|msg| match msg {
            Message::Action(Action::TrackListColumns(TrackListColumnsAction::Move {
                column,
                insertion_index,
            })) => Some((column, insertion_index)),
            _ => None,
        })
        .collect()
}

#[test]
fn column_insertion_index_depends_on_the_center_of_the_headers() {
    let rects = [
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 10.0)),
        Rect::from_min_max(Pos2::new(100.0, 0.0), Pos2::new(200.0, 10.0)),
    ];
    assert_eq!(0, column_insertion_index(&rects, -10.0));
    assert_eq!(0, column_insertion_index(&rects, 49.0));
    assert_eq!(1, column_insertion_index(&rects, 51.0));
    assert_eq!(1, column_insertion_index(&rects, 149.0));
    assert_eq!(2, column_insertion_index(&rects, 151.0));
    assert_eq!(2, column_insertion_index(&rects, 2000.0));
}

#[test]
fn dragging_a_header_moves_the_column() {
    let harness = three_columns_harness();
    // Drop onto the right half of the title column.
    harness.drag(Pos2::new(50.0, HEADER_Y), Pos2::new(380.0, HEADER_Y));
    assert_eq!(vec![(TrackListColumn::Artist, 2)], move_actions(&harness));
}

#[test]
fn columns_could_be_dropped_at_any_vertical_position() {
    for drop_y in [4.0, 20.0, HEADER_Y, 60.0, 120.0, 400.0, 590.0] {
        let harness = three_columns_harness();
        // The grabbed position could be on top of the text.
        drag_slowly(
            &harness,
            Pos2::new(16.0, HEADER_Y),
            Pos2::new(380.0, drop_y),
        );
        assert_eq!(
            vec![(TrackListColumn::Artist, 2)],
            move_actions(&harness),
            "dropped at y = {drop_y}"
        );
    }
}

#[test]
fn dropping_beyond_the_headers_moves_the_column_to_the_end_or_start() {
    let harness = three_columns_harness();
    drag_slowly(&harness, Pos2::new(50.0, HEADER_Y), Pos2::new(995.0, 300.0));
    assert_eq!(vec![(TrackListColumn::Artist, 3)], move_actions(&harness));

    let harness = three_columns_harness();
    drag_slowly(&harness, Pos2::new(500.0, HEADER_Y), Pos2::new(1.0, 300.0));
    assert_eq!(vec![(TrackListColumn::Key, 0)], move_actions(&harness));
}

#[test]
fn consecutive_moves_are_applied() {
    let mut harness = three_columns_harness();
    for (from_x, to_x, expected) in [
        (
            50.0,
            380.0,
            [
                TrackListColumn::Title,
                TrackListColumn::Artist,
                TrackListColumn::Key,
            ],
        ),
        (
            500.0,
            20.0,
            [
                TrackListColumn::Key,
                TrackListColumn::Title,
                TrackListColumn::Artist,
            ],
        ),
    ] {
        drag_slowly(
            &harness,
            Pos2::new(from_x, HEADER_Y),
            Pos2::new(to_x, 200.0),
        );
        let actions = move_actions(&harness);
        assert_eq!(1, actions.len(), "{actions:?}");
        for (column, insertion_index) in actions {
            assert!(harness.columns.move_column(column, insertion_index));
        }
        assert_eq!(expected.as_slice(), harness.columns.selected.as_slice());
        harness.frame(Vec::new(), Modifiers::NONE, false);
    }
}

#[test]
fn dragging_a_row_does_not_move_columns() {
    let harness = three_columns_harness();
    drag_slowly(
        &harness,
        FIRST_ROW_LABEL_POS,
        Pos2::new(380.0, FIRST_ROW_LABEL_POS.y),
    );
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn dragging_a_header_onto_itself_has_no_effect() {
    let harness = three_columns_harness();
    harness.drag(Pos2::new(50.0, HEADER_Y), Pos2::new(120.0, HEADER_Y));
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn dragging_the_border_of_a_header_resizes_the_column() {
    let harness = three_columns_harness();
    harness.drag(Pos2::new(172.0, HEADER_Y), Pos2::new(212.0, HEADER_Y));
    assert_eq!(0, harness.msg_rx.try_iter().count());
    let width = harness.table_state.borrow().widths[&TrackListColumn::Artist];
    assert!(width > 180.0, "{width}");
}

#[test]
fn moving_columns_preserves_their_widths() {
    let mut harness = three_columns_harness();
    harness.drag(Pos2::new(172.0, HEADER_Y), Pos2::new(212.0, HEADER_Y));
    let width_before = harness.table_state.borrow().widths[&TrackListColumn::Artist];
    assert!(harness.columns.move_column(TrackListColumn::Artist, 2));
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.frame(Vec::new(), Modifiers::NONE, false);
    let width_after = harness.table_state.borrow().widths[&TrackListColumn::Artist];
    assert!(
        (width_before - width_after).abs() < 1.0,
        "{width_before} {width_after}"
    );
}
