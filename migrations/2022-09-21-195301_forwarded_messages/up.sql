-- Your SQL goes here

CREATE TABLE IF NOT EXISTS forwarded_messages (
    id serial PRIMARY KEY,
    chat_id bigint NOT NULL,
    forwarded_message_id integer NOT NULL,
    message_url character varying(255) NOT NULL
);

CREATE TYPE media_feature_type AS ENUM (
    'text_trigger',
    'duplicated_forwarded_message_detection'
);

CREATE TABLE IF NOT EXISTS media_to_feature (
    id serial PRIMARY KEY,
    media_id INT NOT NULL,
    feature_type media_feature_type NOT NULL,
    CONSTRAINT fk_media_to_feature_media
        FOREIGN KEY(media_id) 
        REFERENCES media(id)
);

INSERT INTO media_to_feature(media_id, feature_type) (SELECT id, 'text_trigger' from media);