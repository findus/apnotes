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

use crate::apple_imap::*;

pub fn fetch_all() {
    let mut session = login();
    save_all_notes_to_file(&mut session);
}