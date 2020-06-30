extern crate mailparse;
extern crate html2runes;


use std::fs::File;
use walkdir::DirEntry;
use std::hash::Hasher;

#[derive(Serialize,Deserialize)]
pub struct NotesMetadata {
    pub header: Vec<(String, String)>,
    pub old_remote_id: String
}

impl HeaderParser for NotesMetadata {
    fn get_header_value(&self, search_string: &str) -> Option<String> {
        self.header
            .iter()
            .find(|(key, _)| key == search_string)
            .and_then(|val| Some(val.1.clone()))
    }
}

pub trait HeaderParser {
    fn get_header_value(&self, search_string: &str) -> Option<String>;
}

pub trait NoteTrait {
    fn body(&self) -> String;
    fn subject(&self) -> String;
    fn identifier(&self) -> String;
    fn subject_with_identifier(&self) -> String;
}

pub(crate) struct LocalNote {
    pub path: DirEntry,
    pub metadata: NotesMetadata
}

impl LocalNote {
    pub fn new(path: DirEntry) -> LocalNote {
        let folder = path.path().parent().unwrap().to_string_lossy().into_owned();
        let new_file_name = format!(".{}_hash",path.file_name().to_string_lossy().into_owned());

        let metadata_file_path = format!("{}/{}",&folder,&new_file_name).to_owned();
        let hash_loc_path = std::path::Path::new(&metadata_file_path).to_owned();

        let metadata_file = File::open(hash_loc_path).unwrap();

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

    fn subject(&self) -> String {
        self.path.file_name().to_str().unwrap().to_string()
    }

    fn identifier(&self) -> String {
        match self.metadata.get_header_value("X-Universally-Unique-Identifier") {
            Some(subject) => subject,
            _ => panic!("Could not get Identifier of LocalNote {}", self.subject())
        }
    }

    fn subject_with_identifier(&self) -> String {
        format!("{}_{}",self.identifier(), self.subject())
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

    fn subject(&self) -> String {
        match self.mail_headers.get_header_value("Subject") {
            Some(subject) => format!("{}", subject).replace("/", "_").replace(" ", "_"),
            _ => "no_subject".to_string()
        }
    }

    // X-Universally-Unique-Identifier
    fn identifier(&self) -> String {
        match self.mail_headers.get_header_value("X-Universally-Unique-Identifier") {
            Some(subject) => subject,
            _ => panic!("Could not get Identifier of Note {}", self.subject())
        }
    }

    fn subject_with_identifier(&self) -> String {
        format!("{}_{}",self.identifier(), self.subject())
    }
}

impl std::cmp::PartialEq for Box<dyn NoteTrait>  {
    fn eq(&self, other: &Self) -> bool {
        self.subject() == other.subject().as_ref()
    }

    fn ne(&self, other: &Self) -> bool {
        self.subject() != other.subject().as_ref()
    }
}

impl std::cmp::Eq for Box<dyn NoteTrait> {

}

impl std::hash::Hash for Box<dyn NoteTrait> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.subject().hash(state)
    }
}

