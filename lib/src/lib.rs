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
#[cfg(test)]
extern crate mockall;
extern crate colored;

use note::LocalNote;

use db::{DatabaseService, DBConnector};

#[macro_use]
mod macros;
pub mod apple_imap;
pub mod note;
pub mod converter;
pub mod profile;
pub mod sync;
pub mod io;
#[macro_use]
mod util;
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

pub fn create_new_note<C>(db_connection: &dyn DatabaseService<C>, with_subject: String, folder: String)
    -> Result<LocalNote,::error::NoteError> where C: 'static + DBConnector
{

    let note = note!(
        builder::NotesMetadataBuilder::new().with_folder(folder).is_new(true).build(),
        builder::BodyMetadataBuilder::new().with_text(&with_subject).build()
    );

    db_connection.insert_into_db(&note)
        .and_then(|_| Ok(note))
        .map_err(|e| ::error::NoteError::InsertionError(e.to_string()))
}