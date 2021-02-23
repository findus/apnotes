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
extern crate log;

#[macro_use]
mod macros;
mod apple_imap;
mod converter;
mod profile;
mod sync;
#[macro_use]
mod util;
pub mod error;
mod edit;
pub mod db;
mod model;
mod schema;
mod builder;
pub mod notes;
mod merge;

use db::{DatabaseService, DBConnector, SqliteDBConnection};
use error::NoteError::NoteNotFound;
use util::is_uuid;
use notes::localnote::LocalNote;
use error::{UpdateError, NoteError};
use sync::UpdateAction;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Syncs with the imap server
pub fn sync_notes() -> Result<Vec<Result<(String,String)>>> {
    sync::sync_notes()
}

/// Opens a text editor with the content of the specified note
pub fn edit_note(local_note: &LocalNote, new: bool ) -> std::result::Result<LocalNote,NoteError>{
    edit::edit_note(local_note, new)
}

/// Creates a new note, with the specified name inside the specified folder
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
                    error!("Note does not exist");
                    Err(NoteNotFound.into())
                },
                Err(e) => {
                    error!("Error occured: {}", e.to_string());
                    Err(e.into())
                }
            }
        }
        false => {
            match db.fetch_single_note_with_name(&uuid_or_name) {
                Ok(Some(note)) => Ok(note),
                Ok(None) => {
                    error!("Note does not exist");
                    Err(NoteNotFound.into())
                },
                Err(e) => {
                    error!("Error occured: {}", e.to_string());
                    Err(e.into())
                }
            }
        }
    }
}

/// Merges notes that have > 1 bodies (right now only 2 bodies supported)
/// After merging it the default text editor gets opened so that the user
/// can resolve all conflicts, after saving the note is marked as merged
pub fn merge(uuid_or_name: &String, db: &SqliteDBConnection) -> Result<()> {
    find_note(&uuid_or_name, &db)
        .and_then(|note| {

            //TODO currently only supports merging for 2 notes
            if note.needs_merge() == false || note.body.len() > 2 {
                return Err(UpdateError::SyncError("Note not mergeable, right now only notes with 2 bodies are mergeable".to_string()).into());
            }

            let diff = merge::merge_two(&note.body[0].text.as_ref().unwrap(), &note.body[1].text.as_ref().unwrap());
            let note = note![
                note.metadata.clone(),
                builder::BodyMetadataBuilder::new().with_text(&diff).with_message_id(&format!("{},{}",&note.body[0].message_id, &note.body[1].message_id)).build()
            ];
            Ok(note)
        })
        .and_then(|note| edit::edit_note(&note, false).map_err(|e| e.into()))
        .and_then(|note| db.update(&note).map_err(|e| e.into()))
}