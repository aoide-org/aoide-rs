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

#[cfg(feature = "fmt-aiff")]
pub mod aiff;

#[cfg(feature = "fmt-flac")]
pub mod flac;

#[cfg(feature = "fmt-mp3")]
pub mod mp3;

#[cfg(feature = "fmt-mp4")]
pub mod mp4;

#[cfg(feature = "fmt-ogg")]
pub mod ogg;

#[cfg(feature = "fmt-opus")]
pub mod opus;

#[cfg(feature = "fmt-wav")]
pub mod wav;

#[cfg(any(feature = "fmt-mp3", feature = "fmt-aiff", feature = "fmt-wav"))]
pub mod id3;

#[cfg(any(feature = "fmt-flac", feature = "fmt-ogg", feature = "fmt-opus"))]
pub mod vorbis;
