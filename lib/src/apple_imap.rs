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
use self::imap::types::{Fetch};
use model::{NotesMetadata};
use note::{RemoteNoteHeaderCollection, RemoteNoteMetaData, LocalNote, IdentifyableNote};
use ::{apple_imap};
use ::{profile, converter};
use imap::error::Error;
use converter::convert_to_html;
use imap::types::Mailbox;
#[cfg(test)]
extern crate mockall;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

pub trait ImapSession<S> {

}

pub struct TlsImapSession {
    session: Session<TlsStream<TcpStream>>
}

impl TlsImapSession {
    fn login() -> Session<TlsStream<TcpStream>> {
        let profile = self::profile::load_profile();

        let domain = profile.imap_server.as_str();
        info!("Connecting to {}", domain);

        let tls = native_tls::TlsConnector::builder().build().unwrap();

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
}

impl ImapSession<Session<TlsStream<TcpStream>>> for TlsImapSession {

}

#[cfg_attr(test, automock)]
pub trait MailService<T: 'static ,S: 'static + ImapSession<T>> {
    fn fetch_mails(&self) -> Vec<LocalNote>;
    fn fetch_headers(&mut self) -> Vec<RemoteNoteMetaData>;
    fn create_mailbox(&mut self, note: &NotesMetadata) -> Result<(), Error>;
    fn fetch_note_content(&mut self, subfolder: &str, uid: i64) -> Result<String, Error>;
    fn get_session(&self) -> T;
    fn update_message(&mut self, localnote: &LocalNote) -> Result<u32, Error>;
    fn select(&mut self, folder: &str) -> Result<Mailbox, Error>;
}

pub struct MailServiceImpl {
    session: TlsImapSession
}

impl MailServiceImpl {
    pub fn new_with_login() -> MailServiceImpl {
        MailServiceImpl {
            session: TlsImapSession { session: TlsImapSession::login() }
        }
    }

    pub fn fetch_note_content(&mut self, subfolder: &str, uid: i64) -> Result<String,Error> {

        if let Some(result) = self.session.session.select(&subfolder).err() {
            warn!("Could not select folder {} [{}]", &subfolder, result)
        }

        let messages_result = self.session.session.uid_fetch(uid.to_string(), "(RFC822 UID)");
        match messages_result {
            Ok(message) => {
                debug!("Message Loading for message with UID {} successful", uid);
                let first_message = message.first().expect("Expected message");
                Ok(self.get_body(first_message).expect("Expected note body, found none"))
            },
            Err(error) => {
                warn!("Could not load notes from {}! {}", &subfolder, error);
                Err(error)
            }
        }
    }

    /// Iterate thorugh all Note-Imap folders and fetches the mail header content plus
    /// the folder name.
    ///
    /// The generated dataset can be used to check for duplicated notes that needs
    /// to be mergedK
    pub fn fetch_headers(&mut self) -> RemoteNoteHeaderCollection {
        info!("Fetching Headers of Remote Notes...");
        let folders = self.list_note_folders();
        folders.iter().map(|folder_name| {
            self.fetch_headers_in_folder( folder_name.to_string())
        })
            .flatten()
            .collect()
    }

    pub fn create_folder(&mut self, mailbox: &str) {
        match self.session.session.create(&mailbox) {
            Err(e) => warn!("warn {}", e),
            _ => {}
        };
    }

    pub fn copy_uid(&mut self, id: &str, mailbox: &str) {

        if let Some(error) = self.session.session.select(mailbox).and_then( |_| {
            self.session.session.uid_copy(id, &mailbox)
        }).err() {
            warn!("warn {}", error)
        }
    }


    pub fn fetch_headers_in_folder(&mut self, folder_name: String) -> Vec<RemoteNoteMetaData> {
        if let Some(result) = self.session.session.select(&folder_name).err() {
            warn!("Could not select folder {} [{}]", &folder_name, result)
        }
        let messages_result = self.session.session.fetch("1:*", "(RFC822.HEADER UID)");
        match messages_result {
            Ok(messages) => {
                debug!("Message Loading for {} successful", &folder_name.to_string());
                messages.iter().map( |fetch|{
                    self.get_headers(fetch, folder_name.clone())
                }).collect()
            },
            Err(error) => {
                warn!("Could not load notes from {}! Does this Folder contains any messages? {}", &folder_name.to_string(), error);
                Vec::new()
            }
        }
    }

