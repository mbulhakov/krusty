CREATE TYPE media_type AS ENUM (
    'voice',
    'video',
    'picture'
);

CREATE TYPE tag_type AS ENUM (
    'ordinary',
    'regexp'
);

CREATE TABLE media (
    id serial PRIMARY KEY,
    name character varying(255) NOT NULL,
    type media_type NOT NULL,
    data bytea NOT NULL
);

CREATE TABLE tags (
    id serial PRIMARY KEY,
    text character varying(255) NOT NULL,
    type tag_type NOT NULL
);

CREATE TABLE tag_to_media (
    tag_id serial NOT NULL,
    media_id serial NOT NULL,
    CONSTRAINT fk_tags
        FOREIGN KEY(tag_id) 
        REFERENCES tags(id),
    CONSTRAINT fk_media
        FOREIGN KEY(media_id) 
        REFERENCES media(id),
    PRIMARY KEY (tag_id, media_id)
);
