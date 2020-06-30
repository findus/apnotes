extern crate apple_notes_rs;
extern crate log;

use apple_notes_rs::*;
use io::save_all_notes_to_file;

fn main() {
    env_logger::init();

    let mut session = apple_imap::login();
    let notes = apple_imap::fetch_notes(&mut session);
    save_all_notes_to_file(&notes);
}