
use note::{NoteHeader, HeaderParser};

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
    pub mime_version: String,
    pub subject: String
}

impl NotesMetadata {
    pub fn new(header: NoteHeader, subfolder: String, uid: u32) -> Self {
        NotesMetadata {
            old_remote_id: None,
            subfolder,
            locally_deleted: false,
            uid: Some(uid as i64),
            new: false,
            date: header.date(),
            uuid: header.identifier(),
            mime_version: header.mime_version(),
            subject: header.subject()
        }
    }
}

#[derive(Queryable)]
pub struct Body {
    pub message_id: String,
    pub body: String,
    pub metaData: NotesMetadata
}