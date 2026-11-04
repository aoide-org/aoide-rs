// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::tag::{Label, PlainTag, Score};

use super::sort_tags_by_descending_score;

fn tag(label: &'static str, score: f64) -> PlainTag<'static> {
    PlainTag {
        label: Label::clamp_from(label),
        score: Score::new_unchecked(score),
    }
}

#[test]
fn sort_tags_by_descending_score_restores_the_order_and_is_stable() {
    let mut tags = vec![
        tag("Ambient", 0.5625),
        tag("Disco", 0.75),
        tag("House", 1.0),
        tag("Techno", 0.75),
    ];
    sort_tags_by_descending_score(&mut tags);
    let labels = tags
        .iter()
        .map(|tag| tag.label.as_ref().unwrap().as_str())
        .collect::<Vec<_>>();
    assert_eq!(vec!["House", "Disco", "Techno", "Ambient"], labels);
}
