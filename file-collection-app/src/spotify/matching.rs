// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fuzzy matching of tracks by artist, title, and duration.

use std::{collections::BTreeSet, time::Duration};

/// Words that carry no information for matching.
const NOISE_WORDS: &[&str] = &[
    "feat",
    "featuring",
    "ft",
    "remaster",
    "remastered",
    "version",
    "digital",
    "original",
    "explicit",
    "the",
    "a",
    "and",
];

/// Separators between multiple artist names, all lowercase.
const ARTIST_SEPARATORS: &[&str] = &[
    ",",
    ";",
    "/",
    " & ",
    " + ",
    " x ",
    " and ",
    " with ",
    " vs. ",
    " vs ",
    " feat. ",
    " feat ",
    " featuring ",
    " ft. ",
    " ft ",
];

/// Minimum similarity of the titles.
const MIN_TITLE_SCORE: f64 = 0.6;

/// Minimum similarity of the artists.
const MIN_ARTIST_SCORE: f64 = 0.5;

/// Minimum combined similarity of titles and artists.
const MIN_TOTAL_SCORE: f64 = 0.7;

/// Durations within this tolerance are considered equal.
const DURATION_TOLERANCE: Duration = Duration::from_secs(2);

/// Absolute lower bound for [`MAX_DURATION_DIFF_RATIO`].
const MIN_MAX_DURATION_DIFF: Duration = Duration::from_secs(30);

/// Reject candidates that deviate by more than this fraction of the
/// shorter duration, e.g. an extended mix vs. a radio edit.
const MAX_DURATION_DIFF_RATIO: f64 = 0.1;

/// Weight of the duration similarity for ranking candidates.
const DURATION_WEIGHT: f64 = 0.2;

#[must_use]
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[must_use]
fn significant_tokens(s: &str) -> BTreeSet<String> {
    tokenize(s)
        .into_iter()
        .filter(|token| !NOISE_WORDS.contains(&token.as_str()))
        // Years of releases or remasters.
        .filter(|token| !(token.len() == 4 && token.chars().all(|c| c.is_ascii_digit())))
        .collect()
}

#[must_use]
fn jaccard(lhs: &BTreeSet<String>, rhs: &BTreeSet<String>) -> f64 {
    if lhs.is_empty() && rhs.is_empty() {
        return 1.0;
    }
    let intersection = lhs.intersection(rhs).count();
    let union = lhs.union(rhs).count();
    #[expect(clippy::cast_precision_loss)]
    let score = intersection as f64 / union as f64;
    score
}

/// Normalized title, split into the core title and the version.
///
/// Local files typically use "Title (Extended Mix)" while Spotify
/// uses "Title - Extended Mix".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NormalizedTitle {
    core: Vec<String>,
    version: BTreeSet<String>,
}

impl NormalizedTitle {
    #[must_use]
    pub(super) fn new(title: &str) -> Self {
        let core_end = [title.find('('), title.find('['), title.find(" - ")]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(title.len());
        let (core, version) = title.split_at(core_end);
        let mut core = tokenize(core);
        if core.is_empty() {
            // Don't discard the whole title.
            core = tokenize(title);
        }
        Self {
            core,
            version: significant_tokens(version),
        }
    }

    /// The core title for use as an index key or search query.
    #[must_use]
    pub(super) fn core_key(&self) -> String {
        self.core.join(" ")
    }

    #[must_use]
    fn score(&self, other: &Self) -> f64 {
        let version_score = jaccard(&self.version, &other.version);
        if self.core == other.core {
            return 0.7 + 0.3 * version_score;
        }
        let self_tokens = self
            .core
            .iter()
            .cloned()
            .chain(self.version.iter().cloned());
        let other_tokens = other
            .core
            .iter()
            .cloned()
            .chain(other.version.iter().cloned());
        0.6 * jaccard(&self_tokens.collect(), &other_tokens.collect())
    }
}

