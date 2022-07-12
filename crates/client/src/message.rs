// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
