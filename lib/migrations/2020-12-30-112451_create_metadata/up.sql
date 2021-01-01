-- Your SQL goes here

CREATE TABLE metadata (
    old_remote_id VARCHAR,
    subfolder VARCHAR NOT NULL,
    locally_deleted BOOLEAN NOT NULL,
    locally_edited BOOLEAN NOT NULL,
    new BOOLEAN NOT NULL,
    date TIMESTAMP NOT NULL,
    uuid VARCHAR PRIMARY KEY NOT NULL,
    mime_version VARCHAR NOT NULL
);

CREATE TABLE body (
    message_id VARCHAR PRIMARY KEY NOT NULL,
    text VARCHAR,
    uid BIGINT,
    metadata_uuid VARCHAR NOT NULL,
    FOREIGN KEY(metadata_uuid) REFERENCES metadata(uuid)
);