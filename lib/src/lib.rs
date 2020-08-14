extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;
extern crate native_tls;
extern crate imap;
extern crate serde;
extern crate uuid;
extern crate core;
extern crate chrono;

pub mod apple_imap;
pub mod note;
pub mod converter;
pub mod profile;
pub mod sync;
pub mod io;
pub mod util;
pub mod error;
pub mod edit;

use crate::apple_imap::*;
use io::save_all_notes_to_file;
use note::{NotesMetadata, LocalNote};
use std::io::Result;

pub fn fetch_all() {
    let mut session = login();
    let notes = apple_imap::fetch_notes(&mut session);
    save_all_notes_to_file(&notes);
}

pub fn create_new_note(with_subject: String, folder: String) -> Result<NotesMetadata>  {
    let headers = util::generate_mail_headers(with_subject);

    let metadata = NotesMetadata {
        header: headers,
        old_remote_id: None,
        subfolder: format!("Notes.{}",folder),
        locally_deleted: false,
        uid: None,
        new: true
    };

    let note = LocalNote {
        path: util::get_notes_file_path_from_metadata(&metadata),
        metadata: metadata.clone()
    };

    //TODO error handling
    io::save_metadata_to_file(&metadata)
        .and_then(|_| io::save_note_to_file(&note))
        .and_then(|_| io::save_text_to_file(&metadata))
        .map(|_| metadata)
        .map_err(|e| std::io::Error::from(e))

}