extern crate log;
extern crate walkdir;
extern crate glob;
extern crate itertools;
#[cfg(test)]
extern crate ctor;

use self::itertools::Itertools;
use note::{HeaderParser, LocalNote, IdentifyableNote, RemoteNoteHeaderCollection, RemoteNoteMetaData};
use self::log::*;
use std::collections::HashSet;
use ::note::{GroupedRemoteNoteHeaders};

use sync::UpdateAction::AddLocally;



use imap::Session;
use native_tls::TlsStream;
use std::net::TcpStream;
use diesel::SqliteConnection;
use model::{NotesMetadata, Body};
use error::UpdateError::SyncError;
use error::UpdateError;
use error;

pub enum UpdateResult {
    Success()
}

/// Defines the Action that has to be done to the
/// message with the corresponding uuid
#[derive(Debug)]
pub enum UpdateAction<'a> {
    /// Deletes the note on the imap server
    /// Apply to all notes that:
    ///     have their "locally_deleted" Flag set to true inside the db
    ///
    ///     First Argument: Subfolder
    ///     Second Argument: imap-uid
    DeleteRemote(&'a LocalNote),
    /// Apply to all notes that
    ///     are not getting transmitted anymore and dont have the
    ///     "new" flag inside the db
    DeleteLocally(String),
    /// Apply to all notes that:
    ///     have their "locally_edited" flag set
    ///     their "old_remote_id" value equals the remotes message-id
    UpdateRemotely(String),
    /// Apply to all notes that:
    ///     have their locally_edited flag set to false
    ///     remotes message-id != the locals message-id
    UpdateLocally(String),
    /// Apply to all notes that:
    ///     have old_remote id set to non null string
    ///     remotes message-id != the locals message-id
    ///   OR
    ///     Metadata has > 1 bodies as entries
    Merge(String),
    /// Apply to all notes that:
    ///     have new flag set to true
    ///     their uuid is not present remotely
    AddRemotely(&'a LocalNote),
    /// Apply to all notes that:
    ///
    ///     their uuid is not present locally
    ///     first arg: folder
    ///     second arg: imap-uid
    AddLocally(&'a RemoteNoteHeaderCollection),
    DoNothing
}

/// Iterates through all provided local notes and checks if the deletion flag got set
/// If this is the case a DeleteRemote Actions gets returned for this note
///
/// If the local note has multiple non-merged bodies the deletion gets skipped
/// TODO: What to do if local note is flagged for deletion but got updated remotely
fn get_deleted_note_actions<'a>(_remote_note_headers: Option<&GroupedRemoteNoteHeaders>,
                            local_notes: &'a HashSet<LocalNote>) -> Vec<UpdateAction<'a>>
{
    let local_flagged_notes: Vec<UpdateAction> = local_notes
        .iter()
        .filter(|local_note| local_note.metadata.locally_deleted)
        .map(|deleted_local_note| {
            if deleted_local_note.body.len() > 1 {
                warn!("Note with uuid {} is not merged, skipping", deleted_local_note.metadata.uuid);
                UpdateAction::DoNothing
            } else {
                let _note_body = deleted_local_note.body.first().unwrap();
                UpdateAction::DeleteRemote(
                    deleted_local_note
                )
            }
        })
        .collect();
    info!("Found {} Notes that are going to be deleted remotely", &local_flagged_notes.len());
    local_flagged_notes
}

fn get_added_note_actions<'a>(remote_note_headers: &'a GroupedRemoteNoteHeaders,
                          local_notes: &HashSet<LocalNote>) -> Vec<UpdateAction<'a>> {

    let remote_uuids: HashSet<String> =
        remote_note_headers.iter().map(|item| item.uuid()).collect();

    let local_uuids: HashSet<String> =
        local_notes.iter().map(|item| item.uuid()).collect();

    let uuids: Vec<&String> = remote_uuids.difference(&local_uuids).collect();

    remote_note_headers
        .iter()
        .filter(|remote_header_collection|
            uuids.contains(&&remote_header_collection.uuid()))
        .map(|new_note|
                 UpdateAction::AddLocally(new_note )
        )
        .collect()
}

