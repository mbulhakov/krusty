-- This file should undo anything in `up.sql`

ALTER TABLE IF EXISTS forwarded_messages
    DROP COLUMN IF EXISTS forwarded_chat_id;