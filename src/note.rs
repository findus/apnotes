extern crate mailparse;
extern crate html2runes;

use std::path::Path;
use std::fs::File;
use walkdir::DirEntry;
use std::hash::{Hash, Hasher};

pub trait NoteTrait {
    fn hash(&self) -> u64;
    fn body(&self) -> String;
    fn subject(&self) -> String;
    fn identifier(&self) -> String;
}

pub struct LocalNote {
    pub path: DirEntry
}

impl NoteTrait for LocalNote {

    fn hash(&self) -> u64 {
        0
    }

    fn body(&self) -> String {
        " ".to_string()
    }

    fn subject(&self) -> String {
        self.path.file_name().to_str().unwrap().to_string()
    }

    fn identifier(&self) -> String {
        let subject = self.subject();
        let folder = self.path.path().parent().unwrap().file_name().unwrap().to_string_lossy();
        format!("{}_{}", folder, subject)
    }
}

pub struct Note {
    pub mail_headers: Vec<(String, String)>,
    pub folder: String,
    pub body: String,
    pub hash: u64,
    pub uid: u32,
}

impl NoteTrait for Note {

    fn hash(&self) -> u64 {
        self.hash
    }

    fn body(&self) -> String {
        self.body.clone()
    }

    fn subject(&self) -> String {
        let subject = match self.mail_headers.iter().find(|(x, _y)| x.eq("Subject")) {
            Some((_subject, name)) => format!("{}-{}", self.uid, name).replace("/", "_").replace(" ", "_"),
            _ => "no_subject".to_string()
        };
        subject
    }

    fn identifier(&self) -> String {
        let subject = self.subject();
        format!("{}_{}", self.folder, subject)
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