fn get_sync_actions<'a>(remote_note_headers: &'a GroupedRemoteNoteHeaders,
                        local_notes: &'a HashSet<LocalNote>) -> Vec<UpdateAction<'a>> {

    info!("Found {} local Notes", local_notes.len());
    info!("Found {} remote notes", remote_note_headers.len());
    let mut concated_actions = vec![];

    let mut delete_actions =
        get_deleted_note_actions(Some(&remote_note_headers), &local_notes);
    let mut add_actions =
        get_added_note_actions(&remote_note_headers, &local_notes);

    concated_actions.append(&mut delete_actions);
    concated_actions.append(&mut add_actions);
    concated_actions

     /*
    for noteheader in grouped_not_headers.drain() != None {

    }*/
    // check db if deletable notes are present
   /* grouped.into_iter().for_each(|mut note_header_collection| {

        let first_notes_headers =
            note_header_collection.pop()
                .expect("Could not find note headers");

        if note_header_collection.len() > 1 {
            warn!("Note [{}] has more than one body needs to be merged",
                  first_notes_headers.identifier());
        } else {
            let local_note = ::db::fetch_single_note(&db_connection, first_notes_headers.identifier())
                .expect("Error while querying local note");
            if local_note.is_none() {
                //Add locally
                let subfolder = first_notes_headers.folder().clone();
                let uid = first_notes_headers.uid();
                let notemetadata =
                    NotesMetadata::new(&first_notes_headers, subfolder.clone() );
                let text =
                    ::apple_imap::fetch_note_content(&mut imap_session, subfolder, uid);
                let body = Body {
                    message_id: first_notes_headers.message_id(),
                    text: Some(::converter::convert2md(&text.unwrap())),
                    uid: Some(uid),
                    metadata_uuid: notemetadata.uuid.clone()
                };
                ::db::insert_into_db(&db_connection,(&notemetadata,&body));
            }
        }
    });*/
}

pub fn sync(imap_session: &mut Session<TlsStream<TcpStream>>, db_connection: &SqliteConnection) {
    let headers = ::apple_imap::fetch_headers(imap_session);
    let grouped_not_headers = collect_mergeable_notes(headers);
    match ::db::fetch_all_notes(&db_connection) {
        Ok(fetches) => {
            let actions =
                get_sync_actions(&grouped_not_headers,&fetches);
                process_actions(imap_session,db_connection, &actions);
            println!("A: {}", &actions.len());
            for a in actions {
                println!("{:?}", a);
            }
        }
        Err(e) => {
            panic!("mist {}",e);
        }
    }
    //let actions =
}

