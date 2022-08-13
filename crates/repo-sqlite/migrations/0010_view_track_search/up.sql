-- SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
-- SPDX-License-Identifier: AGPL-3.0-or-later

DROP VIEW IF EXISTS view_track_search;
CREATE VIEW view_track_search AS
SELECT
track.*,
media_source.collection_id,
media_source.collected_ms,
media_source.content_type,
media_source.content_link_path,
media_source.audio_duration_ms,
media_source.audio_channel_count,
media_source.audio_samplerate_hz,
media_source.audio_bitrate_bps,
media_source.audio_loudness_lufs,
media_source.advisory_rating
FROM track
JOIN media_source ON media_source.row_id=track.media_source_id;
