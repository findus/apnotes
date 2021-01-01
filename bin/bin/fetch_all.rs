extern crate apple_notes_rs_lib;
extern crate log;


//use apple_notes_rs_lib::apple_imap;
//use apple_notes_rs_lib::io::save_all_notes_to_file;

fn main() {
    simple_logger::init().unwrap();
    /*let mut session = apple_imap::login();
    let notes = apple_imap::fetch_notes(&mut session);
    save_all_notes_to_file(&notes);*/
}