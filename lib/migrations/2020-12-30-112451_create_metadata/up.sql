-- Your SQL goes here

CREATE TABLE metadata (
    old_remote_id VARCHAR,
    subfolder VARCHAR NOT NULL,
    locally_deleted BOOLEAN NOT NULL, -- Bool
    uid BIGINT,
    new BOOLEAN NOT NULL, -- Bool
    date TIMESTAMP NOT NULL,
    uuid VARCHAR PRIMARY KEY NOT NULL,
    mime_version VARCHAR NOT NULL
);

CREATE TABLE body (
    text VARCHAR,
    message_id VARCHAR PRIMARY KEY NOT NULL,
    metadata VARCHAR,
    FOREIGN KEY(metadata) REFERENCES metadata(uuid)
);