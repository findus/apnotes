extern crate clap;
extern crate apple_notes_manager;
#[macro_use]
extern crate log;
extern crate diesel;
extern crate colored;
extern crate itertools;
extern crate flexi_logger;

use clap::{Arg, App, ArgMatches, AppSettings};

use colored::Colorize;
use itertools::*;
use log::Level;
use apple_notes_manager::AppleNotes;
use apple_notes_manager::notes::traits::identifyable_note::IdentifyableNote;
use flexi_logger::{Logger, Record, DeferredNow};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//Minimal println like formatting for flexi_logger
pub fn default_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> core::result::Result<(), std::io::Error> {
    write!(
        w,
        "{}",
        record.args()
    )
}

fn main() {

    Logger::with_env_or_str("info").format(default_format).start().unwrap();

    let app = App::new("NotesManager")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.2.0")
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
        .subcommand(App::new("delete")
            .about("Flags a note as deleted")
            .arg(Arg::with_name("path")
                .required(true)
                .takes_value(true)
                .help("Subject or UUID of the note that should be deleted")
            )
        )
        .subcommand(App::new("undelete")
            .about("Removes deletion flag")
            .arg(Arg::with_name("path")
                .required(true)
                .takes_value(true)
                .help("Subject or UUID of the note")
            )
        )
        .subcommand(App::new("merge")
            .about("Merges unmerged Note")
            .arg(Arg::with_name("path")
                .required(true)
                .takes_value(true)
                .help("Subject or UUID of the note that should be merged")
            )
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
                .takes_value(true)
            )
            .arg(Arg::with_name("title")
                .required(true)
                .takes_value(true)
                .help("Title of the new note")
            )
        );

    let db_connection= ::apple_notes_manager::db::SqliteDBConnection::new();

    let matches = app.get_matches();

    match ::apple_notes_manager::get_user_profile() {
        Ok(profile) => {
            let apple_notes = ::apple_notes_manager::AppleNotes::new(
                profile,
                Box::new(db_connection)
            );

            let result = match matches.subcommand() {
                ("new",  Some(sub_matches)) => new(sub_matches,&apple_notes),
                ("sync", Some(_sub_matches)) => apple_notes.sync_notes().map(|_| ()),
                ("list", Some(sub_matches)) => list_notes(sub_matches,&apple_notes),
                ("edit", Some(sub_matches)) => edit_passed_note(sub_matches,&apple_notes),
                ("merge", Some(sub_matches)) => merge_note(sub_matches,&apple_notes),
                ("delete", Some(sub_matches)) => delete_note(sub_matches,&apple_notes),
                ("undelete", Some(sub_matches)) => undelete_note(sub_matches,&apple_notes),
                (_, _) => unreachable!(),
            };

            match result {
                Ok(_) => {info!("Done")}
                Err(e) => {
                    error!("Error: {}", e.to_string());
                    std::process::exit(-1);
                },
            }
        }
        Err(e) => {
            error!("Could not load profile: {}", e.to_string());
        }
    }



}

fn undelete_note(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    app.undelete_note(&uuid_or_name)
}

fn delete_note(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    app.delete_note(&uuid_or_name)
}

fn merge_note(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    app.merge(&uuid_or_name)
}

fn edit_passed_note(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    app.find_note(&uuid_or_name)
        .and_then(|note| app.edit_note(&note, false).map_err(|e| e.into()))
        .and_then(|note| app.update_note(&note).map_err(|e| e.into()))
}

fn list_notes(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()>{
    let _show_uuid = sub_matches.is_present("uuid");

    app.get_notes()
        .and_then(|notes| {
            let max_len = notes.iter()
                .map(|note| format!("{} {}", note.metadata.uuid, note.metadata.folder()).len())
                .max()
                .unwrap_or(0);

            notes.iter().
                sorted_by_key(|note| format!("{}_{}",&note.metadata.subfolder, &note.body[0].subject()))
                .for_each(|ee| {
                    let titles = ee.body.iter()
                        .map(|body| body.subject())
                        .join(",");

                    let formatted_uuid_folder = format!("{} {}", ee.metadata.uuid, ee.metadata.folder());

                    let formatted_string = if ee.needs_merge() {
                        format!("{:<width$}  [{}] {}", formatted_uuid_folder, titles.red(), "<<Needs Merge>>".red(), width = max_len)
                    } else {
                        format!("{:<width$}  [{}]", formatted_uuid_folder, titles, width = max_len)
                    };

                    let formatted_string = if ee.metadata.locally_deleted == true {
                        format!("{} <<flagged for deletion>>", formatted_string).red().to_string()
                    } else {
                        formatted_string
                    };

                    info!("{}", formatted_string);

                });
            Ok(())
        })
        .map_err(|e| e.into())
}

fn new(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()> {
    let folder = sub_matches.value_of("folder").unwrap_or("").to_string();
    let subject = sub_matches.value_of("title").unwrap().to_string();

    app.create_new_note(&subject,&folder)
        .and_then(|metadata| app.edit_note(&metadata, true))
        .and_then(|local_note| app.update_note(&local_note))
        .map_err(|e| e.into())
}