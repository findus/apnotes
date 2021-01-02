use model::{Body, NotesMetadata};
use util::generate_uuid;
use note::RemoteNoteMetaData;

pub struct NoteTupleBuilder {
    metadata: NotesMetadata,
    body: Body
}

impl NoteTupleBuilder {

}

pub struct BodyMetadataBuilder {
    body: Body
}
/// Builder for Body Objects, mostly for
/// testing purposes
///
/// If no own message-id gets provided it gets randomly
/// generated
impl BodyMetadataBuilder {
    pub fn new() -> BodyMetadataBuilder {
        BodyMetadataBuilder {
            body: Body {
                message_id: generate_uuid(),
                text: None,
                uid: None,
                metadata_uuid: "".to_string()
            }
        }
    }

    pub fn with_uid(mut self, uid: i64) -> Self {
        self.body.uid = Some(uid);
        self
    }

    pub fn with_metadata_uuid(mut self, uuid: &str) -> Self {
        self.body.metadata_uuid = uuid.to_string();
        self
    }

    pub fn with_message_id(mut self, message_id: &str) -> Self {
        self.body.message_id = message_id.to_string();
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.body.text = Some(text.to_string());
        self
    }

    pub fn build(mut self) -> Body {
        self.body
    }
}

pub struct NotesMetadataBuilder {
    notesMetadata: NotesMetadata
}

/// Builder for Metadata Objects, mostly for
/// testing purposes
///
/// If no own uuid gets provided it gets randomly
/// generated
impl NotesMetadataBuilder {
    pub fn new() -> NotesMetadataBuilder {
        NotesMetadataBuilder {
            notesMetadata:  NotesMetadata {
                old_remote_id: None,
                subfolder: "".to_string(),
                locally_deleted: false,
                locally_edited: false,
                new: false,
                date: "".to_string(),
                uuid: generate_uuid(),
                mime_version: "".to_string()
            }
        }

    }

    pub fn with_uuid(mut self, uuid: String) -> Self {
        self.notesMetadata.uuid = uuid;
        self
    }

    pub fn is_new(mut self, new: bool) -> Self {
        self.notesMetadata.new = new;
        self
    }

    pub fn is_flagged_for_deletion(mut self, del: bool) -> Self {
        self.notesMetadata.locally_deleted = del;
        self
    }

    pub fn with_folder(mut self, folder: String) -> Self {
        self.notesMetadata.subfolder = folder;
        self
    }

    pub fn build(self) -> NotesMetadata {
        self.notesMetadata
    }
    
    pub fn build_as_remote_data(self) -> RemoteNoteMetaData {
        RemoteNoteMetaData {
            headers: vec![],
            folder: "".to_string(),
            uid: 0
        }
    }
}