pub fn process_actions<'a>(
    imap_connection: &mut Session<TlsStream<TcpStream>>,
    db_connection: &SqliteConnection,
    actions: &Vec<UpdateAction<'a>>) -> Vec<Result<(),UpdateError>> {
    actions
        .iter()
        .map(|action|{
        match action {
            UpdateAction::DeleteRemote(_note) => {
                unimplemented!();
            }
            UpdateAction::DeleteLocally(_) => {
                unimplemented!();
            }
            UpdateAction::UpdateRemotely(_) => {
                unimplemented!();
            }
            UpdateAction::UpdateLocally(_) => {
                unimplemented!();
            }
            UpdateAction::Merge(_) => {
                unimplemented!();
            }
            UpdateAction::AddRemotely(localnote) => {
                info!("{} changed locally, gonna sent updated file to imap server", &localnote.uuid());
                let metadata = &localnote.metadata;
                ::apple_imap::create_mailbox(imap_connection, metadata)
                    .map_err(|e| SyncError(e.to_string()))
                    .and_then(|_
                    | imap_connection.select(&metadata.subfolder)
                        .map_err(|e| SyncError(e.to_string())))
                    .and_then(|_| update_remotely(localnote, imap_connection))
                    .and_then(|uid| {
                        let body = localnote.body.first().unwrap();
                        let note = note!(
                            NotesMetadata {
                                old_remote_id: localnote.metadata.old_remote_id.clone(),
                                subfolder: localnote.metadata.subfolder.clone(),
                                locally_deleted: localnote.metadata.locally_deleted,
                                locally_edited: localnote.metadata.locally_edited,
                                new: localnote.metadata.new.clone(),
                                date: localnote.metadata.date.clone(),
                                uuid:localnote.metadata.uuid.clone(),
                                mime_version: localnote.metadata.mime_version.clone()
                            },
                            Body {
                                message_id: body.message_id.clone(),
                                text: body.text.clone(),
                                uid: Some(uid as i64),
                                metadata_uuid: body.metadata_uuid.clone()
                            }
                        );
                        ::db::update(db_connection, &note)
                            .map_err(|e| UpdateError::SyncError(e.to_string()))
                    })

            }
            AddLocally(noteheaders) => {
                match localnote_from_remote_header(imap_connection, noteheaders) {
                    Ok(note) => {
                        ::db::insert_into_db(db_connection, &note)
                            .and_then(|_| Ok(note.metadata.uuid))
                            .map_err(|e| UpdateError::IoError(e.to_string()))
                    }
                    Err(e) => { Err(e) }
                };
                Ok(())
            }
            UpdateAction::DoNothing => {
                unimplemented!();
            }
        }
    }).collect()
}

fn localnote_from_remote_header(imap_connection: &mut Session<TlsStream<TcpStream>>, noteheaders: &&Vec<RemoteNoteMetaData>) -> Result<LocalNote,UpdateError> {
    let bodies: Vec<Option<Body>> = noteheaders.into_iter().map(|single_remote_note| {
        (
            single_remote_note,
            ::apple_imap::fetch_note_content(imap_connection,
                                             &single_remote_note.folder,
                                             single_remote_note.uid)
        )
    }).map(|(remote_metadata, result)| {
        match result {
            Ok(body) => {
                Some(Body {
                    message_id: remote_metadata.headers.message_id(),
                    text: Some(body),
                    uid: Some(remote_metadata.uid),
                    metadata_uuid: remote_metadata.headers.uuid()
                })
            }
            Err(e) => {
                warn!("Could not receive message body: {}",e);
                None
            }
        }
    }).collect();

    if bodies.iter().filter(|entry| entry.is_none()).collect::<Vec<_>>().len() > 0 {
        return Err(SyncError(format!("{}: child note was nil", noteheaders.uuid())));
    }

    Ok(LocalNote {
        metadata: NotesMetadata::from_remote_metadata(noteheaders.first().unwrap()),
        body: bodies.into_iter().map(|b|b.unwrap()).collect()
    })
}

///Groups headers that have the same uuid
/// Also sorts the returning vector based of the inner vectors length (ascending)
// TODO check what happens if notes get moved to another folder, do they still have the same uuid?
pub fn collect_mergeable_notes(header_metadata: RemoteNoteHeaderCollection) -> GroupedRemoteNoteHeaders {

    let mut data_grouped: Vec<Vec<RemoteNoteMetaData>> = Vec::new();
    for (_key, group) in &header_metadata.into_iter()
        .sorted_by_key(|entry| entry.headers.uuid())
        .group_by(|header| (header as &RemoteNoteMetaData).headers.uuid()) {
        data_grouped.push(group.collect());
    };
    data_grouped.into_iter().sorted_by_key(|entry| entry.len()).collect()
}

