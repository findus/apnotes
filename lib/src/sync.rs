extern crate log;
extern crate walkdir;
extern crate glob;
extern crate itertools;
#[cfg(test)]
extern crate ctor;

use self::itertools::Itertools;
use self::log::*;
use std::collections::HashSet;
use sync::UpdateAction::{AddLocally, UpdateRemotely, UpdateLocally, AddRemotely, DeleteLocally, DeleteRemote, Merge, DoNothing};
use model::{NotesMetadata, Body};
use error::UpdateError::SyncError;
use error::UpdateError;
use apple_imap::{MailService};
use db::{DBConnector, DatabaseService};
use converter::convert2md;
use notes::localnote::{LocalNote};
use notes::remote_note_metadata::RemoteNoteMetaData;
use notes::remote_note_header_collection::RemoteNoteHeaderCollection;
use notes::grouped_remote_note_headers::GroupedRemoteNoteHeaders;
use notes::traits::identifyable_note::IdentifyableNote;
use notes::traits::header_parser::HeaderParser;
use notes::traits::mergeable_note_body::MergeableNoteBody;
use util::filter_none;
use std::fmt::Display;
use serde::export::Formatter;
use colored::Colorize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub enum UpdateResult {
    Success()
}

/// Defines the Action that has to be done to the
/// message with the corresponding uuid
#[derive(Debug,PartialEq)]
pub enum UpdateAction<'a> {
    /// Deletes the note on the imap server
    DeleteRemote(&'a LocalNote),
    /// Apply to all notes that
    ///     Are note flagged as "new" inside the db
    ///     Are not present anymore on on remote side
    DeleteLocally(&'a LocalNote),
    /// Apply to all notes that:
    ///     have their "locally_edited" flag set
    ///     their "old_remote_id" value equals the remotes message-id
    UpdateRemotely(&'a LocalNote),
    /// Apply to all notes that:
    ///     have their locally_edited flag set to false
    ///     have different message-id on remote side or different count of note_bodies
    /// Action: delete all local bodies and replace with remote content
    UpdateLocally(&'a Vec<RemoteNoteMetaData>),
    /// Apply to all notes that:
    ///     have old_remote id set to non null string
    ///     remotes message-id != the locals message-id
    Merge(MergeMethod, &'a Vec<RemoteNoteMetaData>),
    /// Apply to all notes that:
    ///     have new flag set to true
    ///     their uuid is not present remotely
    AddRemotely(&'a LocalNote),
    /// Apply to all notes that:
    ///     their uuid is not present locally
    AddLocally(&'a RemoteNoteHeaderCollection),
    DoNothing,
}

impl<'a> Display for UpdateAction<'a>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteRemote(_) => write!(f, "DeleteRemote"),
            DeleteLocally(_) => write!(f, "DeleteLocally"),
            UpdateRemotely(_) => write!(f, "UpdateRemotely"),
            UpdateLocally(_) => write!(f, "UpdateLocally"),
            Merge(_,_) => write!(f, "Merge"),
            AddRemotely(_) => write!(f, "AddRemotely"),
            AddLocally(_) => write!(f, "AddLocally"),
            DoNothing => write!(f, "DoNothing")
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum MergeMethod {
    AppendLocally,
}

pub fn sync_notes() -> Result<()> {
    let mut imap_service = ::apple_imap::MailServiceImpl::new_with_login();
    let db_connection= ::db::SqliteDBConnection::new();
    sync(&mut imap_service, &db_connection)
        .map_err(|e| e.into())
        .and_then(|_| imap_service.logout().map_err(|e| e.into()))
}

fn get_sync_actions<'a>(remote_note_headers: &'a GroupedRemoteNoteHeaders,
                        local_notes: &'a HashSet<LocalNote>) -> Vec<UpdateAction<'a>> {
    info!("Found {} local Notes", local_notes.len());
    info!("Found {} remote notes", remote_note_headers.len());

    let all_uuids = get_all_uuids(remote_note_headers, local_notes);

    let mut collection: Vec<(Option<&LocalNote>, Option<&RemoteNoteHeaderCollection>)> = all_uuids.into_iter().map(|uuid| {
        let remote_note = remote_note_headers.iter().filter(|rn| rn.uuid() == uuid).next();
        let local_note = local_notes.iter().filter(|ln| ln.uuid() == uuid).next();
        (local_note, remote_note)
    }).collect();


    let acts: Vec<UpdateAction> = collection.drain(0..).map(|(ln, rn)| {
            get_add_locally_action(rn,ln)
            .or_else(|| get_add_remotely_action(rn,ln))
            .or_else(|| get_update_remotely_action(rn,ln))
            .or_else(|| get_update_locally_action(rn, ln))
            .or_else(|| get_delete_locally_action(rn,ln))
            .or_else(|| get_delete_remotely_action(rn,ln))
            .or_else(|| get_needs_merge_basic(rn, ln))
    })
        .filter_map(|e| { filter_none(e) })
        .collect();

    println!("{:>padding$} Actions pending", acts.len(), padding=4);
    println!("Delete {:>padding$} notes locally",  acts.iter().filter(|act| matches!(act, UpdateAction::DeleteLocally(_))).count(), padding=4);
    println!("Delete {:>padding$} notes remotely", acts.iter().filter(|act| matches!(act, UpdateAction::DeleteRemote(_))).count(), padding=4);
    println!("Add    {:>padding$} notes locally",  acts.iter().filter(|act| matches!(act, UpdateAction::AddLocally(_))).count(), padding=4);
    println!("Add    {:>padding$} notes remotely", acts.iter().filter(|act| matches!(act, UpdateAction::AddRemotely(_))).count(), padding=4);
    println!("Update {:>padding$} notes locally",  acts.iter().filter(|act| matches!(act, UpdateAction::UpdateLocally(_))).count(), padding=4);
    println!("Update {:>padding$} notes remotely", acts.iter().filter(|act| matches!(act, UpdateAction::UpdateRemotely(_))).count(), padding=4);
    println!("Merge  {:>padding$} notes", acts.iter().filter(|act| matches!(act, UpdateAction::Merge(_,_))).count(), padding=4);

    acts

}

fn get_all_uuids(remote_note_headers: &HashSet<Vec<RemoteNoteMetaData>>, local_notes: &HashSet<LocalNote>) -> Vec<String> {
    let remote_uuids: HashSet<String> =
        remote_note_headers.into_iter().map(|item| item.uuid().clone()).collect();

    let local_uuids: HashSet<String> =
        local_notes.into_iter().map(|item| item.uuid().clone()).collect();

    remote_uuids.union(&local_uuids).map(|str| str.clone()).collect()
}

/// Iterates through all provided local notes and checks if the deletion flag got set
/// If this is the case a DeleteRemote Actions gets returned for this note
///
/// If the local note has multiple non-merged bodies the deletion gets skipped
/// TODO: What to do if local note is flagged for deletion but got updated remotely
fn get_delete_remotely_action<'a>(remote_note_headers: Option<&'a RemoteNoteHeaderCollection>,
                                 local_note: Option<&'a LocalNote>) -> Option<UpdateAction<'a>> {
    match (remote_note_headers, local_note) {
        (Some(_rn), Some(ln)) if
            ln.needs_merge() == false &&
            ln.metadata.locally_deleted
        => {
            Some(DeleteRemote(ln))
        }
        _ => None
    }
}


fn get_delete_locally_action<'a>(remote_note_headers: Option<&'a RemoteNoteHeaderCollection>,
                              local_note: Option<&'a LocalNote>) -> Option<UpdateAction<'a>> {
    match (remote_note_headers,local_note) {
        (None,Some(ln)) if ln.metadata.new == false && ln.needs_merge() == false => {
            Some(DeleteLocally(ln))
        },
        _ => None,
    }
}

fn get_add_locally_action<'a>(remote_note_headers: Option<&'a RemoteNoteHeaderCollection>,
                              local_note: Option<&LocalNote>) -> Option<UpdateAction<'a>> {
    match (remote_note_headers,local_note) {
        (Some(remote_note),None) => Some(AddLocally(remote_note)),
        _ => None
    }
}

fn get_add_remotely_action<'a>(remote_note_header: Option<&'a RemoteNoteHeaderCollection>,
                                local_note: Option<&'a LocalNote>) -> Option<UpdateAction<'a>> {
    match (local_note,remote_note_header) {
        (Some(local_note),None) if local_note.metadata.locally_deleted == false &&
            local_note.metadata.new == true => Some(AddRemotely(local_note)),
        _ => None
    }
}

fn get_update_remotely_action<'a>(remote_note_header: Option<&'a RemoteNoteHeaderCollection>,
                                  local_note: Option<&'a LocalNote>) -> Option<UpdateAction<'a>> {

    if let (Some(rn), Some(ln)) = (remote_note_header, local_note) {
        if ln.body[0].old_remote_message_id == rn.get_message_id()
            && rn.get_message_id().is_some()
            && ln.needs_merge() == false
        {

            Some(UpdateRemotely(ln))
        } else {
            // checks if old message_ids == unmerged remote note message ids
            if let (Some(ln_m_ids), rn_m_ids) =  (ln.all_old_message_ids(),rn.all_message_ids()) {
                /*
                Checks if remote note has message id that is not present remotely, if
                this is the case we know that something exists remotely that we do
                not want to override.

                From Local to remote we dont care, if a message id only exists locally
                we expect that this content got merged and we can just override everything.
                */
                if rn_m_ids.difference(&ln_m_ids).count() == 0 {
                    return Some(UpdateRemotely(ln));
                }
                else {
                    return None;
                }
            } else {
                return None;
            }

        }
    } else {
        None
    }
}

fn get_update_locally_action<'a>(remote_note_header: Option<&'a RemoteNoteHeaderCollection>,
                                  local_note: Option<&'a LocalNote>) -> Option<UpdateAction<'a>> {

    match (local_note, remote_note_header) {
        (Some(ln), Some(rn)) if
        ln.metadata.locally_deleted == false
        && ln.content_changed_locally() == false
        => {
            //Check if no merge needs to happen
            if ln.content_changed_locally() == false && ln.changed_remotely(rn){
                return Some(UpdateLocally(rn));
            } else {
                return None
            }
        },
        _ => None
    }
}

/// Checks if a basic merge needs to happen, both notes got changed on both ends
/// but only one note body exist on both ends
fn get_needs_merge_basic<'a>(remote_note_header: Option<&'a RemoteNoteHeaderCollection>,
                             local_note: Option<&'a LocalNote>) -> Option<UpdateAction<'a>> {

    match (local_note, remote_note_header) {
        (Some(ln), Some(rn))
        if ln.metadata.locally_deleted == false
        && ln.needs_merge() == false
        && rn.needs_merge() == false
        && ln.body[0].old_remote_message_id.is_some()
        && ln.body[0].old_remote_message_id.as_ref().unwrap() != &rn.get_message_id().unwrap()
        => {
            Some(Merge(MergeMethod::AppendLocally, rn))
        },
        _ => None
    }
}

pub fn sync<T, C>(imap_session: &mut dyn MailService<T>, db_connection: &dyn DatabaseService<C>)
    -> std::result::Result<(), ::error::UpdateError>
    where C: 'static + DBConnector, T: 'static
{
    let headers = imap_session.fetch_headers().map_err(|e| SyncError(e.to_string()))?;
    let grouped_not_headers = collect_mergeable_notes(headers);
    match db_connection.fetch_all_notes().map_err(|e| SyncError(e.to_string())) {
        Ok(fetches) => {
            let actions =
                get_sync_actions(&grouped_not_headers, &fetches);
            let results = process_actions(imap_session, db_connection, &actions);

            for (action,result) in results {

                let result = match result {
                    Ok(subject) => format!("{} [{}]", "Ok".green(), subject),
                    Err(e) => format!("{} {}", "Failed".red(), e.to_string())
                };

                println!("{:>padding$}...{}", action, result , padding=20 );
            }
            Ok(())
        }
        Err(e) => {
            panic!("mist {}", e);
        }
    }

}

pub fn process_actions<'a, T, C>(
    imap_connection: &mut dyn MailService<T>,
    db_connection: &dyn DatabaseService<C>,
    actions: &'a Vec<UpdateAction<'a>>) -> Vec<(&'a UpdateAction<'a>, Result<String>)>
    where C: 'static + DBConnector, T: 'static
{
    let result = actions
        .iter()
        .map(|action| {
            let result = match action {
                UpdateAction::DeleteRemote(_note) => { unimplemented!(); },
                UpdateAction::DeleteLocally(local_note) => process_delete_locally(db_connection, action, local_note),
                UpdateAction::UpdateLocally(new_note_bodies) => process_update_locally(imap_connection, db_connection, action,new_note_bodies),
                UpdateAction::Merge(_method,remote_note) => { process_merge(imap_connection, db_connection, action, remote_note) },
                UpdateAction::AddRemotely(local_note) | UpdateAction::UpdateRemotely(local_note) => { (action, update_message_remotely(imap_connection, db_connection, &local_note)) }
                UpdateAction::AddLocally(note_headers) => process_add_locally(imap_connection, db_connection, action, note_headers),
                UpdateAction::DoNothing => { (action,Ok("".to_string())) }
            };
            return result;
        }
        ).collect();

    return result;
}

fn process_add_locally<'a,T,C>(imap_connection: &mut dyn MailService<T>,
                               db_connection: &dyn DatabaseService<C>,
                               action: &'a UpdateAction,
                               noteheaders: &RemoteNoteHeaderCollection)
    -> (&'a UpdateAction<'a>, Result<String>)
    where C: 'static + DBConnector, T: 'static {

    let result =
        localnote_from_remote_header(imap_connection, noteheaders)
            .and_then(|note| db_connection.insert_into_db(&note).map_err(|e| e.into()));

    (action, result.map(|_| noteheaders.first_subject() ))
}

fn process_update_locally<'a,T,C>(imap_connection: &mut dyn MailService<T>,
                                   db_connection: &dyn DatabaseService<C>,
                                   action: &'a UpdateAction,
                                   new_note_bodies: &RemoteNoteHeaderCollection)
    -> (&'a UpdateAction<'a>, Result<String>)
    where C: 'static + DBConnector, T: 'static {

    let d: Vec<std::result::Result<Body, UpdateError>> =
        new_note_bodies.iter().map(|e| {
            let folder = &e.folder;
            imap_connection.select(folder)
                .map_err(|e| SyncError(e.to_string()))
                .and_then(|_| imap_connection.fetch_note_content(folder, e.uid)
                    .map_err(|e| UpdateError::SyncError(e.to_string()))
                    .map(|content| (e, content)))
                .and_then(|(headers, content)| {
                    Ok(
                        Body {
                            old_remote_message_id: None,
                            message_id: headers.headers.message_id(),
                            text: Some(convert2md(&content)),
                            uid: Some(headers.uid),
                            metadata_uuid: headers.headers.uuid(),
                        }
                    )
                })
        }).collect();

    if d.iter().filter(|c| c.is_err()).next() != None {
        return (action,Err(UpdateError::SyncError("Could not fetch note bodies".to_string()).into()));
    };

    let f: Vec<Body> = d.into_iter().map(|d| d.unwrap()).collect();

    let result = db_connection.replace_notes(
        &f,
        new_note_bodies.iter().next().unwrap().headers.uuid()
    ).map_err(|e| e.into());

    (action, result.map(|_| new_note_bodies.first_subject()))
}

fn process_delete_remotely<'a, C, T>(imap_connection: &mut dyn MailService<T>,
                                     db_connection: &dyn DatabaseService<C>,
                                     action: &'a UpdateAction,
                                     localnote: &LocalNote)
    -> (&'a UpdateAction<'a>, Result<String>)
    where C: 'static + DBConnector, T: 'static
{
    let result = imap_connection
        .delete_message(localnote)
        .and_then(|subject| db_connection.fetch_single_note(&localnote.metadata.uuid)
            .map_err(|e| e.into())
        )
        .map(|_| localnote.first_subject());

    (action, result)
}

