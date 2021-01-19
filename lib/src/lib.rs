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
extern crate regex;
extern crate diff;

#[macro_use]
pub mod macros;
pub mod apple_imap;
pub mod converter;
pub mod profile;
pub mod sync;
#[macro_use]
mod util;
pub mod error;
pub mod edit;
pub mod db;
pub mod model;
pub mod schema;
pub mod builder;
pub mod notes;
pub mod merge;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use db::{DatabaseService, DBConnector, SqliteDBConnection};
use error::NoteError::NoteNotFound;
use util::is_uuid;
use notes::localnote::LocalNote;

pub fn create_new_note<C>(db_connection: &dyn DatabaseService<C>, with_subject: String, folder: String)
    -> std::result::Result<LocalNote,::error::NoteError> where C: 'static + DBConnector
{

    let note = note!(
        builder::NotesMetadataBuilder::new().with_folder(folder).is_new(true).build(),
        builder::BodyMetadataBuilder::new().with_text(&with_subject).build()
    );

    db_connection.insert_into_db(&note)
        .and_then(|_| Ok(note))
        .map_err(|e| ::error::NoteError::InsertionError(e.to_string()))
}

/// Queries the database and tries to find a note with the provided search string
/// Auto-Detects if the user provides the title or a uuid.
///
/// If multiple notes with the same title exist it returns the first one
/// avaiable
pub fn find_note(uuid_or_name: &String, db: &SqliteDBConnection) -> Result<LocalNote> {
    match is_uuid(&uuid_or_name) {
        true => {
            match db.fetch_single_note(&uuid_or_name) {
                Ok(Some(note)) => Ok(note),
                Ok(None) => {
                    eprintln!("Note does not exist");
                    Err(NoteNotFound.into())
                },
                Err(e) => {
                    eprintln!("Error occured: {}", e.to_string());
                    Err(e.into())
                }
            }
        }
        false => {
            match db.fetch_single_note_with_name(&uuid_or_name) {
                Ok(Some(note)) => Ok(note),
                Ok(None) => {
                    eprintln!("Note does not exist");
                    Err(NoteNotFound.into())
                },
                Err(e) => {
                    eprint!("Error occured: {}", e.to_string());
                    Err(e.into())
                }
            }
        }
    }
}

pub fn merge(uuid_or_name: &String, db: &SqliteDBConnection) -> Result<()> {
    find_note(&uuid_or_name, &db)
        .and_then(|note| {
            let diff = merge::merge_two(&note.body[0].text.as_ref().unwrap(), &note.body[1].text.as_ref().unwrap());
            let note = note![
                note.metadata.clone(),
                builder::BodyMetadataBuilder::new().with_text(&diff).build()
            ];
            Ok(note)
        })
        .and_then(|note| edit::edit_note(&note, false).map_err(|e| e.into()))
        .and_then(|note| db.update(&note).map_err(|e| e.into()))
}