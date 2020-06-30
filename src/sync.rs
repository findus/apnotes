extern crate log;
extern crate walkdir;


use note::{NotesMetadata, Note, LocalNote, NoteTrait, HeaderParser};
use apple_imap::{fetch_notes, fetch_headers};
use std::net::TcpStream;
use native_tls::TlsStream;
use imap::Session;
use self::log::{info, debug};
use std::fs::File;
use self::walkdir::WalkDir;
use std::collections::HashSet;

pub struct RemoteDifference {
    only_remote: Vec<String>,
    only_local: Vec<String>
}

fn get_updated_notes(remote_notes: &Vec<NotesMetadata>) -> Vec<String> {

    remote_notes.iter().map(move |mail_headers| {
        let location = "/home/findus/.notes/".to_string() + &mail_headers.subfolder + "/" + &mail_headers.subject_with_identifier();
        debug!("Compare {}", location);

        let hash_location = "/home/findus/.notes/".to_string() + &mail_headers.subfolder + "/." + &mail_headers.subject_with_identifier() + "_hash";
        let hash_loc_path = std::path::Path::new(&hash_location);
        if hash_loc_path.exists() {
            let f = File::open(hash_loc_path).unwrap();
            let local_metadata : NotesMetadata = serde_json::from_reader(f).unwrap();

            let local_uuid = local_metadata.message_id();
            let oldest_remote_uuid = local_metadata.old_remote_id;

            let remote_uuid = mail_headers.message_id();


            if remote_uuid == local_uuid {
                debug!("Same: {}", mail_headers.subfolder.to_string() + "/" + &mail_headers.subject());
            } else if remote_uuid != local_uuid && oldest_remote_uuid.is_none() {
                info!("Changed Remotely: {}", mail_headers.subject());
            } else if oldest_remote_uuid.is_some() && oldest_remote_uuid.clone().unwrap() == local_uuid {
                info!("Changed Locally: {}", mail_headers.subject());
            } else if oldest_remote_uuid.is_some() && remote_uuid != local_uuid {
                info!("Changed on both ends, needs merge: {}", &mail_headers.subject());
            }
        }
        return None
    }).filter_map(|e|e).collect::<Vec<String>>()
}

pub fn sync(session: &mut Session<TlsStream<TcpStream>>) {
    let metadata = fetch_headers(session);

    let _added_deleted_notes = get_added_deleted_notes(&metadata);
    //TODO check if present remote notes were explicitely deleted locally

    let _updated_notes = get_updated_notes(&metadata);

    let _local_messages = get_local_messages();

}

fn get_local_messages() -> Vec<LocalNote> {

    WalkDir::new("/home/findus/.notes/")
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.file_name().to_str().unwrap().to_string().starts_with("."))
        .map(| d| {
            LocalNote::new(d)
        }).collect()
}


pub fn get_added_deleted_notes(metadata: &Vec<NotesMetadata>) -> RemoteDifference {

    info!("Loading local messages");
    let local_messages = get_local_messages();

    let local_titles: HashSet<String> = local_messages.iter().map(|note| note.metadata.identifier()).collect();
    let remote_titles: HashSet<String> = metadata.iter().map(|mail_headers| mail_headers.identifier()).collect();

    let local_size = local_titles.len();
    info!("Found {} local notes", local_size);

    let _remote_size = remote_titles.len();
    info!("Found {} remote messages", local_size);


    let only_local: Vec<String> = local_titles.difference(&remote_titles).map(|e| e.to_owned()).collect();
    let only_remote: Vec<String> = remote_titles.difference(&local_titles).map(|e| e.to_owned()).collect();


    let only_local_count = only_local.len();
    let only_remote_count = only_remote.len();

    println!("Found {} remote_only_notes", only_remote_count);
    println!("Found {} local_only_notes", only_local_count);

    RemoteDifference {
        only_remote,
        only_local
    }

}