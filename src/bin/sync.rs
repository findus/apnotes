extern crate apple_notes_rs;
extern crate log;

use apple_notes_rs::*;
use apple_notes_rs::sync::sync;

fn main() {
    env_logger::init();

    let mut session = apple_imap::login();
    sync(&mut session);
}