#[cfg(test)]
mod sync_tests {
    use super::*;
    use note::{GroupedRemoteNoteHeaders, RemoteNoteMetaData};
    use builder::{BodyMetadataBuilder, NotesMetadataBuilder};


    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        dotenv::dotenv().ok();
        simple_logger::init_with_level(Level::Debug).unwrap();
    }

    #[test]
    pub fn add_locally_not_merged() {

    }

    #[test]
    pub fn sync_test() {

        let mut imap_session = ::apple_imap::login();
        let db_connection = ::db::establish_connection();

        // ::db::delete_everything(&db_connection);
        sync(&mut imap_session, &db_connection);
    }

    /// Tests if metadata with multiple bodies is getting properly grouped
    #[test]
    fn test_mergable_notes_grouping() {
        use builder::HeaderBuilder;

        let metadata_1 = RemoteNoteMetaData {
            headers:  HeaderBuilder::new().with_subject("Note").build(),
            folder: "test".to_string(),
            uid: 1
        };

        let metadata_2 = RemoteNoteMetaData {
            headers:  metadata_1.headers.clone(),
            folder: "test".to_string(),
            uid: 2
        };

        let metadata_3 = RemoteNoteMetaData {
            headers:  HeaderBuilder::new().with_subject("Another Note").build(),
            folder: "test".to_string(),
            uid: 3
        };

        let mut collected: GroupedRemoteNoteHeaders =
            collect_mergeable_notes(vec![
                metadata_1.clone(),
                metadata_3.clone(),
                metadata_2.clone()]
            );

        //Should be 2, because 2 metadata object should be grouped
        assert_eq!(collected.len(),2);
        let mut collected_list: Vec<Vec<RemoteNoteMetaData>> = vec![];
        for item in collected.drain() {
            collected_list.push(item)
        }
        let sorted_list: Vec<Vec<RemoteNoteMetaData>> =
            collected_list.into_iter().sorted_by_key(|entry| entry.len()).collect();

        let first = &sorted_list.first().unwrap();
        assert_eq!(first.len(),1);
        assert_eq!(first.first().unwrap().headers.uuid(), metadata_3.headers.uuid());

        let second = &sorted_list[1];
        assert_eq!(second.len(),2);
        assert_eq!(second.first().unwrap().headers.uuid(), metadata_1.headers.uuid());
        assert_eq!(second[1].headers.uuid(), metadata_1.headers.uuid());

    }

    /// Should find one item that should be deleted
    #[test]
    fn test_delete_actions() {

        let note_to_be_deleted =
            NotesMetadataBuilder::new()
                .is_flagged_for_deletion(true)
                .build();

        let noteset = set![
        note![
            note_to_be_deleted.clone(),
            BodyMetadataBuilder::new().build()
        ],
        note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
        ]
    ];
        let delete_actions = get_deleted_note_actions(None, &noteset);

        assert_eq!(delete_actions.len(),1);

        match delete_actions.first().unwrap() {
            UpdateAction::DeleteRemote(localnote) => {
                assert_eq!(localnote.metadata.uuid(), note_to_be_deleted.uuid)
            }
            _ => {
                panic!("Wrong Action provided")
            }
        }

    }

    /// Should find zero items because item is flagged but unmerged
    #[test]
    fn test_delete_unmerged_actions() {

        let note_to_be_deleted =
            NotesMetadataBuilder::new()
                .is_flagged_for_deletion(true)
                .build();

        let noteset = set![
        note![
            note_to_be_deleted.clone(),
            BodyMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
        ]
    ];
        let delete_actions = get_deleted_note_actions(None, &noteset);

        assert_eq!(delete_actions.len(),1);

        match delete_actions.first().unwrap() {
            UpdateAction::DoNothing => {
                println!("Success")
            }
            _ => {
                panic!("Wrong Action provided")
            }
        }

    }

    /// Basic add test, there is one new note with a single body on remote side
    #[test]
    fn test_add_actions() {

        let note_to_be_added =
            NotesMetadataBuilder::new().build();

        let remote_only_body= BodyMetadataBuilder::new().build();

        let notes_to_be_added = set![
        note![
            note_to_be_added.clone(),
            remote_only_body.clone()
        ]
    ];

        let local_notes = set![
        note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
        ],
        note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
        ]
    ];

        let remote_data: GroupedRemoteNoteHeaders = notes_to_be_added.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let added_actions = get_added_note_actions(&remote_data, &local_notes);

        assert_eq!(added_actions.len(),1);

        match added_actions.first().unwrap() {
            UpdateAction::AddLocally(header) => {
                assert_eq!(&header.first().unwrap().headers.uuid(), &note_to_be_added.uuid);
                assert_eq!(&header.first().unwrap().uid, &remote_only_body.uid.unwrap());
            }
            _ => {
                panic!("Wrong Action provided")
            }
        }
    }

    /// This add test has a remote note with 2 bodies
    #[test]
    fn test_add_actions_mergeable_note() {
        let first_note = NotesMetadataBuilder::new().build();
        let first_body = BodyMetadataBuilder::new().build();
        let second_body = BodyMetadataBuilder::new().build();

        let notes_to_be_added = set![
        note![
            first_note.clone(),
            first_body.clone(),
            second_body.clone()
        ]
    ];

        let local_notes = set![
        note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
        ],
        note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
        ]
    ];

        let remote_data: GroupedRemoteNoteHeaders = notes_to_be_added.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let added_actions = get_added_note_actions(&remote_data, &local_notes);

        assert_eq!(added_actions.len(), 1);

        match added_actions.first().unwrap() {
            UpdateAction::AddLocally(header) => {
                assert_eq!(&header.uuid(), &first_note.uuid);
                assert_eq!(header.len(), 2_usize);
                //TODO fix
                /*assert_eq!(&uid[0], &second_body.uid.unwrap());
                assert_eq!(&uid[1], &first_body.uid.unwrap());*/
            }
            _ => {
                panic!("Wrong Action provided")
            }
        }
    }


}


    /*pub fn sync(session: &mut Session<TlsStream<TcpStream>>) {
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
            match result {
                Ok(file) => debug!("Sucessfully transferred {}", file),
                Err(error) => warn!("Issue while transferring file: {}", error)
            }
        })

    }*/