    /**
    Returns empty vector if something fails
    */
    fn get_headers(&mut self,fetch: &Fetch, foldername: String) -> RemoteNoteMetaData {
        match mailparse::parse_headers(fetch.header().unwrap()) {
            Ok((header, _)) => {
                let  headers = header.into_iter().map(|h| (h.get_key().unwrap(), h.get_value().unwrap())).collect();
                RemoteNoteMetaData {
                    headers,
                    folder: foldername.to_string(),
                    uid: fetch.uid.unwrap() as i64,
                }
            },
            _ => panic!("No Headers presentfor fetch with uid {}", fetch.uid.unwrap())
        }
    }

    fn get_body(&mut self,fetch: &Fetch) -> Option<String> {
        match mailparse::parse_mail(fetch.body()?) {
            Ok(body) => body.get_body().ok(),
            _ => None
        }
    }

    pub fn list_note_folders(&mut self) -> Vec<String> {
        let folders_result = self.session.session.list(None, Some("Notes*"));
        let result: Vec<String> = match folders_result {
            Ok(result) => {
                let names: Vec<String> = result.iter().map(|name| name.name().to_string()).collect();
                names
            }
            _ => Vec::new()
        };

        return result;
    }

    /// Deletes all notes remotely that have the uuid provided by local_note, expect
    /// the note with uid_to_keep
    fn delete_old_mergeable_notes(&mut self,
                                  local_note: &LocalNote,
                                  uid_to_keep: u32) -> Result<(),Error>
    {
        self.session.session.
            select(&local_note.metadata.folder())
            .and_then(|_| self.session.session.uid_search(
                format!("HEADER X-Universally-Unique-Identifier {}", local_note.body[0].message_id)))
            .and_then(|uids| {
                let uids: Vec<String> = uids.into_iter()
                    .filter(|uid| uid != &uid_to_keep )
                    .map(|x| (x.to_string())).collect();
                for uid in uids {
                    info!("Will delete remote note with uid: {}", uid);
                    self.flag_as_deleted(uid)?;
                }
                self.delete_flagged()?;
                Ok(())
            })
    }

    fn delete_flagged(&mut self) -> imap::error::Result<Vec<u32>> {
        self.session.session.expunge()
    }

    fn flag_as_deleted(&mut self, uid: String) -> imap::error::Result<()> {
        // If note was new everything is ready
        self.session.session.uid_store(uid, "+FLAGS.SILENT (\\Seen \\Deleted)".to_string())?;
        Ok(())
    }
}




impl MailService<Session<TlsStream<TcpStream>>,TlsImapSession> for MailServiceImpl {
    fn fetch_mails(&self) -> Vec<LocalNote> {
        unimplemented!()
    }

    /// Iterate thorugh all Note-Imap folders and fetches the mail header content plus
    /// the folder name.
    ///
    /// The generated dataset can be used to check for duplicated notes that needs
    /// to be merged
    fn fetch_headers(&mut self) -> Vec<RemoteNoteMetaData> {
        info!("Fetching Headers of Remote Notes...");
        let folders = self.list_note_folders();
        folders.iter().map(|folder_name| {
            self.fetch_headers_in_folder(folder_name.to_string())
        })
            .flatten()
            .collect()
    }

    fn create_mailbox(&mut self, note: &NotesMetadata) -> Result<(), Error> {
        self.session.session.create(&note.subfolder).or(Ok(()))
    }

    fn fetch_note_content(&mut self, subfolder: &str, uid: i64) -> Result<String, Error> {
        if let Some(result) = self.session.session.select(&subfolder).err() {
            warn!("Could not select folder {} [{}]", &subfolder, result)
        }

        let messages_result = self.session.session.uid_fetch(uid.to_string(), "(RFC822 UID)");
        match messages_result {
            Ok(message) => {
                debug!("Message Loading for message with UID {} successful", uid);
                let first_message = message.first().expect("Expected message");
                Ok(self.get_body(first_message).expect("Expected note body, found none"))
            },
            Err(error) => {
                warn!("Could not load notes from {}! {}", &subfolder, error);
                Err(error)
            }
        }
    }

    fn get_session(&self) -> Session<TlsStream<TcpStream>> {
        unimplemented!()
    }

