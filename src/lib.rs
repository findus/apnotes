pub mod fetcher;
pub mod note;
pub mod converter;

use crate::fetcher::*;

pub fn fetch_all() {
    let mut session = login();
    save_all_notes_to_file(&mut session);
}