fn process_delete_locally<'a, C>(db_connection: &dyn DatabaseService<C>,
                                 action: &'a UpdateAction,
                                 b: &LocalNote)
    -> (&'a UpdateAction<'a>, Result<String>)
    where C: 'static + DBConnector {
    //TODO what happens if remote umerged note gets deleted only delete this body
    // what happens if to be deleted note with message-id:x has merged un-updated
    //content on local side
    let result = db_connection.delete(b)
        .map_err(|e| e.into());
    (action, result.map(|_| b.first_subject()))
}

fn update_message_remotely<'a, T, C>(imap_connection: &mut dyn MailService<T>,
                                     db_connection: &dyn DatabaseService<C>,
                                     localnote: &LocalNote)
    -> Result<String>
    where C: 'static + DBConnector, T: 'static
{
    info!("{} changed locally, gonna sent updated file to imap server", &localnote.uuid());
    let metadata = &localnote.metadata;
    imap_connection.create_mailbox(metadata)
        .map_err(|e| e.into())
        .and_then(|_| imap_connection.select(&metadata.subfolder)
            .map_err(|e| e.into()))
        .and_then(|_| imap_connection.update_message(localnote)
            .map_err(|e| e.into())
        )
        .and_then(|uid| {
            let body = localnote.body.first().unwrap();
            let note = note!(
                            NotesMetadata {
                                subfolder: localnote.metadata.subfolder.clone(),
                                locally_deleted: localnote.metadata.locally_deleted,
                                locally_edited: localnote.metadata.locally_edited,
                                new: false,
                                date: localnote.metadata.date.clone(),
                                uuid:localnote.metadata.uuid.clone(),
                                mime_version: localnote.metadata.mime_version.clone()
                            },
                            Body {
                                old_remote_message_id: None,
                                message_id: body.message_id.clone(),
                                text: body.text.clone(),
                                uid: Some(uid as i64),
                                metadata_uuid: body.metadata_uuid.clone()
                            }
                        );
            db_connection.update(&note)
                .map_err(|e| e.into())
        })
        .map(|_| localnote.first_subject())
}

