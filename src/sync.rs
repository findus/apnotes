extern crate log;
extern crate walkdir;
extern crate glob;

use note::{NotesMetadata, LocalNote, HeaderParser};
use apple_imap::*;
use std::net::TcpStream;
use native_tls::TlsStream;
use imap::Session;
use self::log::{info, debug, error, warn};
use std::fs::File;
use self::walkdir::WalkDir;
use std::collections::HashSet;
use sync::UpdateAction::{DoNothing, UpdateLocally, UpdateRemotely, Merge, AddLocally, AddRemotely};
use apple_imap;
use io;
use profile;
use error::UpdateError::*;
use error::UpdateError;

#[derive(PartialEq, Clone,Copy)]
pub enum UpdateAction {
    DeleteRemote,
    DeleteLocally,
    UpdateRemotely,
    UpdateLocally,
    Merge,
    AddRemotely,
    AddLocally,
    DoNothing
}

pub fn sync(session: &mut Session<TlsStream<TcpStream>>) {
    let metadata = fetch_headers(session);
    let remote_metadata = metadata.iter().collect();

    let local_messages = get_local_messages();

    let local_metadata = local_messages
        .iter()
        .map(|note| &note.metadata)
        .collect();

    let mut add_delete_actions = get_added_deleted_notes(local_metadata, remote_metadata);
    info!("Need top add or delete {} Notes", &add_delete_actions.len());
    //TODO check if present remote notes were explicitely deleted locally

    let update_actions = get_update_actions(&metadata);
    info!("Need top update {} Notes", &update_actions.len());
    let mut d: Vec<(UpdateAction, &NotesMetadata)> = update_actions.iter().map(|(a,b)| (a.clone(),b)).collect();

    d.append(&mut add_delete_actions);
    let action_results = execute_actions(&d, session);
    action_results.iter().for_each(|result| {
        println!("{}", result.is_ok())
    })

}

fn get_update_actions(remote_notes: &Vec<NotesMetadata>) -> Vec<(UpdateAction, NotesMetadata)> {
    //TODO analyze what happens if title changes remotely, implement logic for local title change
    remote_notes.into_iter().map( |mail_headers| {

        let hash_location = profile::get_notes_dir() + &mail_headers.subfolder + "/." + &mail_headers.uid.to_string() + "_*";
        let hash_loc_path = glob::glob(&hash_location).expect("could not parse glob").next().unwrap().unwrap();

        if hash_loc_path.exists() {
            let f = File::open(hash_loc_path).unwrap();
            let local_metadata : NotesMetadata = serde_json::from_reader(f).unwrap();

            let local_uuid = local_metadata.message_id();
            let oldest_remote_uuid = &local_metadata.old_remote_id;

            let remote_uuid = mail_headers.message_id();

            if remote_uuid == local_uuid {
                debug!("Same: {}", mail_headers.subfolder.to_string() + "/" + &mail_headers.subject());
                return Some((DoNothing, mail_headers.clone()))
            } else if remote_uuid != local_uuid && oldest_remote_uuid.is_none() {
                info!("Changed Remotely: {}", mail_headers.subject());
                return Some((UpdateLocally, mail_headers.clone()))
            } else if oldest_remote_uuid.is_some() && oldest_remote_uuid.clone().unwrap() == remote_uuid {
                info!("Changed Locally: {}", &local_metadata.subject());
                return Some((UpdateRemotely, local_metadata))
            } else if oldest_remote_uuid.is_some() && remote_uuid != local_uuid {
                info!("Changed on both ends, needs merge: {}", &mail_headers.subject());
                return Some((Merge, mail_headers.clone()))
            }
        } else {
            warn!("Could not find metadata_file: {}", &hash_loc_path.to_string_lossy())
        }
        return None
    }).filter_map(|e| {
        if e.is_some() && e.as_ref().unwrap().0 != DoNothing {
            e
        } else {
            None
        }
    }).collect::<Vec<(UpdateAction, NotesMetadata)>>()
}

fn update_remotely(metadata: &NotesMetadata, session: &mut Session<TlsStream<TcpStream>>) -> Result<(), UpdateError> {
    match apple_imap::update_message(session, metadata) {
        Ok(new_uid) => {
            println!("New UID: {}", new_uid);
            let new_metadata = NotesMetadata {
                header: metadata.header.clone(),
                old_remote_id: None,
                subfolder: metadata.subfolder.clone(),
                locally_deleted: metadata.locally_deleted,
                uid: new_uid
            };

            io::save_metadata_to_file(&new_metadata)
                .map_err(|e| SyncError(e.line().to_string()))
        },
        Err(e) => {
            error!("Error while updating note {} {}", metadata.subject(), e.to_string());
            Err(SyncError(e.to_string()))
        }
    }
}

fn update_locally(metadata: &NotesMetadata, session: &mut Session<TlsStream<TcpStream>>) -> Result<(), UpdateError> {
    let note = apple_imap::fetch_single_note(session,metadata).unwrap();
    io::save_note_to_file(&note).map_err(|e| SyncError(e.to_string()))
}

fn execute_actions(actions: &Vec<(UpdateAction, &NotesMetadata)>, session:  &mut Session<TlsStream<TcpStream>>) -> Vec<Result<(), UpdateError>> {
     actions.iter().map(|(action, metadata)| {
        match action {
            AddRemotely => {
                create_mailbox(session, metadata).map_err(|e| SyncError(e.to_string()))
                    .and_then( |_| session.select(&metadata.subfolder).map_err(|e| SyncError(e.to_string()))
                    .and_then(|_| update_remotely(metadata, session)))
            },
            UpdateRemotely => {
                update_remotely(metadata, session)
            },
            UpdateAction::UpdateLocally | UpdateAction::AddLocally => {
                update_locally(metadata, session)
            }
            _ => {
                unimplemented!("Action is not implemented")
            }
        }
    }).collect()
}

fn get_local_messages() -> Vec<LocalNote> {

    WalkDir::new(profile::get_notes_dir())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.file_name().to_str().unwrap().to_string().starts_with("."))
        .map(| d| {
            LocalNote::new(d)
        }).collect()
}


pub fn get_added_deleted_notes<'a>(local_metadata: HashSet<&'a NotesMetadata>, remote_metadata: HashSet<&'a NotesMetadata>) -> Vec<(UpdateAction, &'a NotesMetadata)> {

    info!("Loading local messages");
    let _local_messages = get_local_messages();

    let local_size = local_metadata.len();
    info!("Found {} local notes", local_size);

    let remote_size = remote_metadata.len();
    info!("Found {} remote messages", remote_size);


    let mut only_local: Vec<(UpdateAction,&NotesMetadata)> = local_metadata
        .difference(&remote_metadata)
        .into_iter()
        .map(|e| (AddRemotely,e.clone()))
        .collect();

    let mut only_remote: Vec<(UpdateAction,&NotesMetadata)> = remote_metadata
        .difference(&local_metadata)
        .into_iter()
        .map(|e| (AddLocally,e.to_owned()))
        .collect();

    only_local.append(&mut only_remote);
    only_local

}