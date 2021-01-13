use std::collections::HashSet;
use notes::traits::identifyable_note::IdentifyableNote;
use notes::remote_note_header_collection::RemoteNoteHeaderCollection;

/// The note headers fetched from the server, grouped by uuid
pub type GroupedRemoteNoteHeaders = HashSet<RemoteNoteHeaderCollection>;

impl IdentifyableNote for GroupedRemoteNoteHeaders {

    fn folder(&self) -> String {
        self.iter().map(|note| note.folder()).last().unwrap()
    }

    fn uuid(&self) -> String {
        self.iter().map(|note| note.uuid()).last().unwrap()
    }

}