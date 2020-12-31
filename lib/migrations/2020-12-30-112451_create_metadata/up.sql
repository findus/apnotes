-- Your SQL goes here

CREATE TABLE metadata (
    old_remote_id VARCHAR,
    subfolder VARCHAR NOT NULL,
    locally_deleted BOOLEAN NOT NULL, -- Bool
    new BOOLEAN NOT NULL, -- Bool
    date TIMESTAMP NOT NULL,
    uuid VARCHAR PRIMARY KEY NOT NULL,
    mime_version VARCHAR NOT NULL
);

CREATE TABLE body (
    message_id VARCHAR PRIMARY KEY NOT NULL,
    text VARCHAR,
    uid BIGINT,
    metadata_uuid VARCHAR,
    FOREIGN KEY(metadata_uuid) REFERENCES metadata(uuid)
);