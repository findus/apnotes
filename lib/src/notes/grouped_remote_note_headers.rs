use std::collections::HashSet;
use notes::traits::identifyable_note::IdentifiableNote;
use notes::remote_note_header_collection::RemoteNoteHeaderCollection;

/// The note headers fetched from the server, grouped by uuid
pub type GroupedRemoteNoteHeaders = HashSet<RemoteNoteHeaderCollection>;

impl IdentifiableNote for GroupedRemoteNoteHeaders {

    fn folder(&self) -> String {
        self.iter().map(|note| note.folder()).last().unwrap()
    }

    fn uuid(&self) -> String {
        self.iter().map(|note| note.uuid()).last().unwrap()
    }

    fn first_subject(&self) -> String {
        match self.iter().next() {
            None => "".to_string(),
            Some(body) => body.first_subject()
        }
    }
}