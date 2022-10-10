use crate::notes::traits::header_parser::HeaderParser;

pub type NoteHeaders = Vec<(String, String)>;

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