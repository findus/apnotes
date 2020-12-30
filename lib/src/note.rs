extern crate mailparse;
extern crate html2runes;
extern crate log;

use std::fs::File;

use std::hash::Hasher;
use util;
use std::path::PathBuf;
use self::log::{trace};
use model::NotesMetadata;

pub type NoteHeader = Vec<(String, String)>;

impl HeaderParser for NoteHeader {
    fn get_header_value(&self, search_string: &str) -> Option<String> {
        self.iter()
            .find(|(key, _)| key == search_string)
            .and_then(|val| Some(val.1.clone()))
    }

    fn date(&self) -> String {
        match self.get_header_value("Date") {
            Some(date) => date,
            _ => panic!("Could not get date of Note {:?}", self)
        }
    }

    fn subject(&self) -> String {
        match self.get_header_value("Subject") {
            Some(subject) => subject,
            _ => panic!("Could not get subject of Note {:?}", self)
        }
    }

    fn identifier(&self) -> String {
        match self.get_header_value("X-Universally-Unique-Identifier") {
            Some(subject) => subject,
            _ => panic!("Could not get uuid of this note {:#?}", self)
        }
    }

    fn subject_with_identifier(&self) -> String {
        if self.uid.is_none() {
            format!("{}_{}","new", self.subject_escaped())
        } else {
            format!("{}_{}", self.uid.unwrap(), self.subject_escaped())
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
            _ =>  panic!("Could not get Subject of this note {:?}", self)
        }
    }

    fn message_id(&self) -> String {
        match self.get_header_value("Message-Id") {
            Some(subject) => subject,
            _ =>  panic!("Could not get Message-Id of this note {:?}", self)
        }
    }

    fn mime_version(&self) -> String {
        match self.get_header_value("Mime-Version") {
            Some(subject) => subject,
            _ =>  panic!("Could not get Mime-Version of this note {:?}", self)
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
    fn date(&self) -> String;
    fn mime_version(&self) -> String;
}

pub trait NoteTrait {
    fn metadata(&self) -> NotesMetadata;
    fn folder(&self) -> String;
    fn body(&self) -> String;
    fn uuid(&self) -> String;
}

pub struct LocalNote {
    pub path: PathBuf,
    pub metadata: NotesMetadata
}

impl LocalNote {
    pub fn new(path: PathBuf) -> LocalNote {
        let metadata_file_path = util::get_hash_path(&path);
        trace!("{} - {}", &path.to_string_lossy(), &metadata_file_path.to_string_lossy());
        let metadata_file = File::open(&metadata_file_path).expect(
            &format!("Could not load metadata_file at {:?}", &metadata_file_path)
        );

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

    fn folder(&self) -> String { self.metadata().subfolder.clone() }

    fn metadata(&self) -> NotesMetadata{ self.metadata.clone() }
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

    fn folder(&self) -> String { self.folder.clone() }

    fn metadata(&self) -> NotesMetadata{ self.mail_headers.clone() }
}

impl std::cmp::PartialEq for Box<dyn NoteTrait>  {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid().clone()
    }

    fn ne(&self, other: &Self) -> bool {
        self.uuid() != other.uuid().clone()
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
        self.identifier() == other.identifier() && self.uid == other.uid
    }

    fn ne(&self, other: &Self) -> bool {
        self.identifier() != other.identifier() && self.uid == other.uid
    }
}

impl std::cmp::Eq for NotesMetadata {

}

impl std::hash::Hash for NotesMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier().hash(state);
        self.uid.hash(state);
    }
}

