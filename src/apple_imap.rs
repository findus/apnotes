extern crate imap;
extern crate native_tls;
extern crate mailparse;
extern crate log;
extern crate regex;
extern crate fasthash;
extern crate walkdir;
extern crate jfs;
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use std::fs::{File, FileType};
use self::log::{info, warn, debug};
use std::io::Write;
use self::imap::Session;
use std::net::TcpStream;
use self::native_tls::TlsStream;
use self::imap::types::{ZeroCopy, Fetch};
use self::serde::{Serialize, Deserialize};

use std::borrow::Borrow;
use note::{Note, NoteTrait, LocalNote};
use apple_imap;
use converter;
use profile;
use std::collections::{HashMap, HashSet};
use self::fasthash::metro;
use self::walkdir::WalkDir;
use note::NotesMetadata;


use self::jfs::Store;
use std::hash::Hash;


pub trait MailFetcher {
    fn fetch_mails() -> Vec<Note>;
}

pub fn login() -> Session<TlsStream<TcpStream>> {

    let profile = self::profile::load_profile();

    let domain = profile.imap_server.as_str();
    let tls = native_tls::TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let imap_session = client
        .login(profile.username, profile.password)
        .map_err(|e| e.0);

    return imap_session.unwrap();
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

pub struct Remote_Difference {
    only_remote: Vec<String>,
    only_local: Vec<String>
}

pub fn fetch_notes(session: &mut Session<TlsStream<TcpStream>>) -> Vec<Note> {
    info!("Comparing local and remote messages...");
    let folders = list_note_folders(session);
    info!("Loading remote messagges");
    folders.iter().map(|folder_name| {
        apple_imap::get_messages_from_foldersession(session, folder_name.to_string())
    })
        .flatten()
        .collect()
}

pub fn get_added_deleted_notes(remote_notes: &Vec<Note>) -> Remote_Difference {

    info!("Loading local messages");
    let local_messages = get_local_messages();

    let local_titles: HashSet<String> = local_messages.iter().map(|note| note.identifier()).collect();
    let remote_titles: HashSet<String> = remote_notes.iter().map(|note| note.identifier()).collect();

    let local_size = local_titles.len();
    info!("Found {} local notes", local_size);

    let remote_size = remote_titles.len();
    info!("Found {} rempte messages", local_size);


    let only_local: Vec<String> = local_titles.difference(&remote_titles).map(|e| e.to_owned()).collect();
    let only_remote: Vec<String> = remote_titles.difference(&local_titles).map(|e| e.to_owned()).collect();


    let only_local_count = only_local.len();
    let only_remote_count = only_remote.len();

    println!("Found {} remote_only_notes", only_remote_count);
    println!("Found {} local_only_notes", only_local_count);

    Remote_Difference {
        only_remote,
        only_local
    }

}

fn get_updated_notes(remote_notes: &Vec<Note>) -> Vec<String> {

        remote_notes.iter().map(move |note| {
            let location = "/home/findus/.notes/".to_string() + note.folder.as_ref() + "/" + &note.subject();
            debug!("Compare {}", location);

            let hash_location = "/home/findus/.notes/".to_string() + note.folder.as_ref() + "/." + &note.subject() + "_hash";
            let hash_loc_path = std::path::Path::new(&hash_location);
            if hash_loc_path.exists() {
                let remote_hash = note.hash();
                let mut f = File::open(hash_loc_path).unwrap();
                let local_hash : NotesMetadata = serde_json::from_reader(f).unwrap();
                if remote_hash == local_hash.hash {
                    debug!("Same: {}", note.folder.to_string() + "/" + &note.subject());
                } else {
                    info!("Differ: {} [{}<->{}]", note.folder.to_string() + "/" + &note.subject(), local_hash.hash, remote_hash);
                    return Some(note.identifier().to_owned())
                }
            }
            return None
        }).filter_map(|e|e).collect::<Vec<String>>()
}

pub fn sync(session: &mut Session<TlsStream<TcpStream>>) {
    let notes = fetch_notes(session);

    let added_deleted_notes = get_added_deleted_notes(&notes);
    //TODO check if present remote notes were explicitely deleted locally

    let updated_notes = get_updated_notes(&notes);

    let local_messages = get_local_messages();

}

pub fn duplicate_notes_folder(session: &mut Session<TlsStream<TcpStream>>) {

    let folders = list_note_folders(session);

    folders.iter().for_each(|folder_name| {
        let _messages = apple_imap::get_messages_from_foldersession(session, folder_name.to_string());

        _messages.iter().for_each(|note| {
            let location = "/home/findus/.notes/".to_string() + folder_name + "/" + &note.subject().replace("/", "_").replace(" ", "_");
            info!("Save to {}", location);

            let path = std::path::Path::new(&location);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();

            let mut f = File::create(location).expect("Unable to create file");
            f.write_all(converter::convert2md(&note.body()).as_bytes()).expect("Unable to write file")
        });
    });
}

pub fn create_folder(session: &mut Session<TlsStream<TcpStream>>, mailbox: &str) {
    match session.create(&mailbox) {
        Err(e) => warn!("warn {}", e),
        _ => {}
    };
}

pub fn copy_uid(session: &mut Session<TlsStream<TcpStream>>, id: &str, mailbox: &str) {

    if let Some(error) = session.select(mailbox).and_then( |_| {
        session.uid_copy(id, &mailbox)
    }).err() {
        warn!("warn {}", error)
    }
}

pub fn get_uids(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> HashMap<String,Vec<Option<u32>>> {
    let mut map = HashMap::new();
    if let Some(result) = session.select(&folder_name).err() {
        warn!("Could not select folder {} [{}]", folder_name, result)
    }
    let messages_result = session.fetch("1:*", "(RFC822.HEADER UID)");
    match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            let ids = messages.iter().map(|f| f.uid).collect();
            map.insert(folder_name, ids);
        }
        Err(_error) => {
            warn!("Could not load notes from {}!", &folder_name.to_string());
            let vec: Vec<Option<u32>> = Vec::new();
            map.insert(folder_name, vec);
        }
    };
    map
}

