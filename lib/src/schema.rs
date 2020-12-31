table! {
    body (message_id) {
        message_id -> Text,
        text -> Nullable<Text>,
        uid -> Nullable<BigInt>,
        metadata_uuid -> Nullable<Text>,
    }
}

table! {
    metadata (uuid) {
        old_remote_id -> Nullable<Text>,
        subfolder -> Text,
        locally_deleted -> Bool,
        new -> Bool,
        date -> Timestamp,
        uuid -> Text,
        mime_version -> Text,
    }
}

joinable!(body -> metadata (metadata_uuid));

allow_tables_to_appear_in_same_query!(
    body,
    metadata,
);