    /// Updates a local message, either if it got updated or if it is a new localnote
    /// This App should only support "merged" notes, notes that only have one body.
    ///
    /// If the passed localnote has >1 bodies it will reject it.
    fn update_message(&mut self, localnote: &LocalNote) -> Result<u32, Error> {
        //Todo check >1

        let headers = localnote.to_header_vector().iter().map( |(k,v)| {
            format!("{}: {}",k,v)
        })
            .collect::<Vec<String>>()
            .join("\n");

        // Updated message must be merged
        //let _content = converter::convert_to_html(&localnote.body.first().unwrap());

        let body = localnote.body.first().unwrap();
        let message = format!("{}\n\n{}",headers, convert_to_html(body));

        self.session.session
            // Write new message into the mailbox
            .append(&localnote.metadata.subfolder, message.as_bytes())
            // Select the appropriate mailbox, in which the updated message was saved
            .and_then(|_| self.session.session.select(&localnote.metadata.subfolder))
            // Set the old (overridden) message to "deleted", so that it can be expunged
            .and_then(|_| {
                if localnote.metadata.new == false {
                    self.flag_as_deleted(localnote.body.first().unwrap().uid.unwrap().to_string())
                } else {
                    Ok(())
                }
            })
            // Expunge them //TODO might need check if note is new, skip if note is new
            .and_then(|_| self.delete_flagged())
            // Search for the new message, to get the new UID of the updated message
            .and_then(|_| self.session.session.uid_search(format!("HEADER Message-ID {}", localnote.body[0].message_id)))
            // Get the first UID
            .and_then(|id| id.into_iter().collect::<Vec<u32>>().first().cloned().ok_or(imap::error::Error::Bad("no uid found".to_string())))
            // Save the new UID to the metadata file, also set seen flag so that mail clients dont get notified on updated message
            .and_then(|new_uid| self.session.session.uid_store(format!("{}", &new_uid), "+FLAGS.SILENT (\\Seen)".to_string()).map(|_| new_uid))
            // Delete dangling remote non merged notes
            .and_then(|new_uid| self.delete_old_mergeable_notes(&localnote, new_uid).map(|_| new_uid))
    }

    fn select(&mut self, folder: &str) -> Result<Mailbox, Error> {
        //todo wrap mailbox type?
        self.session.session.select(folder)
    }
}


/*
pub fn fetch_single_note(session: &mut Session<TlsStream<TcpStream>>, metadata: &NotesMetadata) -> Option<NotesMetadata> {
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
}
*/

/*
pub fn fetch_notes(session: &mut Session<TlsStream<TcpStream>>) -> Vec<NotesMetadata> {
let folders = list_note_folders(session);
info!("Loading remote messages");
folders.iter().map(|folder_name| {
    apple_imap::get_messages_from_foldersession(session, folder_name.to_string())
})
    .flatten()
    .collect()
}
*/

/*
pub fn get_messages_from_foldersession(session: &mut Session<TlsStream<TcpStream>>, folder_name: String) -> Vec<NotesMetadata> {

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
}
*/


/*
pub fn get_notes(fetch_vector: ZeroCopy<Vec<Fetch>>, folder_name: String) -> Vec<Fetch> {

    let connection = crate::db::establish_connection();
    fetch_vector.into_iter().map(|fetch| {
        let headers = get_headers(&fetch);
        let body = get_body(&fetch);

       match crate::db::fetch_single_note(&connection,headers.identifier()) {
           Ok(Some((metadata,notes))) =>  {
               //Note aleady exists, append
               debug!("Found note for fetched note, append to body table ({})", metadata.uuid);
               let body = Body {
                   message_id: headers.message_id(),
                   text: body,
                   uid: Some(fetch.uid.unwrap() as i64),
                   metadata_uuid: metadata.uuid
               };
               crate::db::append_note(&connection,&body);
           },
           Ok(None) => {
               crate::db::insert_into_db(&connection, (
                   &NotesMetadata {
                       old_remote_id: None,
                       subfolder: folder_name.clone(),
                       locally_deleted: false,
                       new: false,
                       date: headers.date(),
                       uuid: headers.identifier(),
                       mime_version: headers.mime_version()
                   },
                   &Body {
                       message_id: headers.message_id(),
                       text: body,
                       uid: Some(fetch.uid.unwrap() as i64),
                       metadata_uuid: headers.identifier()
                   }
                   ));
           },
           Err(e) => {
               panic!("{}",e.to_string());
           }
       }

        //TODO check if duplicate notes are present that needs to be merged
        let body = Body {
            message_id: "".to_string(),
            uid: Some(fetch.uid.expect(&format!("No UID found for {}", headers.identifier())) as i64),
            text: None,
            metadata_uuid: "".to_string()
        };
        NotesMetadata::new(headers, folder_name.clone(), fetch.uid.unwrap(), Some(vec![body]))
    }).collect()
}
*/