fn process_merge<'a,T,C>(imap_connection: &mut dyn MailService<T>,
                                  db_connection: &dyn DatabaseService<C>,
                                  action: &'a UpdateAction,
                                  new_notes: &Vec<RemoteNoteMetaData>)
                                  -> (&'a UpdateAction<'a>, Result<String>)
    where C: 'static + DBConnector, T: 'static {

    match action {
        UpdateAction::Merge(MergeMethod::AppendLocally, _remote_note) => {

            let mut append = || {
                let note_bodies: Vec<Result<Body>> = new_notes.iter().map(|new_note| {

                    println!("Merging Note: {} by appending notebody: {}",
                             new_note.headers.uuid(),
                             new_note.headers.subject()
                    );

                    let text =
                        convert2md(&imap_connection.fetch_note_content(&new_note.folder, new_note.uid)?);

                    Ok(Body {
                        old_remote_message_id: None,
                        message_id: new_note.headers.message_id().clone(),
                        text: Some(text),
                        uid: Some(new_note.uid as i64),
                        metadata_uuid: new_note.headers.uuid(),
                    })
                }).collect();

                for result in note_bodies {
                    match result {
                        Err(e) => {
                            return (action,Err(e));
                        },
                        Ok(result) => {
                            if let Err(e) = db_connection.append_note(&result).map_err(|e| e.into()) {
                                return (action,Err(e));
                            }
                        }
                    }

                };

                (action,Ok(new_notes.first_subject()))

            };

            return append();
        },
        _ => { panic!("Unimplemented") }
    }
}

