-- Your SQL goes here

ALTER TABLE IF EXISTS tags
    ADD COLUMN IF NOT EXISTS for_whole_text boolean NOT NULL DEFAULT false;