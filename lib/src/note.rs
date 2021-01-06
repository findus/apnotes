extern crate mailparse;
extern crate html2runes;
extern crate log;

use std::hash::Hasher;
use model::{NotesMetadata, Body};
use std::collections::HashSet;
use builder::{HeaderBuilder, NotesMetadataBuilder};
use profile;

#[derive(Eq,Clone,Debug)]
pub struct LocalNote {
    pub metadata: NotesMetadata,
    pub body: Vec<Body>,
}

pub type NoteHeaders = Vec<(String,String)>;

#[derive(Clone,Eq,Debug)]
pub struct RemoteNoteMetaData {
    pub(crate) headers: NoteHeaders,
    pub(crate) folder: String,
    pub(crate) uid: i64
}

impl LocalNote {
    pub(crate) fn needs_merge(&self) -> bool {
        self.body.len() > 1
    }
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
}

impl IdentifyableNote for LocalNote {

    fn folder(&self) -> String {
        self.metadata.folder()
    }

    fn uuid(&self) -> String {
        self.metadata.uuid()
    }
}

/// A collection of remote note metadata that share the
/// same uuid
pub type RemoteNoteHeaderCollection = Vec<RemoteNoteMetaData>;

impl IdentifyableNote for RemoteNoteHeaderCollection {

    fn folder(&self) -> String {
        self.iter().last().expect("At least one Element must be present").headers.folder()
    }

    fn uuid(&self) -> String {
        self.iter().last().expect("At least one Element must be present").headers.uuid()
    }
}

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
                uid: body.uid.unwrap()
            }
        }).collect()

    }
}

impl HeaderParser for NoteHeaders {
    fn get_header_value(&self, search_string: &str) -> Option<String> {
        self.iter()
            .find(|(key, _)| key == search_string)
            .and_then(|val| Some(val.1.clone()))
    }

    fn subject(&self) -> String {
        match self.get_header_value("Subject") {
            Some(subject) => subject,
            _ => panic!("Could not get subject of Note {:?}", self.uuid())
        }
    }

    fn uuid(&self) -> String {
        match self.get_header_value("X-Universally-Unique-Identifier") {
            Some(subject) => subject,
            _ => panic!("Could not get uuid of this note {:?}", self.uuid())
        }
    }

    ///
    /// Prints an espaced subject, removes any character that might cause problems when
    /// writing files to disk
    ///
    /// Every Filename should include the title of the note, only saving the file with the uuid
    /// would be quite uncomfortable, with the title, the user has a tool to quickly skim or
    /// search through the notes with only using the terminal or explorer.
    ///
    fn subject_escaped(&self) -> String {
        let regex = regex::Regex::new(r#"[.<>:\\"/\|?*]"#).unwrap();
        match self.get_header_value("Subject") {
            Some(subject) => {
                let escaped_string = format!("{}", subject)
                    .replace("/", "_").replace(" ", "_");
                   // .replace(|c: char| !c.is_ascii(), "");
                regex.replace_all(&escaped_string, "").into_owned()
            },
            _ =>  panic!("Could not get Subject of this note {:?}", self.uuid())
        }
    }

    fn message_id(&self) -> String {
        match self.get_header_value("Message-Id") {
            Some(subject) => subject,
            _ =>  panic!("Could not get Message-Id of this note {:?}", self.uuid())
        }
    }

    fn date(&self) -> String {
        match self.get_header_value("Date") {
            Some(date) => date,
            _ => panic!("Could not get date of Note {:?}", self.uuid())
        }
    }

    fn mime_version(&self) -> String {
        match self.get_header_value("Mime-Version") {
            Some(subject) => subject,
            _ =>  panic!("Could not get Mime-Version of this note {:?}", self.uuid())
        }
    }

    fn folder(&self) -> String {
        match self.get_header_value("Folder") {
            Some(folder) => folder,
            _ => panic!("Could not get folder of this note {:?}", self.uuid())
        }
    }

    fn imap_uid(&self) -> i64 {
        match self.get_header_value("Uid") {
            Some(uid) => uid.parse::<i64>().unwrap(),
            _ => panic!("Could not get folder of this note {:#?}", self.uuid())
        }
    }
}

pub trait HeaderParser {
    fn get_header_value(&self, search_string: &str) -> Option<String>;
    fn subject(&self) -> String;
    fn uuid(&self) -> String;
    fn subject_escaped(&self) -> String;
    fn message_id(&self) -> String;
    fn date(&self) -> String;
    fn mime_version(&self) -> String;
    fn folder(&self) -> String;
    fn imap_uid(&self) -> i64;
}

pub trait IdentifyableNote {
    fn folder(&self) -> String;
    fn uuid(&self) -> String;
}

impl IdentifyableNote for NotesMetadata {

    fn folder(&self) -> String { self.subfolder.clone() }
    fn uuid(&self) -> String {
        self.uuid.clone()
    }
}

impl std::hash::Hash for Box<dyn IdentifyableNote> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state)
    }
}

impl std::cmp::PartialEq for Box<dyn IdentifyableNote>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid()
    }
}

impl std::cmp::PartialEq for NotesMetadata  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid != other.uuid
    }
}

impl std::hash::Hash for NotesMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl std::cmp::PartialEq for Body  {
    fn eq(&self, other: &Self) -> bool {
        self.message_id == other.message_id && self.metadata_uuid == other.metadata_uuid
    }

    fn ne(&self, other: &Self) -> bool {
        self.message_id != other.message_id || self.metadata_uuid != other.metadata_uuid
    }
}

impl std::hash::Hash for Body {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.message_id.hash(state);
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

impl std::cmp::Eq for Box<&dyn IdentifyableNote> {}

impl std::cmp::PartialEq for Box<&dyn IdentifyableNote>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid()
    }
}

impl std::hash::Hash for Box<&dyn IdentifyableNote> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state);
    }
}