extern crate clap;
extern crate apple_notes_rs_lib;
extern crate log;
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use clap::{Arg, App, ArgMatches, AppSettings};
use log::Level;
use apple_notes_rs_lib::error::{UpdateError, NoteError};
use apple_notes_rs_lib::create_new_note;
use apple_notes_rs_lib::edit::edit_note;
use self::diesel_migrations::*;
use apple_notes_rs_lib::db::establish_connection;
use apple_notes_rs_lib::note::LocalNote;

//use apple_notes_rs_lib::{apple_imap};
//use apple_notes_rs_lib::sync::sync;
//use apple_notes_rs_lib::error::UpdateError;
//use apple_notes_rs_lib::edit::edit;
//use apple_notes_rs_lib::model::NotesMetadata;

embed_migrations!("../lib/migrations/");

fn main() {
    simple_logger::init_with_level(Level::Info).unwrap();

    let connection = establish_connection();

    // This will run the necessary migrations.
    embedded_migrations::run(&connection);

    let app = App::new("NotesManager")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.1")
        .author("Philipp Hentschel")
        .about("Interface for interacting with Apple Notes on Linux")
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
      //  ("edit", Some(sub_matches)) => edit_notes(sub_matches),
        (_, _) => unreachable!(),
    };

}

/*fn edit_notes(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("path").unwrap().to_string();

    let metadata_file_path =
        apple_notes_rs_lib::util::get_hash_path(Path::new(&folder));

    let metadata_file = std::fs::File::open(metadata_file_path.as_path())
        .expect(&format!("Could not open {}", &metadata_file_path.to_string_lossy()));

    let metadata: NotesMetadata = serde_json::from_reader(metadata_file).unwrap();

    apple_notes_rs_lib::util::get_hash_path(Path::new(&folder));
    apple_notes_rs_lib::edit::edit(&metadata, false).unwrap();
}*/

fn sync_notes() {
    //let mut session = apple_imap::login();
    //sync(&mut session);
}

fn new(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("folder").unwrap().to_string();
    let subject = sub_matches.value_of("title").unwrap().to_string();
    match create_new_note(subject,folder)
        .and_then(|metadata| edit_note(&metadata, true)) {
        Err(NoteError::ContentNotChanged) => {
            println!("Content unchanged wont flag note for update")
        },
        Err(e) => {
            println!("{}",format!("An error occured: {}",e));
        },
        _ => {
            println!("Note saved and ready for syncing");
        }
    };
}