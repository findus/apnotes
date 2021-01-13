extern crate clap;
extern crate apple_notes_rs_lib;
extern crate log;
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate colored;
extern crate itertools;
extern crate regex;

use clap::{Arg, App, ArgMatches, AppSettings};
use log::Level;
use apple_notes_rs_lib::error::{NoteError};
use apple_notes_rs_lib::create_new_note;
use self::diesel_migrations::*;
use apple_notes_rs_lib::sync::sync;
use apple_notes_rs_lib::db::{SqliteDBConnection, DatabaseService};
use colored::Colorize;
use itertools::*;
use regex::Regex;
use apple_notes_rs_lib::edit::edit_note;
use apple_notes_rs_lib::note::{LocalNote, IdentifyableNote, MergeableNoteBody};

//use apple_notes_rs_lib::{apple_imap};
//use apple_notes_rs_lib::sync::sync;
//use apple_notes_rs_lib::error::UpdateError;
//use apple_notes_rs_lib::edit::edit;
//use apple_notes_rs_lib::model::NotesMetadata;

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

    let _res = match app.get_matches().subcommand() {
        ("new",  Some(sub_matches)) => new(sub_matches),
        ("sync", Some(_sub_matches)) => sync_notes().unwrap(),
        ("list", Some(sub_matches)) => list_notes(sub_matches),
        ("edit", Some(sub_matches)) => edit_passed_note(sub_matches).unwrap(),
        (_, _) => unreachable!(),
    };

}

fn edit_passed_note(sub_matches: &ArgMatches) -> Result<(), NoteError> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    let db = apple_notes_rs_lib::db::SqliteDBConnection::new();
    let note = match is_uuid(&uuid_or_name) {
        true => {
            match db.fetch_single_note(&uuid_or_name) {
                Ok(Some(note)) => note,
                Ok(None) => panic!("Note does not exist"),
                Err(e) => panic!(e.to_string())
            }
        }
        false => {
            match db.fetch_single_note_with_name(&uuid_or_name){
                Ok(Some(note)) => note,
                Ok(None) => panic!("Note does not exist"),
                Err(e) => panic!(e.to_string())
            }
        }
    };
    apple_notes_rs_lib::edit::edit_note(&note, false)
        .and_then(|note| db.update(&note)
            .map_err(|e| NoteError::InsertionError(e.to_string()))
        )
}

fn is_uuid(string: &str) -> bool {
    let uuid_regex: Regex =
        Regex::new(r"\b[0-9A-F]{8}\b-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-\b[0-9A-F]{12}\b").unwrap();
    uuid_regex.is_match(string)
}

fn sync_notes() -> Result<(),::apple_notes_rs_lib::error::UpdateError> {
    let mut imap_service = ::apple_notes_rs_lib::apple_imap::MailServiceImpl::new_with_login();
    let db_connection= ::apple_notes_rs_lib::db::SqliteDBConnection::new();
    sync(&mut imap_service, &db_connection)
}

fn list_notes(sub_matches: &ArgMatches) {
    let show_uuid = sub_matches.is_present("uuid");
    let db_connection= ::apple_notes_rs_lib::db::SqliteDBConnection::new();
    match db_connection.fetch_all_notes() {
        Ok(notes) => {
            let max_len = notes.iter()
                .map(|note| format!("{} {}", note.metadata.uuid, note.metadata.folder()).len())
                .max()
                .unwrap();

            notes.iter().
                sorted_by_key(|note| &note.metadata.subfolder)
                .foreach(|ee| {
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
        },
        Err(_e) => {
            println!("Something went wrong, check loggos")
        }
    };
}

fn new(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("folder").unwrap().to_string();
    let subject = sub_matches.value_of("title").unwrap().to_string();

    let db_connection = ::apple_notes_rs_lib::db::SqliteDBConnection::new();

    match create_new_note(&db_connection,subject,folder)
        .and_then(|metadata| edit_note(&metadata, true))
        .and_then(|local_note| db_connection.update(&local_note)
            .map(|_| local_note)
            .map_err(|e| NoteError::InsertionError(e.to_string()))
        )
    {
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