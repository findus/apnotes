extern crate mailparse;
extern crate html2runes;
extern crate log;

use std::hash::{Hasher};
use builder::{HeaderBuilder};
use notes::note_headers::NoteHeaders;
use notes::localnote::LocalNote;
use notes::traits::header_parser::HeaderParser;

/// Data Structure that represents a remote note
#[derive(Clone,Eq,Debug)]
pub struct RemoteNoteMetaData {
    pub headers: NoteHeaders,
    pub folder: String,
    pub uid: i64
}

impl RemoteNoteMetaData {
    pub fn new(local_note: &LocalNote) -> Vec<RemoteNoteMetaData> {
        local_note.body.iter().map(|body| {
            let headers = HeaderBuilder::new()
                .with_subject(&body.subject())
                .with_uuid(local_note.metadata.uuid.clone())
                .with_message_id(body.message_id.clone())
                .build();

            RemoteNoteMetaData {
                headers,
                folder: local_note.metadata.subfolder.clone(),
                uid: body.uid.unwrap_or(-1)
            }
        }).collect()

    }
}

impl std::cmp::PartialEq for RemoteNoteMetaData  {
    fn eq(&self, other: &Self) -> bool {
        self.headers.uuid() == other.headers.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.headers.uuid() != other.headers.uuid()
    }
}

impl std::hash::Hash for RemoteNoteMetaData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.headers.uuid().hash(state);
    }
}

