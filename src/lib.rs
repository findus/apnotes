#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

pub mod apple_imap;
pub mod note;
pub mod converter;
pub mod profile;

use crate::apple_imap::*;

pub fn fetch_all() {
    let mut session = login();
    save_all_notes_to_file(&mut session);
}