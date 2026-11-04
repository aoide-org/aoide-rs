// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide::api::track::search::Params;

use super::*;

#[test]
fn state_changes_without_memo_changes_do_not_block_subsequent_changes() {
    let shared_state = SharedState::default();
    let mut memo_state = MemoState::default();

    // Nothing to apply and nobody would ever complete the pending state.
    assert!(memo_state.try_start_pending(&shared_state).is_none());
    assert!(matches!(memo_state, MemoState::Ready(_)));

    let mut params = Params {
        order: DEFAULT_SORT_ORDER.to_vec(),
        ..Default::default()
    };
    let _ = shared_state.update_params(&mut params);
    let (_, memo_diff) = memo_state
        .try_start_pending(&shared_state)
        .expect("subsequent change is not blocked");
    assert!(matches!(memo_diff, MemoDiff::Changed { .. }));
    assert!(matches!(memo_state, MemoState::Pending { .. }));
}
