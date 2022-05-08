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

pub mod source;
pub mod tracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Only import metadata once, never re-import.
    Once,

    /// Only (re-)import metadata from media source if modified
    /// and if the current track revision matches the synchronized
    /// revision.
    Modified,

    /// Only (re-)import metadata from media source if modified
    /// but regardless of the synchronized revision, i.e. allow to
    /// overwrite changed metadata with metadata imported from the
    /// media source for resynchronization.
    ModifiedResync,

    /// Always (re-)import metadata from media source, regardless
    /// of modification time and synchronization status.
    Always,
}
