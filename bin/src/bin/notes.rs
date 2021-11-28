extern crate clap;
extern crate apnotes_lib;
#[macro_use]
extern crate log;
extern crate diesel;
extern crate colored;
extern crate itertools;
extern crate flexi_logger;
extern crate apnotes_bin;
extern crate serde_json;

use clap::{ArgMatches};
use colored::Colorize;
use itertools::*;
use apnotes_lib::AppleNotes;
use apnotes_lib::notes::traits::identifyable_note::IdentifiableNote;
use flexi_logger::{Logger, Record, DeferredNow};
use apnotes_bin::app::app::gen_app;
use apnotes_lib::error::Result;

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

pub fn main() {

    Logger::with_env_or_str("info").format(default_format).start().unwrap();

    let app = gen_app();

    let db_connection= ::apnotes_lib::db::SqliteDBConnection::new();

    let matches = app.get_matches();

    match ::apnotes_lib::get_user_profile() {
        Ok(profile) => {
            let apple_notes = ::apnotes_lib::AppleNotes::new(
                profile,
                Box::new(db_connection)
            );

            let result = match matches.subcommand() {
                Some(("new",  sub_matches)) => new(sub_matches,&apple_notes),
                Some(("sync", sub_matches)) => sync_notes(sub_matches, &apple_notes).map(|_| ()),
                Some(("list", sub_matches)) => list_notes(sub_matches,&apple_notes),
                Some(("edit", sub_matches)) => edit_passed_note(sub_matches,&apple_notes),
                Some(("merge", sub_matches)) => merge_note(sub_matches,&apple_notes),
                Some(("delete", sub_matches)) => delete_note(sub_matches,&apple_notes),
                Some(("undelete", sub_matches)) => undelete_note(sub_matches,&apple_notes),
                Some(("print", sub_matches)) => print_note(sub_matches, &apple_notes),
                _ => unreachable!(),
            };

            match result {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {}\n{} - ({})", e.human_readable_error_message(), e.to_string(), e.error_code().to_string());
                    std::process::exit(e.error_code());
                },
            }
        }
        Err(e) => {
            error!("Could not load profile: {}", e.to_string());
        }
    }



}

fn print_note(sub_matches: &ArgMatches, app: &AppleNotes) -> Result<()> {
    let uuid_or_name = sub_matches.value_of("path").unwrap().to_string();
    app.print(&uuid_or_name)
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
    let print_names_only = sub_matches.is_present("names");
    let show_only_deleted = sub_matches.is_present("deleted");

    app.get_notes()
        .and_then(|notes| {

            let notes = if show_only_deleted {
                notes.into_iter().filter(|note| note.metadata.locally_deleted).collect()
            } else {
                notes
            };

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

                    let formatted_string = if print_names_only {
                        let subject = ee.body.iter().last().unwrap().subject();
                        // let escaped_subject = escape(subject.into()).to_string();
                        subject
                    }  else if ee.needs_merge() {
                        format!("{:<width$}  [{}] {}", formatted_uuid_folder, titles.red(), "<<Needs Merge>>".red(), width = max_len)
                    } else {
                        format!("{:<width$}  [{}]", formatted_uuid_folder, titles, width = max_len)
                    };

                    let formatted_string = if ee.metadata.locally_deleted == true && print_names_only == false {
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

fn sync_notes(sub_matches: &ArgMatches, app:&AppleNotes) -> Result<()> {
    let is_dry_run = sub_matches.is_present("dry-run");
    app.sync_notes(is_dry_run).map(|_| ())
}