/// Split a single artist string into multiple normalized artist names.
#[must_use]
fn split_artists(artist: &str) -> Vec<String> {
    let mut names = vec![format!(" {} ", artist.to_lowercase())];
    for separator in ARTIST_SEPARATORS {
        names = names
            .iter()
            .flat_map(|name| name.split(separator))
            .map(|name| format!(" {} ", name.trim()))
            .collect();
    }
    names
        .into_iter()
        .map(|name| tokenize(&name).join(" "))
        .filter(|name| !name.is_empty())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NormalizedArtists {
    names: Vec<String>,
    tokens: BTreeSet<String>,
}

impl NormalizedArtists {
    #[must_use]
    pub(super) fn new<'a>(artists: impl IntoIterator<Item = &'a str>) -> Self {
        let names: Vec<_> = artists.into_iter().flat_map(split_artists).collect();
        let tokens = names
            .iter()
            .flat_map(|name| significant_tokens(name))
            .collect();
        Self { names, tokens }
    }

    /// The main artist for use in search queries.
    #[must_use]
    pub(super) fn main_artist(&self) -> Option<&str> {
        self.names.first().map(String::as_str)
    }

    #[must_use]
    fn score(&self, other: &Self) -> f64 {
        let Some(main_artist) = self.main_artist() else {
            return 0.0;
        };
        if other.names.iter().any(|name| name == main_artist) {
            return 0.8 + 0.2 * jaccard(&self.tokens, &other.tokens);
        }
        0.8 * jaccard(&self.tokens, &other.tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NormalizedTrack {
    pub(super) title: NormalizedTitle,
    pub(super) artists: NormalizedArtists,
    pub(super) duration: Option<Duration>,
}

/// Similarity of durations in the range 0.0..=1.0.
///
/// Returns `None` if the durations differ too much.
#[must_use]
fn duration_score(lhs: Duration, rhs: Duration) -> Option<f64> {
    let diff = lhs.abs_diff(rhs);
    if diff <= DURATION_TOLERANCE {
        return Some(1.0);
    }
    let max_diff = lhs
        .min(rhs)
        .mul_f64(MAX_DURATION_DIFF_RATIO)
        .max(MIN_MAX_DURATION_DIFF);
    if diff > max_diff {
        return None;
    }
    Some(
        1.0 - diff.saturating_sub(DURATION_TOLERANCE).as_secs_f64()
            / max_diff.saturating_sub(DURATION_TOLERANCE).as_secs_f64(),
    )
}

impl NormalizedTrack {
    #[must_use]
    pub(super) fn new<'a>(
        title: &str,
        artists: impl IntoIterator<Item = &'a str>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            title: NormalizedTitle::new(title),
            artists: NormalizedArtists::new(artists),
            duration,
        }
    }

    /// Similarity score in the range 0.0..=1.0.
    ///
    /// Returns `None` if the tracks don't match.
    #[must_use]
    pub(super) fn match_score(&self, other: &Self) -> Option<f64> {
        let title_score = self.title.score(&other.title);
        if title_score < MIN_TITLE_SCORE {
            return None;
        }
        let artist_score = self.artists.score(&other.artists);
        if artist_score < MIN_ARTIST_SCORE {
            return None;
        }
        let total_score = 0.5 * title_score + 0.5 * artist_score;
        if total_score < MIN_TOTAL_SCORE {
            return None;
        }
        let (Some(self_duration), Some(other_duration)) = (self.duration, other.duration) else {
            // Unknown durations neither reject nor penalize candidates.
            return Some(total_score);
        };
        let duration_score = duration_score(self_duration, other_duration)?;
        Some(total_score * (1.0 - DURATION_WEIGHT + DURATION_WEIGHT * duration_score))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(local: (&str, &str), remote: (&str, &[&str])) -> Option<f64> {
        score_with_durations(local, None, remote, None)
    }

    fn score_with_durations(
        local: (&str, &str),
        local_duration_secs: Option<u64>,
        remote: (&str, &[&str]),
        remote_duration_secs: Option<u64>,
    ) -> Option<f64> {
        let (local_artist, local_title) = local;
        let (remote_title, remote_artists) = remote;
        NormalizedTrack::new(
            local_title,
            [local_artist],
            local_duration_secs.map(Duration::from_secs),
        )
        .match_score(&NormalizedTrack::new(
            remote_title,
            remote_artists.iter().copied(),
            remote_duration_secs.map(Duration::from_secs),
        ))
    }

    #[test]
    fn split_artists_with_separators() {
        assert_eq!(
            vec!["daft punk", "pharrell williams", "nile rodgers"],
            split_artists("Daft Punk feat. Pharrell Williams & Nile Rodgers")
        );
    }

    #[test]
    fn artist_name_with_separator() {
        // Both sides are split equally.
        assert!(score(("AC/DC", "Thunderstruck"), ("Thunderstruck", &["AC/DC"])).is_some());
    }

    #[test]
    fn exact_match() {
        let exact = score(("Daft Punk", "Get Lucky"), ("Get Lucky", &["Daft Punk"])).unwrap();
        assert!(exact > 0.99);
    }

    #[test]
    fn version_notation() {
        let same_version = score(
            ("Artist", "Title (Extended Mix)"),
            ("Title - Extended Mix", &["Artist"]),
        )
        .unwrap();
        let other_version = score(
            ("Artist", "Title (Extended Mix)"),
            ("Title - Radio Edit", &["Artist"]),
        )
        .unwrap();
        assert!(same_version > other_version);
    }

    #[test]
    fn remaster_is_noise() {
        let remaster = score(
            ("Queen", "Bohemian Rhapsody"),
            ("Bohemian Rhapsody - Remastered 2011", &["Queen"]),
        )
        .unwrap();
        assert!(remaster > 0.99);
    }

    #[test]
    fn featured_artists() {
        assert!(
            score(
                ("Daft Punk feat. Pharrell Williams", "Get Lucky"),
                (
                    "Get Lucky (feat. Pharrell Williams and Nile Rodgers)",
                    &["Daft Punk", "Pharrell Williams", "Nile Rodgers"]
                ),
            )
            .is_some()
        );
    }

    #[test]
    fn different_artist() {
        assert!(score(("Daft Punk", "Get Lucky"), ("Get Lucky", &["Someone Else"])).is_none());
    }

    #[test]
    fn different_title() {
        assert!(
            score(
                ("Daft Punk", "Get Lucky"),
                ("One More Time", &["Daft Punk"])
            )
            .is_none()
        );
    }

    #[test]
    fn duration_within_tolerance() {
        let local = ("Artist", "Title");
        let remote: (&str, &[&str]) = ("Title", &["Artist"]);
        let exact = score_with_durations(local, Some(240), remote, Some(241)).unwrap();
        assert!(exact > 0.99);
    }

    #[test]
    fn closer_duration_ranks_higher() {
        let local = ("Artist", "Title");
        let remote: (&str, &[&str]) = ("Title", &["Artist"]);
        let close = score_with_durations(local, Some(240), remote, Some(245)).unwrap();
        let far = score_with_durations(local, Some(240), remote, Some(260)).unwrap();
        assert!(close > far);
    }

    #[test]
    fn reject_different_duration() {
        // Extended mix vs. radio edit.
        assert!(
            score_with_durations(
                ("Artist", "Title"),
                Some(420),
                ("Title", &["Artist"]),
                Some(210)
            )
            .is_none()
        );
    }

    #[test]
    fn unknown_duration_is_neutral() {
        let local = ("Artist", "Title");
        let remote: (&str, &[&str]) = ("Title", &["Artist"]);
        assert_eq!(
            score(local, remote),
            score_with_durations(local, None, remote, Some(210))
        );
    }

    #[test]
    fn duration_score_bounds() {
        let secs = Duration::from_secs;
        assert_eq!(Some(1.0), duration_score(secs(200), secs(202)));
        // The max. difference is 30 s for short tracks.
        assert!(duration_score(secs(200), secs(230)).is_some_and(|score| score.abs() < 1e-9));
        assert!(duration_score(secs(200), secs(231)).is_none());
        // The max. difference is 10 % for long tracks.
        assert!(duration_score(secs(600), secs(660)).is_some());
        assert!(duration_score(secs(600), secs(661)).is_none());
    }
}
