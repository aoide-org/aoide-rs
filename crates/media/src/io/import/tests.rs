// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[test]
fn import_replay_gain_valid() {
    let mut importer = Importer::new();
    assert_eq!(
        Some(LoudnessLufs(-8.49428)),
        importer.import_replay_gain("-9.50572 dB")
    );
    assert_eq!(
        Some(LoudnessLufs(-8.49428)),
        importer.import_replay_gain(" -9.50572db ")
    );
    assert_eq!(
        Some(LoudnessLufs(-18.178062)),
        importer.import_replay_gain("0.178062 DB")
    );
    assert_eq!(
        Some(LoudnessLufs(-18.178062)),
        importer.import_replay_gain("  +0.178062   dB ")
    );
    assert!(importer.finish().into_messages().is_empty());
}

#[test]
fn import_replay_gain_invalid() {
    let mut importer = Importer::new();
    assert!(importer.import_replay_gain("-9.50572").is_none());
    assert!(importer.import_replay_gain("- 9.50572 dB").is_none());
    assert!(importer.import_replay_gain("+ 0.178062 dB").is_none());
    assert!(importer.import_replay_gain("+0.178062").is_none());
}

#[test]
#[cfg(feature = "fmt-mp3")]
#[ignore] // a hack for debugging purposes
fn import_tmp_test_mp3() {
    let path = Path::new("/tmp/test.mp3");
    let file = Box::new(std::fs::File::open(path).unwrap());
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let input = NewTrackInput {
        collected_at: aoide_core::util::clock::DateTime::now_utc(),
        content_rev: None,
    };
    let mime = guess_mime_from_path(path).unwrap();
    let mut track = input.into_new_track(Default::default(), mime);
    let issues = import_into_track(&mut reader, &Default::default(), &mut track).unwrap();
    assert!(issues.is_empty());
}
