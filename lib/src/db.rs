extern crate log;
#[cfg(test)]
extern crate mockall;

/**
Windows: Register sqlite dll with "lib /MACHINE:X64 /def:sqlite3.def /out:sqlite3.lib" on x64
Set: SQLITE3_LIB_DIR
**/

use crate::{schema};

use diesel::{SqliteConnection, Connection};
use diesel::*;
use diesel::result::Error;
use crate::model::{NotesMetadata, Body};
use crate::schema::metadata::dsl::metadata;
use crate::schema::body::dsl::body;
use self::log::*;
use crate::schema::body::columns::metadata_uuid;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use crate::schema::metadata::columns::subfolder;
use crate::notes::localnote::LocalNote;

embed_migrations!("../migrations/");

pub trait DatabaseService {
    /// Deletes everything
    fn delete_everything(&self) -> Result<(), Error>;
    /// Appends a note to an already present note
    ///
    /// Multiple notes only occur if you altered a note locally
    /// and server-side, or if 2 separate devices edited the
    /// same note, in that case 2 notes exists on the imap
    /// server.
    ///
    /// If multiple notes exists tell user that a merge needs to happen
    ///
    fn append_note(&self, model: &crate::model::Body) -> Result<(), Error>;
    /// In case of a successful merge this method replaces all unmerged notes with a single
    /// merged note
    fn update_merged_note(&self, note_body: &Body) -> Result<(), Error>;
    /// Deletes the passed local_note with all note_bodies
    fn delete(&self, local_note: &LocalNote) -> Result<(), Error>;
    /// Deletes a single note_body
    fn delete_note_body(&self, note_body: &Body) -> Result<(), Error>;
    /// Delete multiple note_bodies
    fn delete_note_bodies<'a>(&self, note_bodies: &Vec<&'a Body>) -> Result<(), Error>;
    /// Updates the passed local_note with the new content
    ///
    /// Overrides everything
    fn update(&self, local_note: &LocalNote) -> Result<(), Error>;
    /// Inserts the passed local_note
    fn insert_into_db(&self,note: &LocalNote) -> Result<(), Error>;
    /// Returns all local_notes that are currently inside the database, including
    /// the note_bodies
    fn fetch_all_notes(&self) -> Result<HashSet<LocalNote>,Error>;
    /// Returns a single note with a specified subject-name. If multiple
    /// notes with the same subject exist only the first one gets returned.
    fn fetch_single_note_with_name(&self, name: &str) -> Result<Option<LocalNote>, Error>;
    /// Returns a single note with the specified uuid
    fn fetch_single_note(&self, uuid: &str) -> Result<Option<LocalNote>, Error>;
    /// Checks if the Note-Metadata Entry with the specified id
    /// does have note-bodies.
    fn is_widow(&self, metadata_unique_id: &str) -> Result<bool, Error>;
    /// Deletes a single metadata object, needed to delete widow_metadata_entries
    fn delete_metadata(&self, uuid: &str) -> Result<(), Error>;
    fn replace_notes(&self, notes: &Vec<Body>, uuid: String) -> Result<(), Error>;
}

struct SqLiteConnector {

}

impl SqLiteConnector {
    fn connect() -> SqliteConnection {
        let database_url = crate::profile::get_db_path().into_os_string().to_string_lossy().to_string();

        #[cfg(debug)]
        info!("Database Path: {}", database_url);

        let connection = SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let _ = &connection.execute("PRAGMA foreign_keys = ON").unwrap();

        let _ = embedded_migrations::run_with_output(&connection, &mut std::io::stdout());
        connection
    }
}

pub struct SqliteDBConnection {
    connection: ::diesel::sqlite::SqliteConnection
}

impl SqliteDBConnection {
    pub fn new() -> SqliteDBConnection {
        SqliteDBConnection {
            connection: SqLiteConnector::connect()
        }
    }

    pub fn connection(&self) -> &::diesel::sqlite::SqliteConnection {
        &self.connection
    }
}


