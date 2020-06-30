extern crate log;

use note::NotesMetadata;
use fasthash::metro;
use converter;
use std::fs::File;
use std::io::Write;
use self::log::info;
use note::{Note, NoteTrait};

pub fn save_all_notes_to_file(notes: &Vec<Note>) {
    notes.iter().for_each(|note| {
        let location = "/home/findus/.notes/".to_string() + &note.folder + "/" + &note.subject();
        info!("Save to {}", location);

        let path = std::path::Path::new(&location);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let mut f = File::create(location).expect("Unable to create file");
        f.write_all(converter::convert2md(&note.body()).as_bytes()).expect("Unable to write file");


        let location = "/home/findus/.debug_html/".to_string() +  &note.folder + "/" + &note.subject();
        info!("Save to {}", location);

        let path = std::path::Path::new(&location);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let mut f = File::create(location).expect("Unable to create file");
        f.write_all(&note.body().as_bytes()).expect("Unable to write file");


        let location = "/home/findus/.notes/".to_string() +  &note.folder + "/." + &note.subject() + "_hash";
        info!("Save hash to {}", location);

        let path = std::path::Path::new(&location);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();


        let hash = metro::hash64(&note.body().as_bytes());

        let f = File::create(&location).expect(format!("Unable to create hash file for {}", location).as_ref());

        let note = note.mail_headers.clone();

        let dd = NotesMetadata {
            header: note,
            hash
        };


        serde_json::to_writer(f, &dd);
    });
}