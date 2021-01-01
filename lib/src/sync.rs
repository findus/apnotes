extern crate log;
extern crate walkdir;
extern crate glob;
extern crate itertools;

use self::itertools::Itertools;
use note::{NoteHeader, HeaderParser};


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

///Groups headers that have the same uuid
/// Also sorts the returning vector based of the inner vectors length (ascending)
pub fn collect_mergable_notes(header_metadata: Vec<NoteHeader>) -> Vec<Vec<NoteHeader>> {

    let mut data_grouped: Vec<Vec<NoteHeader>> = Vec::new();
    for (_key, group) in &header_metadata.into_iter()
        .sorted_by_key(|entry| entry.identifier())
        .group_by(|header| (header as &NoteHeader).identifier()) {
        data_grouped.push(group.collect());
    };
    data_grouped.into_iter().sorted_by_key(|entry| entry.len()).collect()
}

#[test]
fn test_mergable_notes_grouping() {
    use util::HeaderBuilder;
    let metadata_1 = HeaderBuilder::new().with_subject("Note".to_string()).build();
    let metadata_2 = metadata_1.clone();
    let metadata_3 = HeaderBuilder::new().with_subject("Another Note".to_string()).build();

    let collected: Vec<Vec<NoteHeader>> =
        collect_mergable_notes(vec![
            metadata_1.clone(),
            metadata_3.clone(),
            metadata_2.clone()]
        );

    //Should be 2, because 2 metadata object should be grouped
    assert_eq!(collected.len(),2);

    let first = &collected.first().unwrap();
    assert_eq!(first.len(),1);
    assert_eq!(first.first().unwrap().identifier(),metadata_3.identifier());

    let second = &collected[1];
    assert_eq!(second.len(),2);
    assert_eq!(second.first().unwrap().identifier(),metadata_1.identifier());
    assert_eq!(second[1].identifier(),metadata_1.identifier());

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

/*fn update_remotely(metadata: &NotesMetadata, session: &mut Session<TlsStream<TcpStream>>) -> Result<String, UpdateError> {
    match apple_imap::update_message(session, metadata) {
        Ok(new_uid) => {
            let new_metadata = NotesMetadata {
                old_remote_id: None,
                subfolder: metadata.subfolder.clone(),
                locally_deleted: metadata.locally_deleted,
                uid: Some(new_uid as i64),
                new: false,
                date: metadata.date.clone(),
                uuid: metadata.uuid.clone(),
                message_id: metadata.message_id.clone(),
                mime_version: metadata.mime_version.clone(),
                subject: metadata.subject.clone()
            };

            io::save_metadata_to_file(&new_metadata)
                .map_err(|e| std::io::Error::from(e))
                .and_then(|_| io::move_note(&new_metadata, &metadata.subject_with_identifier()))
                .and_then(|_| io::delete_metadata_file(&metadata))
                .map(|_| metadata.subject_escaped())
                .map_err(|e| SyncError(e.to_string()))
        },
        Err(e) => {
            error!("Error while updating note {} {}", metadata.subject.clone(), e.to_string());
            Err(SyncError(e.to_string()))
        }
    }
}*/

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