extern crate apple_notes_rs;
extern crate log;

use apple_notes_rs::*;
use apple_notes_rs::fetcher::save_all_notes_to_file;

fn main() {
    env_logger::init();

    let mut session = fetcher::login();
    save_all_notes_to_file(&mut session);
}