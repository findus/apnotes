
use note::{NoteHeader, HeaderParser};

#[derive(Clone,Queryable,Deserialize,Serialize)]
pub struct NotesMetadata {

    pub old_remote_id: Option<String>,
    pub subfolder: String,
    pub locally_deleted: bool,

    //IMAP UID
    pub uid: Option<i64>,
    pub new: bool,

    pub date: String, //TODO type
    pub uuid: String,
    pub message_id: String,
    pub mime_version: String,
    pub subject: String
}

impl NotesMetadata {
    pub fn new(header: NoteHeader, subfolder: String, uid: u32) -> Self {
        NotesMetadata {
            old_remote_id: None,
            subfolder,
            locally_deleted: false,
            uid: Some(uid as i64),
            new: false,
            date: header.date(),
            uuid: header.identifier(),
            mime_version: header.mime_version(),
            subject: header.subject(),
            message_id: header.message_id()
        }
    }

    pub fn subject_with_identifier(&self) -> String {
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
    pub fn subject_escaped(&self) -> String {
        let regex = regex::Regex::new(r#"[.<>:\\"/\|?*]"#).unwrap();
        let escaped_string = format!("{}", self.subject)
            .replace("/", "_").replace(" ", "_");
        // .replace(|c: char| !c.is_ascii(), "");
        regex.replace_all(&escaped_string, "").into_owned()
    }

    pub fn subject(&self) -> String {
        self.subject.clone()
    }
}

#[derive(Queryable)]
pub struct Body {
    pub message_id: String,
    pub body: String,
    pub meta_data: NotesMetadata
}