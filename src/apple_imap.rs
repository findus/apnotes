extern crate imap;
extern crate native_tls;
extern crate mailparse;
extern crate log;
extern crate regex;

use std::fs::File;
use self::log::{info, warn, debug};
use std::io::Write;
use self::imap::Session;
use std::net::TcpStream;
use self::native_tls::TlsStream;
use self::imap::types::{ZeroCopy, Fetch, Uid};

use std::borrow::Borrow;

use self::regex::Regex;
use note::{Note, NoteTrait};
use apple_imap;
use converter;
use profile;
use std::collections::HashMap;

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
            f.write_all(converter::convert2md(&note.body).as_bytes()).expect("Unable to write file")
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
    let messages = match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            let ids = messages.iter().map(|f| f.uid).collect();
            map.insert(folder_name, ids);
        }
        Err(_error) => {
            warn!("Could not load notes from {}!", &folder_name.to_string());
            let mut vec: Vec<Option<u32>> = Vec::new();
            map.insert(folder_name, vec);
        }
    };
    map
}

pub fn get_messages_from_foldersession(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> Vec<Note> {

    if let Some(result) = session.select(&folder_name).err() {
        warn!("Could not select folder {} [{}]", folder_name, result)
    }
    let messages_result = session.fetch("1:*", "(RFC822 RFC822.HEADER)");
    let messages = match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            get_notes(messages)
        }
        Err(_error) => {
            warn!("Could not load notes from {}!", &folder_name.to_string());
            Vec::new()
        }
    };
    messages
}

pub fn get_notes(fetch_vector: ZeroCopy<Vec<Fetch>>) -> Vec<Note> {
    fetch_vector.into_iter().map(|fetch| {
        let headers = get_headers(fetch.borrow());
        let body = get_body(fetch.borrow());
        Note {
            mail_headers: headers,
            body: body.unwrap_or("mist".to_string()),
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
            f.write_all(converter::convert2md(&note.body).as_bytes()).expect("Unable to write file")
        });
    });
}