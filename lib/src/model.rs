
#[derive(Clone,Queryable,Deserialize,Serialize)]
pub struct NotesMetadata {
    pub old_remote_id: Option<String>,
    pub subfolder: String,
    pub locally_deleted: bool,

    //IMAP UID
    pub uid: Option<i64>,
    pub new: bool,

    pub date: String, //TODO type
    pub uuid: String,
    pub mime_version: String
}

#[derive(Queryable)]
pub struct Body {
    pub message_id: String,
    pub body: String,
    pub metaData: NotesMetadata
}