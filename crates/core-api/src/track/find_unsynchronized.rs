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

use aoide_core::{
    entity::{Entity, EntityRevision},
    media::content::ContentLink,
};

use crate::{filtering::StringPredicate, media::source::ResolveUrlFromContentPath};

#[derive(Debug, Clone, Default)]
pub struct Params {
    pub resolve_url_from_content_path: Option<ResolveUrlFromContentPath>,
    pub content_path_predicate: Option<StringPredicate>,
}

#[derive(Debug, Clone)]
pub struct UnsynchronizedTrack {
    pub content_link: ContentLink,
    pub last_synchronized_rev: Option<EntityRevision>,
}

pub type UnsynchronizedTrackEntity = Entity<(), UnsynchronizedTrack>;
