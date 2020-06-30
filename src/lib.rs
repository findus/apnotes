extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;
extern crate native_tls;
extern crate imap;

pub mod apple_imap;
pub mod note;
pub mod converter;
pub mod profile;
pub mod sync;
pub mod io;

use crate::apple_imap::*;
use io::save_all_notes_to_file;

pub fn fetch_all() {
    let mut session = login();
    let notes = apple_imap::fetch_notes(&mut session);
    save_all_notes_to_file(&notes);
}