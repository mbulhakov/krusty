-- This file should undo anything in `up.sql`

ALTER TABLE IF EXISTS tags
    DROP COLUMN IF EXISTS for_whole_text;