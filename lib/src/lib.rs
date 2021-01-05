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
extern crate mailparse;

use note::LocalNote;
use diesel::SqliteConnection;

#[macro_use]
pub mod macros;
pub mod apple_imap;
pub mod note;
pub mod converter;
pub mod profile;
pub mod sync;
pub mod io;
#[macro_use]
pub mod util;
pub mod error;
pub mod edit;
pub mod db;
pub mod model;
pub mod schema;
pub mod builder;


//use io::save_all_notes_to_file;
/*pub fn fetch_all() {
    let mut session = login();
    let notes = apple_imap::fetch_notes(&mut session);
    //save_all_notes_to_file(&notes);
}*/

pub fn create_new_note(db_connection: &SqliteConnection, with_subject: String, folder: String) -> Result<LocalNote,::error::NoteError> {

    let note = note!(
        builder::NotesMetadataBuilder::new().with_folder(folder).is_new(true).build(),
        builder::BodyMetadataBuilder::new().with_text(&with_subject).build()
    );

    db::insert_into_db(&db_connection,&note)
        .and_then(|_| Ok(note))
        .map_err(|e| ::error::NoteError::InsertionError(e.to_string()))
}