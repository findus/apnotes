use notes::remote_note_metadata::RemoteNoteMetaData;
use notes::traits::mergeable_note_body::MergeableNoteBody;
use notes::traits::identifyable_note::IdentifiableNote;
use notes::traits::header_parser::HeaderParser;
use std::collections::HashSet;

/// A collection of remote note metadata that share the
/// same uuid
pub type RemoteNoteHeaderCollection = Vec<RemoteNoteMetaData>;

impl MergeableNoteBody for RemoteNoteHeaderCollection {
    fn needs_merge(&self) -> bool {
        self.len() > 1
    }

    /// Returns the message-id of the Remote Note
    /// Returns None if note needs to be merged
    fn get_message_id(&self) -> Option<String> {
        match self.needs_merge() {
            true => None,
            false => {
                Some(self.iter().last()
                    .expect("At least one Element must be present")
                    .headers.message_id())
            }
        }
    }

    fn all_message_ids(&self) -> HashSet<String> {
        self.iter()
            .map(|n| n.headers.message_id())
            .collect()
    }

}

impl IdentifiableNote for RemoteNoteHeaderCollection {

    fn folder(&self) -> String {
        self.iter().last().expect("At least one Element must be present").headers.folder()
    }

    fn uuid(&self) -> String {
        self.iter().last().expect("At least one Element must be present").headers.uuid()
    }

    fn first_subject(&self) -> String {
        match self.first() {
            Some(e) => e.headers.subject(),
            None => "".to_string()
        }
    }
}