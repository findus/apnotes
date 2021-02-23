use model::NotesMetadata;
use profile;
use model::Body;
use std::hash::Hasher;
use notes::note_headers::NoteHeaders;
use notes::remote_note_metadata::RemoteNoteMetaData;
use notes::remote_note_header_collection::RemoteNoteHeaderCollection;
use notes::traits::identifyable_note::IdentifyableNote;
use notes::traits::mergeable_note_body::MergeableNoteBody;
use notes::traits::header_parser::HeaderParser;
use std::collections::HashSet;
use quoted_printable::ParseMode;

#[derive(Eq,Clone,Debug)]
pub struct LocalNote {
    pub metadata: NotesMetadata,
    pub body: Vec<Body>,
}

impl LocalNote {
    pub(crate) fn needs_merge(&self) -> bool {
        self.body.len() > 1
    }
    //TODO right not it only works for merged notes
    pub fn to_header_vector(&self) -> NoteHeaders {
        let mut headers: Vec<(String,String)> = vec![];
        let profile = profile::load_profile();
        headers.push(("X-Uniform-Type-Identifier".to_string(), "com.apple.mail-note".to_string()));
        headers.push(("Content-Type".to_string(), "text/html; charset=utf-8".to_string()));
        headers.push(("Content-Transfer-Encoding".to_string(), "quoted-printable".to_string()));
        headers.push(("Mime-Version".to_string(), "1.0 (Mac OS X Notes 4.6 \\(879.10\\))".to_string()));
        headers.push(("Date".to_string(), self.metadata.date.clone()));
        headers.push(("X-Mail-Created-Date".to_string(), self.metadata.date.clone()));
        headers.push(("From".to_string(), profile.email)); //todo implement in noteheader
        headers.push(("Message-Id".to_string(), self.body.first().unwrap().message_id.clone()));
        headers.push(("X-Universally-Unique-Identifier".to_string(), self.metadata.uuid.clone()));
        headers.push(("Subject".to_string(), self.body.first().unwrap().subject().clone()));
        headers
    }

    pub fn to_remote_metadata(&self) -> RemoteNoteMetaData {
        RemoteNoteMetaData {
            headers: self.to_header_vector(),
            folder: self.folder(),
            uid: self.body.first().unwrap().uid.unwrap_or(-1)
        }
    }

    pub fn content_changed_locally(&self) -> bool {
        self.body.iter().filter(|body| body.old_remote_message_id != None).next() != None
    }

    pub fn changed_remotely(&self, remote_metadata: &RemoteNoteHeaderCollection) -> bool {

        if remote_metadata.len() != self.body.len() {
            return true;
        }

        let remote_message_ids:Vec<String> = remote_metadata
            .iter()
            .map(|e| e.headers.message_id())
            .collect();

        self.body.iter()
            .filter(|local_body| remote_message_ids.contains(&local_body.message_id))
            .count() != self.body.len()
    }

    pub fn all_old_message_ids(&self) -> Option<HashSet<String>> {
        if self.needs_merge() == false && self.content_changed_locally() {
            return Some(self.body.first().unwrap().old_remote_message_id.clone().unwrap().split(",").map(|e| e.to_string()).collect());
        } else {
            return None;
        }
    }
}

impl IdentifyableNote for LocalNote {

    fn folder(&self) -> String {
        let decoded = quoted_printable::decode(&self.metadata.subfolder, ParseMode::Robust).unwrap();
        String::from_utf8(decoded).unwrap()
    }

    fn uuid(&self) -> String {
        self.metadata.uuid()
    }

    fn first_subject(&self) -> String {
        match self.body.first() {
            None => "".to_string(),
            Some(body) => body.subject()
        }
    }
}

impl MergeableNoteBody for LocalNote {

    fn needs_merge(&self) -> bool {
        self.body.len() > 1
    }

    fn get_message_id(&self) -> Option<String> {
        if self.needs_merge() {
            None
        } else {
            return Some(self.body[0].message_id.clone());
        }
    }

    fn all_message_ids(&self) -> HashSet<String> {
        self.body.iter().map(|b| b.message_id.clone()).collect()
    }
}

impl std::cmp::PartialEq for LocalNote  {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.uuid == other.metadata.uuid
    }

    fn ne(&self, other: &Self) -> bool {
        self.metadata.uuid != other.metadata.uuid
    }
}

impl std::hash::Hash for LocalNote {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.metadata.uuid.hash(state);
    }
}
