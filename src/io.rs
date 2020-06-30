extern crate log;

use note::{NotesMetadata, HeaderParser};
use converter;
use std::fs::File;
use std::io::Write;
use self::log::info;
use note::{Note, NoteTrait};
use profile;

pub fn save_all_notes_to_file(notes: &Vec<Note>) {
    notes.iter().for_each(|note| {
      save_note_to_file(note)
    });
}

pub fn save_note_to_file(note: &Note) {
    let location = profile::get_notes_dir() + &note.folder + "/" + &note.mail_headers.subject_with_identifier();
    info!("Save to {}", location);

    let path = std::path::Path::new(&location);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let mut f = File::create(location).expect("Unable to create file");
    f.write_all(converter::convert2md(&note.body()).as_bytes()).expect("Unable to write file");

    let location = profile::get_notes_dir() +  &note.folder + "/." + &note.mail_headers.subject_with_identifier() + "_hash";
    info!("Save hash to {}", location);

    let path = std::path::Path::new(&location);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let f = File::create(&location).expect(format!("Unable to create hash file for {}", location).as_ref());

    serde_json::to_writer(f, &note.mail_headers);
}