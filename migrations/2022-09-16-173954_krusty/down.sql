-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS tag_to_media;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS media;
DROP TYPE tag_type;
DROP TYPE media_type;