/*fn get_update_actions(remote_notes: &Vec<NotesMetadata>) -> Vec<(UpdateAction, NotesMetadata)> {
    //TODO analyze what happens if title changes remotely, implement logic for local title change
    remote_notes.into_iter().map( |mail_headers| {
        let hash_location = profile::get_notes_dir()
            .join(PathBuf::from(&mail_headers.subfolder))
            .join(PathBuf::from(format!("{}{}",".".to_string(), &mail_headers.uid.unwrap().to_string())));

        let glob_string = &(hash_location.to_string_lossy().into_owned() + "_*");
        match glob::glob(glob_string).expect("could not parse glob").next() {
            Some(result) => {
                let hash_loc_path = result.unwrap();
                if hash_loc_path.exists() {
                    let f = File::open(&hash_loc_path).unwrap();
                    let local_metadata: NotesMetadata = serde_json::from_reader(f).unwrap();

                    let local_uuid = local_metadata.message_id.clone();
                    let oldest_remote_uuid = &local_metadata.old_remote_id;

                    let remote_uuid = mail_headers.message_id.clone();

                    if remote_uuid == local_uuid {
                        debug!("Same: {}", mail_headers.subfolder.to_string() + "/" + &mail_headers.subject.clone());
                        return Some((DoNothing, mail_headers.clone()))
                    } else if remote_uuid != local_uuid && oldest_remote_uuid.is_none() {
                        info!("Changed Remotely: {}", mail_headers.subject.clone());
                        return Some((UpdateLocally, mail_headers.clone()))
                    } else if oldest_remote_uuid.is_some() && oldest_remote_uuid.clone().unwrap() == remote_uuid {
                        info!("Changed Locally: {}", &local_metadata.subject.clone());
                        return Some((UpdateRemotely, local_metadata))
                    } else if oldest_remote_uuid.is_some() && remote_uuid != local_uuid {
                        info!("Changed on both ends, needs merge: {}", &mail_headers.subject.clone());
                        return Some((Merge, mail_headers.clone()))
                    } else {
                        warn!("Could not find metadata_file: {}", &hash_loc_path.to_string_lossy())
                    }
                }
            },
            None => return Some((AddLocally, mail_headers.clone()))
        }

        return None
    }).filter_map(|e| {
        if e.is_some() && e.as_ref().unwrap().0 != DoNothing {
            e
        } else {
            None
        }
    }).collect::<Vec<(UpdateAction, NotesMetadata)>>()
}*/

