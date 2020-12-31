
use note::{NoteHeader, HeaderParser};
use ::{util, profile};
use profile::Profile;

use schema::metadata;
use schema::body;

#[derive(Identifiable,Clone,Queryable,Insertable)]
#[table_name="metadata"]
#[primary_key(uuid)]
pub struct NotesMetadata {
    pub old_remote_id: Option<String>,
    pub subfolder: String,
    pub locally_deleted: bool,
    pub new: bool,
    pub date: String, //TODO type
    pub uuid: String,
    pub mime_version: String,
}

impl NotesMetadata {
    pub fn new(header: NoteHeader, subfolder: String, uid: u32, body: Option<Vec<Body>>) -> Self {
        NotesMetadata {
            old_remote_id: None,
            subfolder,
            locally_deleted: false,
            new: false,
            date: header.date(),
            uuid: header.identifier(),
            mime_version: header.mime_version(),
        }
    }

}

#[derive(Identifiable,Clone,Queryable,Insertable,Associations)]
#[table_name="body"]
#[belongs_to(NotesMetadata, foreign_key="metadata_uuid")]
#[primary_key(message_id)]
pub struct Body {
    pub message_id: String,
    pub text: String,
    pub uid: Option<i64>,
    pub metadata_uuid: String
}

impl Body {
    pub fn new(uid: i64) -> Body {
        let profile = profile::load_profile();
        Body {
            message_id: format!("<{}@{}", util::generate_uuid(), profile.domain()),
            text: "".to_string(),
            uid: Some(uid),
            //TODO set
            metadata_uuid: "".to_string()
        }
    }

    pub fn subject(&self) -> String {
        //return self.text.split("\\n").into_iter().map(|e| e.into_string()).collect::<Vec<String>>().first().unwrap().clone()
        return "todo".to_string()
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
        let escaped_string = format!("{}", self.subject())
            .replace("/", "_").replace(" ", "_");
        // .replace(|c: char| !c.is_ascii(), "");
        regex.replace_all(&escaped_string, "").into_owned()
    }
}