extern crate clap;
extern crate apple_notes_rs;

use clap::{Arg, App, ArgMatches};
use apple_notes_rs::io::save_note_to_file;
use apple_notes_rs::create_new_note;
use apple_notes_rs::edit::*;
use apple_notes_rs::error::UpdateError;

fn main() {
    simple_logger::init().unwrap();

    let app = App::new("My Super Program")
        .version("0.1")
        .author("Findus")
        .about("Interface for Apple Notes on Linux")
        .subcommand(App::new("edit")
            .about("Edits an existing note")
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

    let res = match app.get_matches().subcommand() {
        ("new", Some(sub_matches)) => new(sub_matches),
        (_, _) => unreachable!(),
    };

}

fn new(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("folder").unwrap().to_string();
    let subject = sub_matches.value_of("title").unwrap().to_string();
    create_new_note(subject,folder)
        .map_err(|e| UpdateError::IoError(e.to_string()))
        .and_then(|metadata| edit(&metadata)).unwrap();
}