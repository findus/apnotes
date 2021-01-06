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
use apple_notes_rs_lib::edit::edit_note;
use self::diesel_migrations::*;
use apple_notes_rs_lib::sync::sync;
use apple_notes_rs_lib::db::{SqliteDBConnection, DatabaseService};
use colored::Colorize;
use itertools::*;

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
            //TODO handling of duplicate note names
            .about("Edits an existing note")
            .arg(Arg::with_name("name")
                .required(true)
                .takes_value(true)
                .help("Name of the note that you want to edit")
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
        ("list", Some(sub_matches)) => list_notes(sub_matches),
      //  ("edit", Some(sub_matches)) => edit_notes(sub_matches),
        (_, _) => unreachable!(),
    };

}

/*fn edit_notes(sub_matches: &ArgMatches) {
    let folder = sub_matches.value_of("name").unwrap().to_string();

    let metadata_file_path =
        apple_notes_rs_lib::util::get_hash_path(Path::new(&folder));

    let metadata_file = std::fs::File::open(metadata_file_path.as_path())
        .expect(&format!("Could not open {}", &metadata_file_path.to_string_lossy()));

    let metadata: NotesMetadata = serde_json::from_reader(metadata_file).unwrap();

    apple_notes_rs_lib::util::get_hash_path(Path::new(&folder));
    apple_notes_rs_lib::edit::edit(&metadata, false).unwrap();
}*/

fn sync_notes() {
    let mut imap_service = ::apple_notes_rs_lib::apple_imap::MailServiceImpl::new_with_login();
    let db_connection= ::apple_notes_rs_lib::db::SqliteDBConnection::new();
    sync(&mut imap_service, &db_connection);
}

fn list_notes(sub_matches: &ArgMatches) {
    let show_uuid = sub_matches.is_present("uuid");
    let db_connection= ::apple_notes_rs_lib::db::SqliteDBConnection::new();
    match db_connection.fetch_all_notes() {
        Ok(notes) => {
            let n_plus_1 =
                notes.iter().sorted_by_key(|i| &i.metadata.subfolder).skip(1);

            let n =
                notes.iter().sorted_by_key(|i| &i.metadata.subfolder);

            n_plus_1.enumerate().zip(n).for_each(|((idx, prev_note), this_note)| {
                if prev_note.metadata.subfolder != this_note.metadata.subfolder || idx == 0 {
                    println!("Folder: {}", prev_note.metadata.subfolder.white() );
                }
                if prev_note.body.len() > 1 {
                    if show_uuid {
                        print!("{}",this_note.metadata.uuid.as_str().bright_black());
                    }
                    print!("     ");
                    prev_note.body.iter().for_each(|body| {
                        print!("{}", format!("[{}], ", body.subject()));
                    });
                    print!("{}","Needs merge!".red());
                    println!();
                } else {
                    if show_uuid {
                        print!("{}",this_note.metadata.uuid.as_str().bright_black());
                    }
                    print!("     {} ", prev_note.body.first().unwrap().subject());
                    println!();
                }

            });
        },
        Err(e) => {
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