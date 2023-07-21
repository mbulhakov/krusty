-- Your SQL goes here

CREATE TABLE IF NOT EXISTS cron_jobs (
    id serial PRIMARY KEY,
    pattern character varying(255) NOT NULL,
    chat_id bigint,
    caption character varying(255),
    description character varying(255)
);

CREATE TABLE IF NOT EXISTS media_to_cron_job (
    id serial PRIMARY KEY,
    media_id INT NOT NULL,
    cron_job_id INT NOT NULL,
    CONSTRAINT fk_media_to_cron_job
        FOREIGN KEY(media_id) 
        REFERENCES media(id),
        FOREIGN KEY(cron_job_id) 
        REFERENCES cron_jobs(id)
);