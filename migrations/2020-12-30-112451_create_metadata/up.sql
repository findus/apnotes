-- Your SQL goes here

CREATE TABLE metadata (
    subfolder VARCHAR NOT NULL,
    locally_deleted BOOLEAN NOT NULL,
    new BOOLEAN NOT NULL,
    date TIMESTAMP NOT NULL,
    uuid VARCHAR PRIMARY KEY NOT NULL,
    mime_version VARCHAR NOT NULL
);

CREATE TABLE body (
    old_remote_message_id VARCHAR,
    message_id VARCHAR PRIMARY KEY NOT NULL,
    text VARCHAR,
    uid BIGINT,
    metadata_uuid VARCHAR NOT NULL,
    FOREIGN KEY(metadata_uuid) REFERENCES metadata(uuid)
);