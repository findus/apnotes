extern crate clap;
extern crate apple_notes_rs_lib;
extern crate log;
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate colored;
extern crate itertools;

use clap::{Arg, App, ArgMatches, AppSettings};
use log::Level;
use apple_notes_rs_lib::error::{NoteError};
use apple_notes_rs_lib::create_new_note;
use self::diesel_migrations::*;
use apple_notes_rs_lib::sync::sync;
use apple_notes_rs_lib::db::{SqliteDBConnection, DatabaseService};
use colored::Colorize;
use itertools::*;
use apple_notes_rs_lib::edit::edit_note;
use apple_notes_rs_lib::note::{IdentifyableNote, MergeableNoteBody, LocalNote};
use apple_notes_rs_lib::error::NoteError::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

embed_migrations!("../migrations/");

fn main() {
    simple_logger::init_with_level(Level::Info).unwrap();

    let connection = SqliteDBConnection::new();

    // This will run the necessary migrations.
    embedded_migrations::run(connection.connection()).unwrap();

    let app = App::new("NotesManager")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.1")
        .author("Philipp Hentschel")
        .subcommand(App::new("list")
            .about("Lists all available notes")
            .arg(Arg::with_name("uuid")
                .short("u")
                .long("uuid")
                .help("Prints additional uuid")
                .required(false)
            )
        )
        .about("Interface for interacting with Apple Notes on Linux")
        .subcommand(App::new("edit")
            .about("Edits an existing note")
            .arg(Arg::with_name("path")
                .required(true)
                .takes_value(true)
                .help("Subject or UUID of the note that should be edited")
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

    let result = match app.get_matches().subcommand() {
        ("new",  Some(sub_matches)) => new(sub_matches),
        ("sync", Some(_sub_matches)) => ::apple_notes_rs_lib::sync::sync_notes(),
        ("list", Some(sub_matches)) => list_notes(sub_matches),
        ("edit", Some(sub_matches)) => edit_passed_note(sub_matches),
        (_, _) => unreachable!(),
    };

    if result.is_err() {
        eprint!("Error: {}", result.err().unwrap().to_string())
    }

}

fn edit_passed_note(sub_matches: &ArgMatches) -> Result<()> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    let db = apple_notes_rs_lib::db::SqliteDBConnection::new();
    ::apple_notes_rs_lib::find_note(&uuid_or_name, &db)
        .and_then(|note| apple_notes_rs_lib::edit::edit_note(&note, false).map_err(|e| e.into()))
        .and_then(|note| db.update(&note).map_err(|e| e.into()))
        .map_err(|e| NoteError::InsertionError(e.to_string()).into())
}

fn list_notes(sub_matches: &ArgMatches) -> Result<()>{
    let _show_uuid = sub_matches.is_present("uuid");
    let db_connection= ::apple_notes_rs_lib::db::SqliteDBConnection::new();
    db_connection
        .fetch_all_notes()
        .and_then(|notes| {
            let max_len = notes.iter()
                .map(|note| format!("{} {}", note.metadata.uuid, note.metadata.folder()).len())
                .max()
                .unwrap();

            notes.iter().
                sorted_by_key(|note| &note.metadata.subfolder)
                .for_each(|ee| {
                    let titles = ee.body.iter()
                        .map(|body| body.subject())
                        .join(",");

                    let formatted_uuid_folder = format!("{} {}", ee.metadata.uuid, ee.metadata.folder());

                    if ee.needs_local_merge() {
                        println!("{:<width$}  [{}] {}", formatted_uuid_folder, titles.red(), "<<Needs Merge>>".red(), width = max_len);
                    } else {
                        println!("{:<width$}  [{}]", formatted_uuid_folder, titles, width = max_len);
                    }
                });
            Ok(())
        })
        .map_err(|e| e.into())
}

fn new(sub_matches: &ArgMatches) -> Result<()> {
    let folder = sub_matches.value_of("folder").unwrap().to_string();
    let subject = sub_matches.value_of("title").unwrap().to_string();

    let db_connection = ::apple_notes_rs_lib::db::SqliteDBConnection::new();

    create_new_note(&db_connection,subject,folder)
        .and_then(|metadata| edit_note(&metadata, true))
        .and_then(|local_note| db_connection.update(&local_note)
            .map(|_| ())
            .map_err(|e| NoteError::InsertionError(e.to_string()))
        )
        .map_err(|e| e.into())
}