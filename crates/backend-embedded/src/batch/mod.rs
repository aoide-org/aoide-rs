// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod export_unsynchronized_tracks;
pub mod revert_unsynchronized_tracks;
pub mod synchronize_collection_vfs;
pub mod update_tracks;

#[cfg(feature = "tantivy")]
pub mod reindex_tracks;

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    #[default]
    Finished,
    Aborted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    /// The number of tracks to process.
    pub total: usize,

    /// The number of tracks that have been visited.
    pub finished: usize,
}

/// Process tracks one after another.
///
/// Each track is processed by the future returned from `process_track`.
/// Its result is then passed to `collect_result`. Reports the progress
/// initially and after each track. The abort flag is checked before each
/// track. `operation` describes the batch operation in log messages,
/// e.g. "update of tracks".
pub(crate) async fn run_batch<T, R, F>(
    operation: &str,
    tracks: Vec<T>,
    process_track: impl Fn(T) -> F,
    mut collect_result: impl FnMut(R),
    mut report_progress_fn: impl FnMut(Progress),
    abort_flag: &AtomicBool,
) -> Completion
where
    F: Future<Output = R>,
{
    let mut progress = Progress {
        total: tracks.len(),
        finished: 0,
    };
    report_progress_fn(progress);
    for track in tracks {
        if abort_flag.load(Ordering::Relaxed) {
            log::info!(
                "Aborting {operation} after {finished} of {total} track(s)",
                finished = progress.finished,
                total = progress.total
            );
            return Completion::Aborted;
        }
        collect_result(process_track(track).await);
        progress.finished += 1;
        report_progress_fn(progress);
    }
    Completion::Finished
}

#[cfg(test)]
mod tests {
    use std::{
        pin::pin,
        sync::atomic::{AtomicBool, Ordering},
        task::{Context, Poll, Waker},
    };

    use super::{Completion, Progress, run_batch};

    /// Run a future that never needs to wait.
    fn run_ready<F: Future>(future: F) -> F::Output {
        let mut context = Context::from_waker(Waker::noop());
        match pin!(future).poll(&mut context) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("future is pending"),
        }
    }

    #[test]
    fn run_batch_processes_all_tracks_and_reports_progress() {
        let abort_flag = AtomicBool::new(false);
        let mut collected = Vec::new();
        let mut reported = Vec::new();
        let completion = run_ready(run_batch(
            "test",
            vec![1, 2, 3],
            async |track| track * 10,
            |result| collected.push(result),
            |progress: Progress| reported.push(progress.finished),
            &abort_flag,
        ));
        assert_eq!(completion, Completion::Finished);
        assert_eq!(collected, [10, 20, 30]);
        assert_eq!(reported, [0, 1, 2, 3]);
    }

    #[test]
    fn run_batch_stops_when_aborted() {
        let abort_flag = AtomicBool::new(false);
        let mut collected = Vec::new();
        let mut reported = Vec::new();
        let completion = run_ready(run_batch(
            "test",
            vec![1, 2, 3, 4, 5],
            async |track| track,
            |result| {
                collected.push(result);
                if result == 2 {
                    abort_flag.store(true, Ordering::Relaxed);
                }
            },
            |progress: Progress| reported.push((progress.finished, progress.total)),
            &abort_flag,
        ));
        assert_eq!(completion, Completion::Aborted);
        assert_eq!(collected, [1, 2]);
        assert_eq!(reported, [(0, 5), (1, 5), (2, 5)]);
    }
}
