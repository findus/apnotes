table! {
    body (message_id) {
        old_remote_message_id -> Nullable<Text>,
        message_id -> Text,
        text -> Nullable<Text>,
        uid -> Nullable<BigInt>,
        metadata_uuid -> Text,
    }
}

table! {
    metadata (uuid) {
        subfolder -> Text,
        locally_deleted -> Bool,
        new -> Bool,
        edited -> Bool,
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
