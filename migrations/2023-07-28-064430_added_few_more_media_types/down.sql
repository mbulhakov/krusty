-- This file should undo anything in `up.sql`

DELETE FROM media WHERE type IN (  -- not the best solution; unknown should have been added from the start
    'animation',
    'plain_text',
    'document',
    'video_note',
    'sticker',
    'unknown');

ALTER TYPE media_type RENAME TO media_type_old;

CREATE TYPE media_type AS ENUM(
    'voice',
    'video',
    'picture');

ALTER TABLE media ALTER COLUMN type TYPE media_type USING (type::text::media_type);

DROP TYPE media_type_old;