pub fn get_messages_from_foldersession(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> Vec<Note> {

    if let Some(result) = session.select(&folder_name).err() {
        warn!("Could not select folder {} [{}]", folder_name, result)
    }
    let messages_result = session.fetch("1:*", "(RFC822 RFC822.HEADER UID)");
    let messages = match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            get_notes(messages, folder_name)
        }
        Err(_error) => {
            warn!("Could not load notes from {}!", &folder_name.to_string());
            Vec::new()
        }
    };
    messages
}

pub fn get_notes(fetch_vector: ZeroCopy<Vec<Fetch>>, folder_name: String) -> Vec<Note> {
    fetch_vector.into_iter().map(|fetch| {
        let headers = get_headers(fetch.borrow());
        let body = get_body(fetch.borrow());
        let hash_sequence = body.clone().unwrap_or("".to_string());
        let hash = metro::hash64(hash_sequence);
            Note {
                mail_headers: headers,
                body: body.clone().unwrap_or("".to_string()),
                hash,
                uid: fetch.uid.unwrap(),
                folder: folder_name.to_owned()
            }
    }).collect()
}

/**
Returns empty vector if something fails
*/
pub fn get_headers(fetch: &Fetch) -> Vec<(String, String)> {
    match mailparse::parse_headers(fetch.header().unwrap()) {
        Ok((header, _)) => header.into_iter().map(|h| (h.get_key().unwrap(), h.get_value().unwrap())).collect(),
        _ => Vec::new()
    }
}

pub fn get_body(fetch: &Fetch) -> Option<String> {
    match mailparse::parse_mail(fetch.body()?) {
        Ok(body) => body.get_body().ok(),
        _ => None
    }
}

pub fn list_note_folders(imap: &mut Session<TlsStream<TcpStream>>) -> Vec<String> {
    let folders_result = imap.list(None, Some("Notes*"));
    let result: Vec<String> = match folders_result {
        Ok(result) => {
            let names: Vec<String> = result.iter().map(|name| name.name().to_string()).collect();
            names
        }
        _ => Vec::new()
    };

    return result;
}

pub fn save_all_notes_to_file(session: &mut Session<TlsStream<TcpStream>>) {
    let folders = list_note_folders(session);

    folders.iter().for_each(|folder_name| {
        let _messages = apple_imap::get_messages_from_foldersession(session, folder_name.to_string());

        _messages.iter().for_each(|note| {
            let location = "/home/findus/.notes/".to_string() + folder_name + "/" + &note.subject().replace("/", "_").replace(" ", "_");
            info!("Save to {}", location);

            let path = std::path::Path::new(&location);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();

            let mut f = File::create(location).expect("Unable to create file");
            f.write_all(converter::convert2md(&note.body()).as_bytes()).expect("Unable to write file");


            let location = "/home/findus/.debug_html/".to_string() + folder_name + "/" + &note.subject().replace("/", "_").replace(" ", "_");
            info!("Save to {}", location);

            let path = std::path::Path::new(&location);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();

            let mut f = File::create(location).expect("Unable to create file");
            f.write_all(&note.body().as_bytes()).expect("Unable to write file");


            let location = "/home/findus/.notes/".to_string() + folder_name + "/." + &note.subject().replace("/", "_").replace(" ", "_") + "_hash";
            info!("Save hash to {}", location);

            let path = std::path::Path::new(&location);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();


            let hash = metro::hash64(&note.body().as_bytes());

            let mut f = File::create(&location).expect(format!("Unable to create hash file for {}", location).as_ref());

            let note = note.mail_headers.clone();

            let dd = NotesMetadata {
                header: note,
                hash
            };


            serde_json::to_writer(f, &dd);

        });
    });
}