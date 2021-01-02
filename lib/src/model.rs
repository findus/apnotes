use note::{NoteHeaders, HeaderParser, NoteTrait};
use ::{util, profile};
use schema::metadata;
use schema::body;

#[derive(Identifiable,Clone,Queryable,Insertable,Debug,Eq)]
#[table_name="metadata"]
#[primary_key(uuid)]
pub struct NotesMetadata {
    /// Stores old message-id after editing
    /// the note. If the notes are getting synced
    /// this is neede to check if the remote note
    /// also changed, if this is the case
    pub old_remote_id: Option<String>,
    /// Stores the subfolder name of the folder in which
    /// the note is saved
    pub subfolder: String,
    pub locally_deleted: bool,
    pub locally_edited: bool,
    /// Indicator for newly created notes, so that they
    /// dont get deleted while syncing
    pub new: bool,
    pub date: String, //TODO type
    /// UUID for the message. This uuid never changes after
    /// creating a note.
    ///
    /// However multiple notes with the name can exist remotely
    /// if notes are getting edited simultaneously on multiple
    /// devices, the notes app recognizes this and duplicates
    /// the note the first with the content that was edited on
    /// device1, and the second with the content that was
    /// edited on device2.
    pub uuid: String,
    pub mime_version: String,
}

impl NotesMetadata {
    pub fn new(header: &NoteHeaders, subfolder: String) -> Self {
        NotesMetadata {
            old_remote_id: None,
            subfolder,
            locally_deleted: false,
            locally_edited: false,
            new: false,
            date: header.date(),
            uuid: header.uuid(),
            mime_version: header.mime_version(),
        }
    }
}

#[derive(Identifiable,Clone,Queryable,Insertable,Associations,Debug,Eq)]
#[table_name="body"]
#[belongs_to(NotesMetadata, foreign_key="metadata_uuid")]
#[primary_key(message_id)]
pub struct Body {
    /// Identifier for a note in a certain state. This
    /// ID changes every time the note gets edited.
    ///
    /// If you sync the notes and the remote message-id
    /// changed it is likely that the note got edited
    /// on another device.
    pub message_id: String,
    pub text: Option<String>,
    /// The IMAP UID identifier
    pub uid: Option<i64>,
    /// Foreign key to a Metadata Object, every Metadata
    /// Object can have n Bodies
    pub metadata_uuid: String
}

impl Body {
    pub fn new(uid: Option<i64>, metadata_reference: String) -> Body {
        let profile = profile::load_profile();
        Body {
            message_id: format!("<{}@{}", util::generate_uuid(), profile.domain()),
            text: None,
            uid,
            metadata_uuid: metadata_reference
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