fn localnote_from_remote_header<T>(imap_connection: &mut dyn MailService<T>, noteheaders: &Vec<RemoteNoteMetaData>)
    -> Result<LocalNote>
    where T: 'static
{
    let bodies: Vec<Option<Body>> = noteheaders.into_iter().map(|single_remote_note| {
        (
            single_remote_note,
            imap_connection.fetch_note_content(
                &single_remote_note.folder,
                single_remote_note.uid,
            )
        )
    }).map(|(remote_metadata, result)| {
        match result {
            Ok(body) => {
                Some(Body {
                    old_remote_message_id: None,
                    message_id: remote_metadata.headers.message_id(),
                    text: Some(convert2md(&body)),
                    uid: Some(remote_metadata.uid),
                    metadata_uuid: remote_metadata.headers.uuid(),
                })
            }
            Err(e) => {
                warn!("Could not receive message body: {}", e);
                None
            }
        }
    }).collect();

    if bodies.iter().filter(|entry| entry.is_none()).collect::<Vec<_>>().len() > 0 {
        return Err(SyncError(format!("{}: child note was nil", noteheaders.uuid())).into());
    }

    Ok(LocalNote {
        metadata: NotesMetadata::from_remote_metadata(noteheaders.first().unwrap()),
        body: bodies.into_iter().map(|b| b.unwrap()).collect(),
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
    use builder::{NotesMetadataBuilder, BodyMetadataBuilder};


    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        dotenv::dotenv().ok();
        simple_logger::init_with_level(Level::Debug).unwrap();
    }

    /// tests : Standard, note with 2 bodies, check if parent note gets deleted if only 1 body
    #[test]
    pub fn delete_local_body_test() {}

    /// tests if locally updated note gets properly detected
    #[test]
    pub fn update_remotely_test() {

        let remote_note = note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("1").build()
        ];

        let remote_header = set![vec![remote_note.to_remote_metadata()]];

        let local_note = note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("2").with_old_remote_message_id("1").build()
        ];

        let noteset = set![
            local_note
        ];
        let update = get_sync_actions(&remote_header, &noteset);

        assert_eq!(update.len(), 1);

        match update.first().unwrap() {
            UpdateAction::UpdateRemotely(localnote) => {
                assert_eq!(localnote.metadata.uuid(), "1".to_string());
                assert_eq!(localnote.body[0].message_id, "2".to_string());

            }
            _ => {
                panic!("Wrong Action provided")
            }
        }
    }

    /// tests if locally and remotely changed notes are getting detected and flagged as
    /// merge action
    #[test]
    pub fn update_remotely_test_also_changed_remotely() {

        let remote_note = note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("3").build()
        ];

        let remote_header = set![vec![remote_note.to_remote_metadata()]];

        let local_note = note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_old_remote_message_id("1").with_message_id("2").build()
        ];

        let noteset = set![
            local_note
        ];

        let update = get_sync_actions(&remote_header, &noteset);

        assert_eq!(update.len(), 1);
        assert!(matches!(update[0], UpdateAction::Merge(MergeMethod::AppendLocally,_)));
    }

    /// Tests if metadata with multiple bodies is getting properly grouped
    #[test]
    fn test_mergable_notes_grouping() {
        use builder::HeaderBuilder;

        let metadata_1 = RemoteNoteMetaData {
            headers: HeaderBuilder::new().with_subject("Note").build(),
            folder: "test".to_string(),
            uid: 1,
        };

        let metadata_2 = RemoteNoteMetaData {
            headers: metadata_1.headers.clone(),
            folder: "test".to_string(),
            uid: 2,
        };

        let metadata_3 = RemoteNoteMetaData {
            headers: HeaderBuilder::new().with_subject("Another Note").build(),
            folder: "test".to_string(),
            uid: 3,
        };

        let mut collected: GroupedRemoteNoteHeaders =
            collect_mergeable_notes(vec![
                metadata_1.clone(),
                metadata_3.clone(),
                metadata_2.clone()]
            );

        //Should be 2, because 2 metadata object should be grouped
        assert_eq!(collected.len(), 2);
        let mut collected_list: Vec<Vec<RemoteNoteMetaData>> = vec![];
        for item in collected.drain() {
            collected_list.push(item)
        }
        let sorted_list: Vec<Vec<RemoteNoteMetaData>> =
            collected_list.into_iter().sorted_by_key(|entry| entry.len()).collect();

        let first = &sorted_list.first().unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first.first().unwrap().headers.uuid(), metadata_3.headers.uuid());

        let second = &sorted_list[1];
        assert_eq!(second.len(), 2);
        assert_eq!(second.first().unwrap().headers.uuid(), metadata_1.headers.uuid());
        assert_eq!(second[1].headers.uuid(), metadata_1.headers.uuid());
    }

    /// Should find one item that should be deleted
    #[test]
    fn test_delete_remote_actions() {

        let noteset = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").is_flagged_for_deletion(true).build(),
                BodyMetadataBuilder::new().build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let delete_actions = get_sync_actions(&remote_data, &noteset);

        assert_eq!(delete_actions.len(), 1);

        match delete_actions.first().unwrap() {
            UpdateAction::DeleteRemote(localnote) => {
                assert_eq!(localnote.metadata.uuid(), "1".to_string())
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

        let nothing = GroupedRemoteNoteHeaders::new();
        let delete_actions = get_sync_actions(&nothing, &noteset);

        assert_eq!(delete_actions.len(), 0);


    }

    /// Note got updated remotely, that note should not appear as add-action
    #[test]
    fn test_add_action_remotely_changed() {
        let note_metadata = NotesMetadataBuilder::new().build();

        let local_note = note![
                note_metadata.clone(),
                BodyMetadataBuilder::new().build()
        ];

        let changed_remote_note = note![
                note_metadata.clone(),
                BodyMetadataBuilder::new().build()
        ];

        let local_notes = set![local_note];

        let remote_data: GroupedRemoteNoteHeaders = set![RemoteNoteMetaData::new(&changed_remote_note)];

        let added_actions = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(added_actions.len(), 1);
        assert!(matches!(added_actions[0],UpdateLocally(_)));
    }

    /// Basic add test, there is one new note with a single body on remote side
    #[test]
    fn test_add_actions() {
        let note_to_be_added =
            NotesMetadataBuilder::new().build();

        let remote_only_body = BodyMetadataBuilder::new().with_uid(Some(1)).build();

        let remote_notes = set![
        note![
            note_to_be_added.clone(),
            remote_only_body.clone()
        ],
         note![
            NotesMetadataBuilder::new().with_uuid("2").build(),
            BodyMetadataBuilder::new().with_message_id("21").build()
        ],
        note![
            NotesMetadataBuilder::new().with_uuid("3").build(),
            BodyMetadataBuilder::new().with_message_id("31").build()
        ]
    ];

        let local_notes = set![
        note![
            NotesMetadataBuilder::new().with_uuid("2").build(),
            BodyMetadataBuilder::new().with_message_id("21").build()
        ],
        note![
            NotesMetadataBuilder::new().with_uuid("3").build(),
            BodyMetadataBuilder::new().with_message_id("31").build()
        ]
    ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let added_actions = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(added_actions.len(), 1);

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
        ],
        note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("11").build()
        ],
        note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("21").build()
        ]
    ];

        let local_notes = set![
        note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("11").build()
        ],
        note![
            NotesMetadataBuilder::new().with_uuid("1").build(),
            BodyMetadataBuilder::new().with_message_id("21").build()
        ]
    ];

        let remote_data: GroupedRemoteNoteHeaders = notes_to_be_added.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let added_actions = get_sync_actions(&remote_data, &local_notes);

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

    /**
    This should be handled by the updateLocally action generator
    because note only gets altered, not deleted entirely
    **/
    #[test]
    pub fn delete_locally_2_bodies() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::UpdateLocally(_)));
    }

    /// whole note should be deleted because it does not exist remotely anymore
    #[test]
    pub fn delete_locally() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build()
            ]
        ];

        let remote = GroupedRemoteNoteHeaders::new();

        let action = &get_sync_actions(&remote, &local_notes);

        match action.iter().next() {
            Some(UpdateAction::DeleteLocally(actions)) => {
                assert_eq!(actions.metadata.uuid, "1");
                assert_eq!(actions.body[0].message_id, "1");
            }
            _ => panic!("wrong action")
        }
    }

    //Check: 2 "normal" notes
    #[test]
    pub fn update_locally() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);

        let first_action = &action[0];

        match first_action {
            UpdateAction::UpdateLocally(headers) => {
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].headers.uuid(), "1");
                assert_eq!(headers[0].headers.message_id(), "2");
            }
            _ => panic!("wrong action")
        }
    }

    // unmerged local, one remote, should update because nothing changed remotely
    #[test]
    pub fn update_locally_unmerged_local() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);

        let first_action = &action[0];

        match first_action {
            UpdateAction::UpdateLocally(headers) => {
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].headers.uuid(), "1");
                assert_eq!(headers[0].headers.message_id(), "2");
            }
            _ => panic!("wrong action")
        }
    }

    // merged local, changed remote, should return nothing, this case needs
    // to be handled in merge checker
    #[test]
    pub fn update_locally_merged_local_changed_remote() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("3").with_old_remote_message_id("1,2").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("5").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::Merge(MergeMethod::AppendLocally,_)));

    }

    // merged local, unchanged remote (multi old message-id(), should do nothing because
    // update_remotly should take care of it
    #[test]
    pub fn update_locally_merged_local_unchanged_remote() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("3").with_old_remote_message_id("1,2").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateRemotely(_) ));


    }

    #[test]
    pub fn update_locally_unmerged_local_partly_changed_remote() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("3").build(),
                BodyMetadataBuilder::new().with_message_id("4").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("12").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::UpdateLocally(_)));

    }

    // Message got changed remotely and locally
    #[test]
    pub fn basic_merge_test() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_old_remote_message_id("1").with_message_id("2").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("5").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::Merge(MergeMethod::AppendLocally,_)));

    }

    // Should return update locally
    #[test]
    pub fn merge_test_remote_2_bodies_local_unchaged() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("4").build(),
                BodyMetadataBuilder::new().with_message_id("5").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::UpdateLocally(_)));

    }

    // Both remote notes should get deleted and a new one should get added with new content
    #[test]
    pub fn update_remotely_merged() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("6").with_old_remote_message_id("4,5").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("4").build(),
                BodyMetadataBuilder::new().with_message_id("5").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::UpdateRemotely(_)));

    }

    // Local note changed and remote note changed on another device
    #[test]
    pub fn update_remotely_merged_one_remote_one_local() {
        let local_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("6").with_old_remote_message_id("4,5").build()
            ]
        ];

        let remote_notes = set![
            note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("4").build()
            ]
        ];

        let remote_data: GroupedRemoteNoteHeaders = remote_notes.iter().map(|entry| {
            RemoteNoteMetaData::new(entry)
        }).collect();

        let action = get_sync_actions(&remote_data, &local_notes);

        assert_eq!(action.len(), 1);
        assert!(matches!(action[0], UpdateAction::UpdateRemotely(_)));

    }


}