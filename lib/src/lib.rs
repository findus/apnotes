extern crate serde_json;
extern crate serde_derive;
extern crate walkdir;
extern crate native_tls;
extern crate imap;
extern crate serde;
extern crate uuid;
extern crate core;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate alloc;

pub mod apple_imap;
pub mod note;
pub mod converter;
pub mod profile;
pub mod sync;
pub mod io;
pub mod util;
pub mod error;
pub mod edit;
pub mod db;
pub mod model;
pub mod schema;


//use io::save_all_notes_to_file;



/*pub fn fetch_all() {
    let mut session = login();
    let notes = apple_imap::fetch_notes(&mut session);
    //save_all_notes_to_file(&notes);
}*/

/*pub fn create_new_note(with_subject: String, folder: String) -> Result<NotesMetadata> {
    let _headers = util::generate_mail_headers(with_subject);

    let profile = self::profile::load_profile();

    let note = NotesMetadata {
        old_remote_id: None,
        subfolder: format!("Notes.{}",folder),
        locally_deleted: false,
        new: true,
        date: Default::default(),
        uuid: "".to_string(),
        mime_version: "".to_string(),
    };

    let body = Body {
        message_id: format!("<{}@{}", util::generate_uuid(), profile.domain()),
        text: "".to_string(),
        uid: None,
        metadata_uuid: note.uuid.clone()
    };

    //TODO error handling
    let connection = io::establish_connection();
    io::insert_into_db(&connection, &note)
}*/