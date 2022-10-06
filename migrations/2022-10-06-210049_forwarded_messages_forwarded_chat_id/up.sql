-- Your SQL goes here

ALTER TABLE IF EXISTS forwarded_messages
    ADD COLUMN IF NOT EXISTS forwarded_chat_id bigint NOT NULL DEFAULT -1;