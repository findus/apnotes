extern crate log;
extern crate walkdir;


use note::{NotesMetadata, Note, LocalNote, NoteTrait, HeaderParser};
use apple_imap::*;
use std::net::TcpStream;
use native_tls::TlsStream;
use imap::Session;
use self::log::{info, debug, warn};
use std::fs::File;
use self::walkdir::WalkDir;
use std::collections::HashSet;
use sync::UpdateAction::{DoNothing, UpdateLocally, UpdateRemotely, Merge, DeleteRemote, DeleteLocally, AddLocally, AddRemotely};
use apple_imap;
use io;
use std::iter::FromIterator;
use converter;
use profile;

pub struct RemoteDifference {
    only_remote: Vec<String>,
    only_local: Vec<String>
}

#[derive(PartialEq)]
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
    //TODO check if present remote notes were explicitely deleted locally

    let mut update_actions = get_update_actions(&metadata);

    // let mut hist: Vec<(UpdateAction,&NotesMetadata)> = add_delete_actions.into_iter().filter(|(_,metadata)| {
    //     let s: Vec<&&NotesMetadata> = update_actions.iter().map(|(_,b)|b).collect();
    //     !s.contains(&metadata)
    // }).collect();

    update_actions.append(&mut add_delete_actions);
    execute_actions(&update_actions, session);

}

fn get_update_actions(remote_notes: &Vec<NotesMetadata>) -> Vec<(UpdateAction, &NotesMetadata)> {

    remote_notes.into_iter().map( |mail_headers| {
        let location = profile::get_notes_dir() + &mail_headers.subfolder + "/" + &mail_headers.subject_with_identifier();
        debug!("Compare {}", location);

        let hash_location = profile::get_notes_dir() + &mail_headers.subfolder + "/." + &mail_headers.subject_with_identifier() + "_hash";
        let hash_loc_path = std::path::Path::new(&hash_location);

        if hash_loc_path.exists() {
            let f = File::open(hash_loc_path).unwrap();
            let local_metadata : NotesMetadata = serde_json::from_reader(f).unwrap();

            let local_uuid = local_metadata.message_id();
            let oldest_remote_uuid = local_metadata.old_remote_id;

            let remote_uuid = mail_headers.message_id();

            if remote_uuid == local_uuid {
                debug!("Same: {}", mail_headers.subfolder.to_string() + "/" + &mail_headers.subject());
                return Some((DoNothing, mail_headers))
            } else if remote_uuid != local_uuid && oldest_remote_uuid.is_none() {
                info!("Changed Remotely: {}", mail_headers.subject());
                return Some((UpdateLocally, mail_headers))
            } else if oldest_remote_uuid.is_some() && oldest_remote_uuid.clone().unwrap() == remote_uuid {
                info!("Changed Locally: {}", mail_headers.subject());
                return Some((UpdateRemotely, mail_headers))
            } else if oldest_remote_uuid.is_some() && remote_uuid != local_uuid {
                info!("Changed on both ends, needs merge: {}", &mail_headers.subject());
                return Some((Merge, mail_headers))
            }
        }
        return None
    }).filter_map(|e| {
        if e.is_some() && e.as_ref().unwrap().0 != DoNothing {
            e
        } else {
            None
        }
    }).collect::<Vec<(UpdateAction, &NotesMetadata)>>()
}

fn update_remotely(metadata: &NotesMetadata, session: &mut Session<TlsStream<TcpStream>>) {
    match apple_imap::update_message(session, metadata) {
         Ok(_) => {
             let new_metadata = NotesMetadata {
                 header: metadata.header.clone(),
                 old_remote_id: None,
                 subfolder: metadata.subfolder.to_string(),
                 locally_deleted: false,
                 //TODO pass here new uid
                 uid: metadata.uid
             };

             io::save_metadata_to_file(&new_metadata);
        }, Err(e) => {
            warn!("Could not push changes to mail-server {} for {}", metadata.subject(), e.to_string());
        }
    }

}

fn update_locally(metadata: &NotesMetadata, session: &mut Session<TlsStream<TcpStream>>) {
    let note = apple_imap::fetch_single_note(session,metadata).unwrap();
    io::save_note_to_file(&note);
}

fn execute_actions(actions: &Vec<(UpdateAction, &NotesMetadata)>, session:  &mut Session<TlsStream<TcpStream>>) {
    actions.iter().for_each(|(action, metadata)| {
        match action {
            AddRemotely => {
                create_mailbox(session, metadata);
                session.select(&metadata.subfolder);
                update_remotely(metadata, session);
            },
            UpdateRemotely => {
                update_remotely(metadata, session);
            },
            UpdateAction::UpdateLocally | UpdateAction::AddLocally => {
                update_locally(metadata, session);
            }
            _ => {
                unimplemented!("Action is not implemented");
            }
        }
    })
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
    let local_messages = get_local_messages();



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

    let only_local_count = only_local.len();
    let only_remote_count = only_remote.len();

    println!("Found {} remote_only_notes", only_remote_count);
    println!("Found {} local_only_notes", only_local_count);

    only_local.append(&mut only_remote);
    only_local

}