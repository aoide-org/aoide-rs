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

/// A message is either an intent or an effect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<Intent, Effect> {
    Intent(Intent),
    Effect(Effect),
}

impl<Intent, Effect> Message<Intent, Effect> {
    pub fn from_intent(intent: impl Into<Intent>) -> Self {
        Self::Intent(intent.into())
    }

    pub fn from_effect(effect: impl Into<Effect>) -> Self {
        Self::Effect(effect.into())
    }
}
