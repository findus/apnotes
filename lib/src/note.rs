extern crate mailparse;
extern crate html2runes;
extern crate log;



use std::hash::Hasher;



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

    fn folder(&self) -> String {
        match self.get_header_value("Folder") {
            Some(folder) => folder,
            _ => panic!("Could not get folder of this note {:#?}", self)
        }
    }

    fn uid(&self) -> i64 {
        match self.get_header_value("Uid") {
            Some(uid) => uid.parse::<i64>().unwrap(),
            _ => panic!("Could not get folder of this note {:#?}", self)
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
    fn subject_escaped(&self) -> String;
    fn message_id(&self) -> String;
    fn date(&self) -> String;
    fn mime_version(&self) -> String;
    fn folder(&self) -> String;
    fn uid(&self) -> i64;
}

pub trait NoteTrait {
    fn metadata(&self) -> NotesMetadata;
    fn folder(&self) -> String;
    fn body(&self) -> String;
    fn uuid(&self) -> String;
}

impl NoteTrait for NotesMetadata {
    fn metadata(&self) -> NotesMetadata { self.clone() }

    fn folder(&self) -> String { self.subfolder.clone() }

    fn body(&self) -> String {
        /*assert_eq!(self.needs_merge(), false);
        self.notes.first()
            .expect(&format!("No note found for {}", self.uuid.clone()))
            .body
            .clone()*/
        unimplemented!()
    }

    fn uuid(&self) -> String {
        self.uuid.clone()
    }
}

impl std::hash::Hash for Box<dyn NoteTrait> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid().hash(state)
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