impl DatabaseService for SqliteDBConnection {
    fn delete_everything(&self) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {

            diesel::delete(schema::body::dsl::body)
                .execute(&self.connection)?;

            diesel::delete(schema::metadata::dsl::metadata)
                .execute(&self.connection)?;


            Ok(())
        })
    }

    fn append_note(&self, model: &Body) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {
            diesel::insert_into(schema::body::table)
                .values(model)
                .execute(&self.connection)?;

            Ok(())
        })
    }

    fn update_merged_note(&self, note_body: &Body) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {
            diesel::delete(schema::body::dsl::body
                .filter(metadata_uuid.eq(note_body.metadata_uuid.clone()))
            )
                .execute(&self.connection)?;
            self.append_note(note_body)?;
            Ok(())
        })
    }

    fn delete(&self, local_note: &LocalNote) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {

            diesel::delete(schema::body::dsl::body)
                .filter(schema::body::dsl::metadata_uuid.eq(&local_note.metadata.uuid))
                .execute(&self.connection)?;

            diesel::delete(schema::metadata::dsl::metadata)
                .filter(schema::metadata::dsl::uuid.eq(&local_note.metadata.uuid))
                .execute(&self.connection)?;

            Ok(())
        })
    }

    fn delete_note_body(&self, note_body: &Body) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {

            diesel::delete(schema::body::dsl::body)
                .filter(schema::body::dsl::message_id.eq(&note_body.message_id))
                .execute(&self.connection)?;

            let uuid = note_body.metadata_uuid.clone();

            // if parent localnote object has no childs any more delete it
            if self.is_widow(&uuid)? {
                self.delete_metadata(&uuid)?;
            }

            Ok(())
        })
    }

    fn delete_note_bodies<'a>(&self, note_bodies: &Vec<&'a Body>) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {

            for b in note_bodies {
                self.delete_note_body(b)?;
            }

            Ok(())
        })
    }

    fn update(&self, local_note: &LocalNote) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {
            //TODO replace with upsert with diesel 2.0
            self.delete( local_note)?;
            self.insert_into_db(local_note)?;
            Ok(())
        })
    }

    fn insert_into_db(&self, note: &LocalNote) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {
            diesel::insert_into(schema::metadata::table)
                .values(&note.metadata)
                .execute(&self.connection)?;

            for note_content in &note.body {
                diesel::insert_into(schema::body::table)
                    .values(note_content)
                    .execute(&self.connection)?;
            }

            Ok(())
        })
    }

    fn fetch_all_notes(&self) -> Result<HashSet<LocalNote, RandomState>, Error> {
        let notes: Vec<NotesMetadata> = metadata
            .order(subfolder.asc())
            .load::<NotesMetadata>(&self.connection)?;

        let note_bodies: Vec<Body> = crate::model::Body::belonging_to(&notes)
            .load::<Body>(&self.connection)?;

        let grouped = note_bodies.grouped_by(&notes);

        let d = notes
            .into_iter()
            .zip(grouped)
            .map(|(m_data,bodies)| {
            LocalNote {
                metadata: m_data,
                body: bodies
            }
        }).collect();

        Ok(d)
    }

    fn fetch_single_note_with_name(&self, name: &str) -> Result<Option<LocalNote>, Error> {
        let note_bodies: Vec<Body> = body
            .filter(schema::body::dsl::text.like(&format!("{}%",name)))
            .limit(1)
            .load::<Body>(&self.connection)?;

        if note_bodies.len() == 0 {
            return Ok(None)
        }

        let m_data: Vec<NotesMetadata> = metadata
            .filter(schema::metadata::dsl::uuid
                .eq(&note_bodies.first().unwrap().metadata_uuid)
            )
            .limit(1)
            .load::<NotesMetadata>(&self.connection)?;

        let first_metadata = m_data.first()
            .expect("Expected at least one metadata object");

        // Refetch note in case note has umerged notes with other subjects
        let note = self.fetch_single_note(&first_metadata.uuid)?;

        debug!("Fetched note with uuid {}", first_metadata.uuid);

        Ok(note)
    }

    fn fetch_single_note(&self, id: &str) -> Result<Option<LocalNote>, Error> {
        let mut notes: Vec<NotesMetadata> = metadata
            .filter(schema::metadata::dsl::uuid.eq(&id))
            .load::<NotesMetadata>(&self.connection)?;

        assert!(notes.len() <= 1);

        if notes.len() == 0 {
            return Ok(None)
        }

        let first_note = notes.remove(0);

        debug!("Fetched note with uuid {}", first_note.uuid.clone());

        let note_body = crate::model::Body::belonging_to(&first_note)
            .load::<Body>(&self.connection)?;

        debug!("This note has {} subnotes ", note_body.len());

        Ok(Some(LocalNote {
            metadata: first_note,
            body: note_body
        }))
    }

    fn is_widow(&self, metadata_unique_id: &str) -> Result<bool, Error> {
        let first_note = metadata
            .filter(schema::metadata::dsl::uuid.eq(&metadata_unique_id))
            .load::<NotesMetadata>(&self.connection)?;

        let first_note = first_note.first();

        if let Some(first_note) = first_note {
            let d = crate::model::Body::belonging_to(first_note)
                .load::<Body>(&self.connection)?.len() == 0;
            return Ok(d);
        } else {
            return Ok(false);
        }
    }

    fn delete_metadata(&self, uuid: &str) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {

            diesel::delete(schema::metadata::dsl::metadata)
                .filter(schema::metadata::dsl::uuid.eq(uuid))
                .execute(&self.connection)?;

            Ok(())
        })
    }

    fn replace_notes(&self, notes: &Vec<Body>, uuid: String) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {

            diesel::delete(schema::body::dsl::body)
                .filter(schema::body::dsl::metadata_uuid.eq(uuid))
                .execute(&self.connection)?;

            for note in notes {
                self.append_note(note)?;
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod db_tests {
    use crate::model::NotesMetadata;
    use crate::builder::*;
    use crate::model::Body;
    use super::*;
    use crate::notes::traits::identifyable_note::IdentifiableNote;

    /// Should return an error, because this note still has child note_bodies
    #[test]
    fn delete_fake_widow_note() {

        let note_body = BodyMetadataBuilder::new()
            .with_text("meem\nTestTestTest").build();

        let note = note![
            NotesMetadataBuilder::new().build(),
            note_body.clone()
        ];

        let db_connection = ::db::SqliteDBConnection::new();

        db_connection.delete_everything().unwrap();
        db_connection.insert_into_db(&note).unwrap();

        match db_connection.delete_metadata(&note.uuid()) {
            Ok(_) => { panic!("Should fail") }
            _ => {}
        }


    }

    //complete note should be gone because of only one body child
    #[test]
    fn delete_body_test_one_child() {

        let note_body = BodyMetadataBuilder::new()
            .with_text("meem\nTestTestTest").build();

        let note = note![
            NotesMetadataBuilder::new().build(),
            note_body.clone()
        ];

        let note_body = note.body[0].clone();

        let db_connection = ::db::SqliteDBConnection::new();

        db_connection.delete_everything().unwrap();
        db_connection.insert_into_db(&note).unwrap();

        match db_connection.delete_note_body(&note_body) {
            Ok(()) => {
                let new_note = db_connection
                    .fetch_single_note(&note.metadata.uuid).unwrap();

                assert_eq!(new_note,None);
            }
            _ => { panic!("Failed") }
        }
    }

    /// Metadataobject should remain because of second body
    #[test]
    fn delete_body_test_two_child() {

        let note_body = BodyMetadataBuilder::new()
            .with_text("meem\nTestTestTest").build();

        let note = note![
            NotesMetadataBuilder::new().build(),
            note_body.clone(),
            BodyMetadataBuilder::new().with_text("meem\nTestTestTest").build()
        ];

        let note_body = note.body[0].clone();

        let db_connection = ::db::SqliteDBConnection::new();

        db_connection.delete_everything().unwrap();
        db_connection.insert_into_db(&note).unwrap();

        match db_connection.delete_note_body(&note_body) {
            Ok(()) => {
                let note_len = db_connection.fetch_all_notes().unwrap().len() == 1;

                assert_eq!(note_len,true);
            }
            _ => { panic!("Failed") }
        }
    }

    // #[test]
    // pub fn mock_test() {
    //     let mut mock_db_service = MockDatabaseService::<SqliteDBConnection>::new();
    //
    //     mock_db_service.expect_fetch_all_notes().returning(|| Err(diesel::result::Error::NotFound));
    //
    //     let mut mock_imap_service: ::apple_imap::MockMailService<Session<TlsStream<TcpStream>>> =
    //         ::apple_imap::MockMailService::<_>::new();
    //
    //     mock_imap_service.expect_fetch_headers().returning(|| Err(imap::error::Error::Append) );
    //
    //
    //     let _err = ::sync::sync(
    //         &mut mock_imap_service,
    //         &mock_db_service)
    //         .err();
    //
    //     //assert_eq!(err,Some(::error::UpdateError::SyncError("oops".to_string())))
    //
    // }

    #[test]
    pub fn note_by_subject() {
        let note = note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().with_text("meem\nTestTestTest").build()
        ];

        let db_connection = ::db::SqliteDBConnection::new();

        db_connection.delete_everything().unwrap();
        db_connection.insert_into_db(&note).unwrap();

        match db_connection.fetch_single_note_with_name("meem") {
            Ok(Some(note)) => {
                assert_eq!(note.body.len(),1);
                assert_eq!(
                    note.body.first().expect("expected note body").text,
                    Some("meem\nTestTestTest".to_string())
                )
            }
            _ => { panic!("Failed") }
        }
    }

    /// Checks if all transactions are getting reverted if one fails
    #[test]
    pub fn nested_transaction() {
        let note = note![
            NotesMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().with_message_id("1").build(),
            BodyMetadataBuilder::new().with_message_id("1").build(),
            BodyMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build(),
            BodyMetadataBuilder::new().build()
    ];

        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().unwrap();

        assert_eq!(con.insert_into_db(&note).is_err(), true);

        let a = con.fetch_all_notes().unwrap();
        assert_eq!(a.len(),0);
    }


    /// Checks if all notes are getting fetched properly
    #[test]
    pub fn fetch_all_test() {

        let body1 = BodyMetadataBuilder::new().build();
        let body2 = BodyMetadataBuilder::new().build();
        let body3 = BodyMetadataBuilder::new().build();

        let note = note![
            NotesMetadataBuilder::new().build(),
            body1
    ];

        let note_with_2_bodies = note![
            NotesMetadataBuilder::new().build(),
            body2,
            body3
    ];

        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete the db");
        con.insert_into_db(&note).expect("Should insert note into the db");
        con.insert_into_db(&note_with_2_bodies).expect("Should insert note into the db");

        match con.fetch_all_notes() {
            Ok(notes) => {
                let body_in_first_note = note.body.first().unwrap();

                // Check if the first note with only one body has
                // correct body assigned to metadata object
                let first_note: Vec<&LocalNote> = notes.iter()
                    .filter(|e| e.metadata.uuid == body_in_first_note.metadata_uuid)
                    .collect();

                assert_eq!(first_note.len(),1);
                let first: &LocalNote = first_note.first().unwrap();
                assert_eq!(first.body.len(), 1);
                assert_eq!(&first.body[0], note.body.first().unwrap());

                // Check the same with the object that has 2 bodies
                let second_bodies = &note_with_2_bodies.body;
                assert_eq!(second_bodies.len(),2);

                let first_body = &note_with_2_bodies.body[0];
                let second_body =  &note_with_2_bodies.body[1];

                let first_note: Vec<&LocalNote> = notes
                    .iter()
                    .filter(|e| e.metadata.uuid == first_body.metadata_uuid)
                    .collect();
                assert!(first_body.is_inside_localnote(first_note.first().unwrap()));

                let second_note: Vec<&LocalNote> = notes
                    .iter()
                    .filter(|e| e.metadata.uuid == second_body.metadata_uuid)
                    .collect();
                assert!(first_body.is_inside_localnote(second_note.first().unwrap()));

                //Negative test, this note should not be present

                let third_note: Vec<&LocalNote> = notes
                    .iter()
                    .filter(|e| e.metadata.uuid == second_body.metadata_uuid)
                    .collect();
                assert_eq!(third_note.len(),1);
                assert_eq!(
                    body_in_first_note.is_inside_localnote(third_note.first().unwrap()),
                    false
                );

            },
            _ => panic!("could not fetch notes")
        }


    }

    #[test]
    fn update_single_note() {
        use crate::builder::HeaderBuilder;
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete the db");

        let m_data: ::model::NotesMetadata =
            NotesMetadata::new(
                &HeaderBuilder::new().build(),
                "test".to_string()
        );

        let note_body = Body::new(Some(0), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body
    );

        let note_2 = note!(
        NotesMetadataBuilder::new().with_uuid("meem").build(),
        BodyMetadataBuilder::new().with_text("old text").build()
    );

        let note_3 = note!(
        NotesMetadataBuilder::new().with_uuid("meem").build(),
        BodyMetadataBuilder::new().with_text("new text").build()
    );

        con.insert_into_db(&note).expect("Should insert note into the db");
        con.insert_into_db(&note_2).expect("Should insert note into the db");

        let item_count = con.fetch_all_notes()
            .expect("Fetch should be successful")
            .len();

        assert_eq!(item_count,2);

        con.update(&note_3).unwrap();

        let item_count = con.fetch_all_notes()
            .expect("Fetch should be successful")
            .len();

        assert_eq!(item_count,2);

        match con.fetch_single_note(&note_3.uuid().clone()) {
            Ok(Some(mut note2)) => {
                assert_eq!(note_3.metadata,note2.metadata);
                assert_eq!(note2.body.len(),1);

                let first_note = note2.body.pop().unwrap();
                assert_eq!(&first_note,note_3.body.first().unwrap());
                assert_eq!(&first_note.text.expect("text"),"new text");

            },
            Ok(None) => panic!("No note found"),
            Err(e) => panic!("Fetch DB Call failed {}", e.to_string())
        }
    }

    /// The correct note should remain in side the db
    #[test]
    fn delete_single_note() {
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete the db");

        let m_data: ::model::NotesMetadata =
            NotesMetadata::new(&::builder::HeaderBuilder::new().build(),
                               "test".to_string()
        );

        let note_body = Body::new(Some(0), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body
    );

        let note_2 = note!(
        NotesMetadataBuilder::new().build(),
        BodyMetadataBuilder::new().build()
    );

        con.insert_into_db(&note).expect("Should insert note into the db");
        con.insert_into_db(&note_2).expect("Should insert note into the db");

        let item_count = con.fetch_all_notes()
            .expect("Fetch should be successful")
            .len();

        assert_eq!(item_count,2);

        con.delete( &note_2).unwrap();

        let item_count = con.fetch_all_notes()
            .expect("Fetch should be successful")
            .len();

        assert_eq!(item_count,1);

        match con.fetch_single_note(&note.uuid().clone()) {
            Ok(Some(mut note2)) => {
                assert_eq!(note.metadata,note2.metadata);
                assert_eq!(note2.body.len(),1);

                let first_note = note2.body.pop().unwrap();
                assert_eq!(&first_note,note.body.first().unwrap());

            },
            Ok(None) => panic!("No note found"),
            Err(e) => panic!("Fetch DB Call failed {}", e.to_string())
        }
    }

    /// Should insert a single metadata object with a body
    ///
    /// This test should return this note correctly after it got
    /// saved.
    #[test]
    fn insert_single_note() {
        use crate::builder::HeaderBuilder;
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete the db");
        let m_data: ::model::NotesMetadata =
            NotesMetadata::new(
                &HeaderBuilder::new().build(),
                "test".to_string()
            );

        let note_body = Body::new(Some(0), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body
    );

        con.insert_into_db(&note).expect("Should insert note into the db");

        match con.fetch_single_note(&note.uuid().clone()) {
            Ok(Some(mut note2)) => {
                assert_eq!(note2.metadata,note.metadata);
                assert_eq!(note2.body.len(),1);

                let first_note = note2.body.pop().unwrap();
                assert_eq!(&first_note,note.body.first().unwrap());

            },
            Ok(None) => panic!("No note found"),
            Err(e) => panic!("Fetch DB Call failed {}", e.to_string())
        }
    }

    /// Should crash because it inserts multiple notes with the same
    /// uuid
    #[test]
    fn no_duplicate_entries() {
        use crate::builder::HeaderBuilder;
        //Setup
        dotenv::dotenv().ok();
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");

        let m_data: ::model::NotesMetadata =
            NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());

        let note_body = Body::new(Some(0), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body
    );

        match con.insert_into_db(&note)
            .and_then(|_| con.insert_into_db(&note)) {
            Err(e) => assert_eq!(e.to_string(),"UNIQUE constraint failed: metadata.uuid") ,
            _ => panic!("This insert operation should panic"),
        };
    }

    /// Appends an additional note to a super-note and checks if both are there
    #[test]
    fn append_additional_note() {
        use crate::builder::HeaderBuilder;

        dotenv::dotenv().ok();
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");
        let m_data: ::model::NotesMetadata =
            NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());

        let note_body = Body::new(Some(0), m_data.uuid.clone());
        let additional_body = Body::new(Some(1), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body.clone()
    );

        match con.insert_into_db(&note)
            .and_then(|_| con.append_note(&additional_body))
            .and_then(|_| con.fetch_single_note(&note.metadata.uuid.clone())) {
            Ok(Some(mut note2)) => {
                assert_eq!(note.metadata,note2.metadata);
                assert_eq!(note2.body.len(),2);

                let first_note = note2.body.pop().unwrap();
                let second_note = note2.body.pop().unwrap();

                //TODO check if order is always the same
                assert_eq!(second_note,note_body);
                assert_eq!(first_note,additional_body);

            },
            Ok(None) => panic!("No Note found, should at least find one"),
            Err(e) => panic!("DB Transaction failed: {}", e.to_string())
        }
    }

    #[test]
    /// This test adds 2 bodies to a note and replaces it with a "merged" one
    /// the old bodies should be gone now and a new single one should be present
    fn replace_with_merged_body() {
        use crate::builder::HeaderBuilder;

        //Setup
        dotenv::dotenv().ok();
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");
        let m_data: ::model::NotesMetadata =
            NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());

        let note_body = Body::new(Some(0), m_data.uuid.clone());
        let additional_body = Body::new(Some(1), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body.clone()
    );

        match con.insert_into_db(&note)
            .and_then(|_| con.append_note(&additional_body))
            .and_then(|_| con.fetch_single_note(&note.metadata.uuid.clone())) {
            Ok(Some(mut note2)) => {
                assert_eq!(note2.metadata, note.metadata);
                assert_eq!(note2.body.len(), 2);

                let first_note = note2.body.pop().unwrap();
                let second_note = note2.body.pop().unwrap();

                //TODO check if order is always the same
                assert_eq!(second_note, note_body);
                assert_eq!(first_note, additional_body);
            },
            Ok(None) => panic!("No Note found, should at least find one"),
            Err(e) => panic!("DB Transaction failed: {}", e.to_string())
        }

        //Actual test

        let merged_body = Body::new(None, note.metadata.uuid.clone());
        match con.update_merged_note(&merged_body).
            and_then(|_| con.fetch_single_note(&note.metadata.uuid.clone())) {
            Ok(Some(mut note2)) => {
                assert_eq!(note.metadata,note2.metadata);
                assert_eq!(note2.body.len(),1_usize);
                assert_eq!(note2.body.pop().unwrap(),merged_body);
            },
            Ok(None) => panic!("No Note found, should at least find one"),
            Err(e) => {
                panic!("Error while updating merged body: {}", e.to_string());
            }
        }
    }

    #[test]
    fn test_delete_multiple_bodies() {

        dotenv::dotenv().ok();
        let con = ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");

        let first = note![
                NotesMetadataBuilder::new().with_uuid("1").build(),
                BodyMetadataBuilder::new().with_message_id("1").build(),
                BodyMetadataBuilder::new().with_message_id("2").build()
        ];

        let second = note![
                NotesMetadataBuilder::new().with_uuid("2").build(),
                BodyMetadataBuilder::new().with_message_id("3").build()
        ];

        con.insert_into_db(&first).unwrap();
        con.insert_into_db(&second).unwrap();

        assert_eq!(con.fetch_all_notes().unwrap().len(),2);

        con.delete_note_bodies(&vec![
            &BodyMetadataBuilder::new().with_metadata_uuid("2").with_message_id("3").build(),
            &BodyMetadataBuilder::new().with_metadata_uuid("1").with_message_id("2").build()
        ]).unwrap();

        let notes = con.fetch_all_notes().unwrap();

        // one note should be deleted
        assert_eq!(notes.len(),1);
        assert_eq!(notes.iter().next().unwrap().metadata.uuid,"1".to_string());

    }
}