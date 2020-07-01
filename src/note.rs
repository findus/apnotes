extern crate mailparse;
extern crate html2runes;


use std::fs::File;
use walkdir::DirEntry;
use std::hash::Hasher;
use converter;
use util;

#[derive(Serialize,Deserialize)]
pub struct NotesMetadata {
    pub header: Vec<(String, String)>,
    pub old_remote_id: Option<String>,
    pub subfolder: String,
    pub locally_deleted: bool,
    pub uid: u32
}

impl HeaderParser for NotesMetadata {
    fn get_header_value(&self, search_string: &str) -> Option<String> {
        self.header
            .iter()
            .find(|(key, _)| key == search_string)
            .and_then(|val| Some(val.1.clone()))
    }

    fn subject(&self) -> String {
        match self.get_header_value("Subject") {
            Some(subject) => subject,
            _ => panic!("Could not get Identifier of LocalNote {:?}", self.header)
        }
    }

    fn identifier(&self) -> String {
        match self.get_header_value("X-Universally-Unique-Identifier") {
            Some(subject) => subject,
            _ => panic!("Could not get uuid of this note {:#?}", self.header)
        }
    }

    fn subject_with_identifier(&self) -> String {
        format!("{}_{}",self.identifier(), self.subject_escaped())
    }

    fn subject_escaped(&self) -> String {
        match self.get_header_value("Subject") {
            Some(subject) => format!("{}", subject).replace("/", "_").replace(" ", "_"),
            _ =>  panic!("Could not get Subject of this note {:?}", self.header)
        }
    }

    fn message_id(&self) -> String {
        match self.get_header_value("Message-Id") {
            Some(subject) => subject,
            _ =>  panic!("Could not get Message-Id of this note {:?}", self.header)
        }
    }
}

pub trait HeaderParser {
    fn get_header_value(&self, search_string: &str) -> Option<String>;
    fn subject(&self) -> String;
    fn identifier(&self) -> String;
    fn subject_with_identifier(&self) -> String;
    fn subject_escaped(&self) -> String;
    fn message_id(&self) -> String;
}

pub trait NoteTrait {
    fn body(&self) -> String;
    fn uuid(&self) -> String;
}

pub struct LocalNote {
    pub path: DirEntry,
    pub metadata: NotesMetadata
}

impl LocalNote {
    pub fn new(path: DirEntry) -> LocalNote {
        let metadata_file_path = util::get_hash_path(path.path());
        let metadata_file = File::open(metadata_file_path).unwrap();

        LocalNote {
            metadata: serde_json::from_reader(metadata_file).unwrap(),
            path
        }
    }

}

impl NoteTrait for LocalNote {

    fn body(&self) -> String {
        " ".to_string()
    }

    fn uuid(&self) -> String {
        self.metadata.identifier()
    }
}

pub struct Note {
    pub mail_headers: NotesMetadata,
    pub folder: String,
    pub body: String,
}

impl NoteTrait for Note {

    fn body(&self) -> String {
        self.body.clone()
    }

    fn uuid(&self) -> String {
        self.mail_headers.identifier()
    }
}

impl std::cmp::PartialEq for Box<dyn NoteTrait>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid().as_ref()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid().as_ref()
    }
}

impl std::cmp::Eq for Box<dyn NoteTrait> {

}

impl std::hash::Hash for Box<dyn NoteTrait> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state)
    }
}



impl std::cmp::PartialEq for NotesMetadata  {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }

    fn ne(&self, other: &Self) -> bool {
        self.identifier() != other.identifier()
    }
}

impl std::cmp::Eq for NotesMetadata {

}

impl std::hash::Hash for NotesMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier().hash(state)
    }
}