fn update_remotely(local_note: &LocalNote, session: &mut Session<TlsStream<TcpStream>>) -> Result<u32, error::UpdateError> {
    ::apple_imap::update_message(session, local_note).map_err(|e| UpdateError::SyncError(e.to_string()))
}

/*fn update_locally(metadata: &NotesMetadata, session: &mut Session<TlsStream<TcpStream>>) -> Result<String, UpdateError> {
    let note = apple_imap::fetch_single_note(session,metadata).unwrap();
    io::save_note_to_file(&note).map(|_| "".to_string()).map_err(|e| SyncError(e.to_string()))
        .and_then(|_| io::save_metadata_to_file(&metadata).map_err(|e| SyncError(e.to_string())))
}*/

/*fn execute_actions(actions: &Vec<(UpdateAction, &NotesMetadata)>, session:  &mut Session<TlsStream<TcpStream>>) -> Vec<Result<String, UpdateError>> {
     actions.iter().map(|(action, metadata)| {
        match action {
            AddRemotely => {
                info!("{} changed locally, gonna sent updated file to imap server", &metadata.subject_escaped());
                create_mailbox(session, metadata).map_err(|e| SyncError(e.to_string()))
                    .and_then( |_| session.select(&metadata.subfolder).map_err(|e| SyncError(e.to_string()))
                    .and_then(|_| update_remotely(metadata, session)))
            },
            UpdateRemotely => {
                update_remotely(metadata, session)
            },
            DeleteLocally => {
                delete_locally(metadata)
            },
            UpdateAction::UpdateLocally | UpdateAction::AddLocally => {
                update_locally(metadata, session)
            },
            _ => {
                unimplemented!("Action is not implemented")
            }
        }
    }).collect()
}*/

/*fn delete_locally(metadata: &NotesMetadata) -> Result<String, UpdateError> {
    info!("Deleting {} locally", metadata.subject_escaped());
    io::delete_metadata_file(metadata)
        .and_then(|_| io::delete_note(metadata))
        .map(|_| metadata.subject_escaped())
        .map_err(|e| IoError(e.to_string()))
}*/


/*pub fn get_added_deleted_notes<'a>(local_metadata: HashSet<&'a NotesMetadata>,
                                   remote_metadata: HashSet<&'a NotesMetadata>)
    -> Vec<(UpdateAction, &'a NotesMetadata)> {

    info!("Loading local messages");
    let _local_messages = get_local_messages();

    let local_size = local_metadata.len();
    info!("Found {} local notes", local_size);

    let remote_size = remote_metadata.len();
    info!("Found {} remote messages", remote_size);


    let mut only_local: Vec<(UpdateAction,&NotesMetadata)> = local_metadata
        .difference(&remote_metadata)
        .into_iter()
        .map(|e| if e.new {
            (AddRemotely,e.clone())
        } else {
            (DeleteLocally,e.clone())
        })
        .collect();

    let mut only_remote: Vec<(UpdateAction,&NotesMetadata)> = remote_metadata
        .difference(&local_metadata)
        .into_iter()
        .map(|e| (AddLocally,e.to_owned()))
        .collect();

    only_local.append(&mut only_remote);
    only_local

}*/