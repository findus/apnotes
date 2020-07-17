extern crate clap;
extern crate apple_notes_rs;
extern crate log;

use clap::{Arg, App, ArgMatches};

use apple_notes_rs::{create_new_note, apple_imap};
use apple_notes_rs::edit::*;
use apple_notes_rs::error::UpdateError;
use apple_notes_rs::sync::sync;
use std::path::{Path};
use apple_notes_rs::note::NotesMetadata;
use log::Level;

fn main() {
    simple_logger::init_with_level(Level::Info).unwrap();

    let app = App::new("NotesManager")
        .version("0.1")
        .author("Findus")
        .about("Interface for Apple Notes on Linux")
        .subcommand(App::new("edit")
            .about("Edits an existing note")
            .arg(Arg::with_name("path")
                .required(true)
                .takes_value(true)
                .help("Path to note that should be edited")
            )
        )
        .subcommand(App::new("sync")
            .about("Syncs local with remote notes and vice versa")
        )
        .subcommand(App::new("backup")
            .about("Duplicates current note tree on the imap server")
        )
        .subcommand(App::new("new")
            .about("Creates a new note")
            .arg(Arg::with_name("folder")
                .short("f")
                .long("folder")
                .help("Specifies the subfolder where the note should be put in. Uses default folder, if not used")
                .required(false)
                .default_value("Notes")
                .takes_value(true)
            )
            .arg(Arg::with_name("title")
                .required(true)
                .takes_value(true)
                .help("Title of the new note")
            )
        );

    let _res = match app.get_matches().subcommand() {
        ("new",  Some(sub_matches)) => new(sub_matches),
        ("sync", Some(_sub_matches)) => sync_notes(),
        ("edit", Some(sub_matches)) => edit_notes(sub_matches),
        (_, _) => unreachable!(),
    };

}

fn edit_notes(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("path").unwrap().to_string();

    let metadata_file_path =
        apple_notes_rs::util::get_hash_path(Path::new(&folder));

    let metadata_file = std::fs::File::open(&metadata_file_path)
        .expect(&format!("Could not open {}", &metadata_file_path.to_string_lossy()));

    let metadata: NotesMetadata = serde_json::from_reader(metadata_file).unwrap();

    apple_notes_rs::util::get_hash_path(Path::new(&folder));
    apple_notes_rs::edit::edit(&metadata, false).unwrap();
}

fn sync_notes() {
    let mut session = apple_imap::login();
    sync(&mut session);
}

fn new(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("folder").unwrap().to_string();
    let subject = sub_matches.value_of("title").unwrap().to_string();
    create_new_note(subject,folder)
        .map_err(|e| UpdateError::IoError(e.to_string()))
        .and_then(|metadata| edit(&metadata, true)).unwrap();
}