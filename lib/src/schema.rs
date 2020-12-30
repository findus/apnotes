table! {
    body (message_id) {
        text -> Nullable<Text>,
        message_id -> Text,
        metadata -> Nullable<Text>,
    }
}

table! {
    metadata (uuid) {
        old_remote_id -> Nullable<Text>,
        subfolder -> Text,
        locally_deleted -> Bool,
        uid -> Nullable<BigInt>,
        new -> Bool,
        date -> Timestamp,
        uuid -> Text,
        mime_version -> Text,
    }
}

joinable!(body -> metadata (metadata));

allow_tables_to_appear_in_same_query!(
    body,
    metadata,
);