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
use model::{NotesMetadata, Body};

use std::borrow::Borrow;
use note::{NoteHeader, HeaderParser};
use ::{apple_imap, converter};
use profile;
use imap::error::Error;

pub trait MailFetcher {
    fn fetch_mails() -> Vec<NotesMetadata>;
}

pub fn login() -> Session<TlsStream<TcpStream>> {

    let profile = self::profile::load_profile();

    let domain = profile.imap_server.as_str();
    info!("Connecting to {}", domain);
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

/*pub fn fetch_notes(session: &mut Session<TlsStream<TcpStream>>) -> Vec<NotesMetadata> {
    let folders = list_note_folders(session);
    info!("Loading remote messages");
    folders.iter().map(|folder_name| {
        apple_imap::get_messages_from_foldersession(session, folder_name.to_string())
    })
        .flatten()
        .collect()
}*/

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

pub fn fetch_uid_of_message_id(_session: &mut Session<TlsStream<TcpStream>>, _metadata: &NotesMetadata) {

}

/*pub fn fetch_single_note(session: &mut Session<TlsStream<TcpStream>>, metadata: &NotesMetadata) -> Option<NotesMetadata> {
    if let Some(result) = session.select(&metadata.subfolder).err() {
        warn!("Could not select folder {} [{}]", &metadata.subfolder, result)
    }

    assert_eq!(&metadata.needs_merge(), &false);
    let note = &metadata.notes
        .first()
        .expect(&format!("No notes available for {}", metadata.uuid.clone()));

    let uid = &note.uid
        .expect(&format!("Note does not have a UID {}", metadata.uuid.clone()))
        .to_string();

    let messages_result = session.uid_fetch(uid, "(RFC822 RFC822.HEADER UID)");
    match messages_result {
        Ok(message) => {
            debug!("Message Loading for {} successful", &note.subject());

            let first_message = message.first().unwrap();
            let headers = get_headers(message.first().unwrap());

            let body = Body {
                message_id: headers.message_id(),
                body: get_body(first_message).unwrap(),
                uid: first_message.uid.map(|uid| uid as i64)
            };

            Some(
                NotesMetadata::new(
                    get_headers(message.first().unwrap()),
                    metadata.subfolder.clone(),
                    first_message.uid.unwrap(),
                    Some(vec![body])
                )
            )

        },
        Err(error) => {
            warn!("Could not load notes from {}! {}", &metadata.subfolder, error);
            None
        }
    }
}*/

pub fn fetch_headers_in_folder(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> Vec<NotesMetadata> {
    if let Some(result) = session.select(&folder_name).err() {
        warn!("Could not select folder {} [{}]", folder_name, result)
    }
    let messages_result = session.fetch("1:*", "(RFC822.HEADER UID)");
    match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            messages.iter().map(|fetch| {
                NotesMetadata::new(
                    get_headers(fetch),
                    folder_name.clone(),
                    fetch.uid.expect("No UID found"),
                    // todo how to handle this, allow zero len data because only metadata was fetched?
                    None
                )
            }).collect()
        },
        Err(error) => {
            warn!("Could not load notes from {}! Does this Folder contains any messages? {}", &folder_name.to_string(), error);
            Vec::new()
        }
    }
}

/*pub fn get_messages_from_foldersession(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> Vec<NotesMetadata> {

    if let Some(result) = session.select(&folder_name).err() {
        warn!("Could not select folder {} [{}]", folder_name, result)
    }
    let messages_result = session.fetch("1:*", "(RFC822 RFC822.HEADER UID)");
    let messages = match messages_result {
        Ok(messages) => {
            debug!("Message Loading for {} successful", &folder_name.to_string());
            get_notes(messages, folder_name)
        }
        Err(error) => {
            warn!("Could not load notes from {}! {}", &folder_name.to_string(), error);
            Vec::new()
        }
    };
    messages
}*/

/*pub fn get_notes(fetch_vector: ZeroCopy<Vec<Fetch>>, folder_name: String) -> Vec<NotesMetadata> {
    fetch_vector.into_iter().map(|fetch| {
        let headers = get_headers(fetch.borrow());
        //TODO check if duplicate notes are present that needs to be merged
        let body = get_body(fetch.borrow());
        let body = Body {
            message_id: "".to_string(),
            body: body.expect(&format!("No body found for {}", headers.identifier())),
            uid: Some(fetch.uid.expect(&format!("No UID found for {}", headers.identifier())) as i64),
        };
        NotesMetadata::new(headers, folder_name.clone(), fetch.uid.unwrap(), Some(vec![body]))
    }).collect()
}*/

/**
Returns empty vector if something fails
*/
pub fn get_headers(fetch: &Fetch) -> NoteHeader {
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
/*
pub fn update_message(session: &mut Session<TlsStream<TcpStream>>, metadata: &NotesMetadata) -> Result<u32, Error> {
    //TODO wenn erste Zeile != Subject: Subject = Erste Zeile
    let uid = if metadata.uid.is_some() {
        format!("{}", metadata.uid.unwrap())
    } else {
        format!("new")
    };

  /*  let headers = metadata.header.iter().map( |(k,v)| {
        //TODO make sure that updated message has new message-id
        format!("{}: {}",k,v)
    })
        .collect::<Vec<String>>()
        .join("\n");*/

    let _content = converter::convert_to_html(&metadata);

    //let message = format!("{}\n\n{}",headers, content);
    let message = "".clone();

    session
        //Write new message into the mailbox
        .append(&metadata.subfolder, message.as_bytes())
        //Select the appropriate mailbox, in which the updated message was saved
        .and_then(|_| session.select(&metadata.subfolder))
        // Set the old (overridden) message to "deleted", so that it can be expunged
        .and_then(|_| {
            if metadata.new {
                Ok(())
            } else {
                session.uid_store(&uid, "+FLAGS.SILENT (\\Seen \\Deleted)".to_string()).map(|_| ())
            }
        })
        //Expunge them
        .and_then(|_| {
            if metadata.new {
                Ok(())
            } else {
                session.uid_expunge(&uid).map(|_| ())
            }
        })
        //Search for the new message, to get the new UID of the updated message
        .and_then(|_| session.uid_search(format!("HEADER Message-ID {}", metadata.message_id)))
        //Get the first UID
        .and_then(|id| id.into_iter().collect::<Vec<u32>>().first().cloned().ok_or(imap::error::Error::Bad("no uid found".to_string())))
        //Save the new UID to the metadata file, also set seen flag so that mail clients dont get notified on updated message
        .and_then(|new_uid| {
            session.uid_store(format!("{}", &new_uid), "+FLAGS.SILENT (\\Seen)".to_string()).map(|_| new_uid)
        })
}*/


pub fn create_mailbox(session: &mut Session<TlsStream<TcpStream>>, note: &NotesMetadata) -> Result<(),Error> {
    session.create(&note.subfolder).or(Ok(()))
}