extern crate imap;
extern crate native_tls;
extern crate mailparse;
extern crate log;
extern crate regex;
extern crate jfs;
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use self::log::{info, warn, debug};
use self::imap::Session;
use std::net::TcpStream;
use self::native_tls::TlsStream;
use self::imap::types::{ZeroCopy, Fetch};

use std::borrow::Borrow;
use note::{Note, NotesMetadata, HeaderParser, LocalNote};
use ::{apple_imap, converter};
use profile;
use std::collections::{HashMap};
use imap::error::Error;

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

pub fn fetch_notes(session: &mut Session<TlsStream<TcpStream>>) -> Vec<Note> {
    let folders = list_note_folders(session);
    info!("Loading remote messagges");
    folders.iter().map(|folder_name| {
        apple_imap::get_messages_from_foldersession(session, folder_name.to_string())
    })
        .flatten()
        .collect()
}

pub fn fetch_headers(session: &mut Session<TlsStream<TcpStream>>) -> Vec<NotesMetadata> {
    info!("Fetching Headers of Remote Notes...");
    let folders = list_note_folders(session);
    folders.iter().map(|folder_name| {
        apple_imap::fetch_headers_in_folder(session, folder_name.to_string())
    })
        .flatten()
        .collect()
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

pub fn fetch_single_note(session: &mut Session<TlsStream<TcpStream>>, metadata: &NotesMetadata) -> Option<Note> {
    if let Some(result) = session.select(&metadata.subfolder).err() {
        warn!("Could not select folder {} [{}]", &metadata.subfolder, result)
    }
    let messages_result = session.uid_fetch(metadata.uid.to_string(), "(RFC822 RFC822.HEADER UID)");
    match messages_result {
        Ok(message) => {
            debug!("Message Loading for {} successful", &metadata.subject());
            let first_message = message.first().unwrap();

            let new_metadata = NotesMetadata {
                header: get_headers(message.first().unwrap()),
                old_remote_id: None,
                subfolder: metadata.subfolder.clone(),
                locally_deleted: false,
                uid: first_message.uid.unwrap()
            };

            Some(
                Note {
                    mail_headers: new_metadata,
                    folder: metadata.subfolder.to_string(),
                    body: get_body(first_message).unwrap()
                }
            )
        },
        Err(_error) => {
            warn!("Could not load notes from {}!", &metadata.subfolder);
            None
        }
    }
}

pub fn fetch_headers_in_folder(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> Vec<NotesMetadata> {
    if let Some(result) = session.select(&folder_name).err() {
        warn!("Could not select folder {} [{}]", folder_name, result)
    }
    let messages_result = session.fetch("1:*", "(RFC822.HEADER UID)");
    match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            messages.iter().map(|f| {
                NotesMetadata {
                    header: get_headers(f),
                    old_remote_id: None,
                    subfolder: folder_name.clone(),
                    locally_deleted: false,
                    uid: f.uid.unwrap()
                }
            }).collect()
        },
        Err(_error) => {
            warn!("Could not load notes from {}!", &folder_name.to_string());
            Vec::new()
        }
    }
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
            Note {
                mail_headers: NotesMetadata { header: headers, old_remote_id: None, subfolder: folder_name.clone(), locally_deleted: false, uid: fetch.uid.unwrap() },
                body: body.clone().unwrap_or("".to_string()),
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

pub fn update_message(session: &mut Session<TlsStream<TcpStream>>, metadata: &NotesMetadata) -> Result<(), Error> {
    //TODO wenn erste Zeile != Subject: Subject = Erste Zeile
    let uid = format!("{}", metadata.uid);

    let headers = metadata.header.iter().map( |(k,v)| {
        //TODO make sure that updated message has new message-id
        format!("{}: {}",k,v)
    })
        .collect::<Vec<String>>()
        .join("\n");

    let content = converter::convert2Html(metadata);

    let message = format!("{}\n\n{}",headers, content);

    match session.append(&metadata.subfolder, message.as_bytes())
        .and_then(|_| session.select(&metadata.subfolder))
        .and_then(|_| session.uid_store(&uid, "+FLAGS.SILENT (\\Seen \\Deleted)".to_string().replace("\\\\","\\")))
        .and_then(|_| session.uid_expunge(&uid)) {
        Err(e)  => {
            println!("{}",e);
            Err(imap::error::Error::No("no".to_owned()))
        },
        _ => Ok(()),
    }
}

pub fn create_message(session: &mut Session<TlsStream<TcpStream>>, note: &NotesMetadata) {

}

pub fn create_mailbox(session: &mut Session<TlsStream<TcpStream>>, note: &NotesMetadata) {
    session.create(&note.subfolder);
}