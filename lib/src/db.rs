extern crate log;

/**
Windows: Register sqlite dll with "lib /MACHINE:X64 /def:sqlite3.def /out:sqlite3.lib" on x64
Set: SQLITE3_LIB_DIR
**/

use ::{schema};

//use util::get_notes_file_path_from_metadata;
use diesel::{SqliteConnection, Connection};
use std::env;
use diesel::*;
use diesel::result::Error;
use model::{NotesMetadata, Body};
use schema::metadata::dsl::metadata;
use schema::body::dsl::body;
use self::log::*;
use schema::body::columns::metadata_uuid;
use std::collections::HashSet;
use note::{LocalNote};
use std::collections::hash_map::RandomState;
#[cfg(test)]
extern crate mockall;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

pub trait DBConnector {

}

#[cfg_attr(test, automock)]
pub trait DatabaseService<C: 'static + DBConnector> {
    fn delete_everything(&self) -> Result<(), Error>;
    fn append_note(&self, model: &::model::Body) -> Result<(), Error>;
    fn update_merged_note(&self, note_body: &Body) -> Result<(), Error>;
    fn delete(&self, local_note: &LocalNote) -> Result<(), Error>;
    fn update(&self, local_note: &LocalNote) -> Result<(), Error>;
    fn insert_into_db(&self,note: &LocalNote) -> Result<(), Error>;
    fn fetch_all_notes(&self) -> Result<HashSet<LocalNote>,Error>;
    fn fetch_single_note_with_name(&self, name: &str) -> Result<Option<LocalNote>, Error>;
    fn fetch_single_note(&self, id: &str) -> Result<Option<LocalNote>, Error>;
}

pub struct SqLiteConnector {

}

impl SqLiteConnector {
    fn connect() -> SqliteConnection {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or("test".to_string());

        /*    println!("{}", env::current_dir().unwrap().to_string_lossy());
            println!("{}", database_url);*/

        SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }
}

impl DBConnector for SqliteDBConnection {

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


impl DatabaseService<SqliteDBConnection> for SqliteDBConnection {
    fn delete_everything(&self) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {
            diesel::delete(schema::metadata::dsl::metadata)
                .execute(&self.connection)?;

            diesel::delete(schema::body::dsl::body)
                .execute(&self.connection)?;

            Ok(())
        })
    }
    /// Appends a note to an already present note
    ///
    /// Multiple notes only occur if you altered a note locally
    /// and server-side, or if 2 separate devices edited the
    /// same note, in that case 2 notes exists on the imap
    /// server.
    ///
    /// If multiple notes exists tell user that a merge needs to happen
    ///
    fn append_note(&self, model: &Body) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {
            diesel::insert_into(schema::body::table)
                .values(model)
                .execute(&self.connection)?;

            Ok(())
        })
    }

    /// In case of a successful merge this method replaces all unmerged notes with a single
/// merged note
    fn update_merged_note(&self, note_body: &Body) -> Result<(), Error> {
        self.connection.transaction::<_,Error,_>(|| {
            diesel::delete(schema::body::dsl::body.filter(metadata_uuid.eq(note_body.metadata_uuid.clone())))
                .execute(&self.connection)?;
            self.append_note(note_body)?;
            Ok(())
        })
    }

    fn delete(&self, local_note: &LocalNote) -> Result<(), Error> {
        self.connection.transaction::<_, Error, _>(|| {

            diesel::delete(schema::metadata::dsl::metadata)
                .filter(schema::metadata::dsl::uuid.eq(&local_note.metadata.uuid))
                .execute(&self.connection)?;

            diesel::delete(schema::body::dsl::body)
                .filter(schema::body::dsl::metadata_uuid.eq(&local_note.metadata.uuid))
                .execute(&self.connection)?;

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
    /// Inserts the provided post into the sqlite db
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
            .load::<NotesMetadata>(&self.connection)?;

        let note_bodies: Vec<Body> = ::model::Body::belonging_to(&notes)
            .load::<Body>(&self.connection)?;

        let grouped = note_bodies.grouped_by(&notes);

        let d = notes.into_iter().zip(grouped).map(|(m_data,bodies)| {
            LocalNote {
                metadata: m_data,
                body: bodies
            }
        }).collect();

        Ok(d)
    }

    /// Returns a note with a specific subject, if multiple notes have the same subject, the first
    /// found note gets returned
    fn fetch_single_note_with_name(&self, name: &str) -> Result<Option<LocalNote>, Error> {
        let note_bodies: Vec<Body> = body
            .filter(schema::body::dsl::text.like(&format!("{}%",name)))
            .limit(1)
            .load::<Body>(&self.connection)?;

        if note_bodies.len() == 0 {
            return Ok(None)
        }

        let m_data: Vec<NotesMetadata> = metadata
            .filter(schema::metadata::dsl::uuid.eq(&note_bodies.first().unwrap().metadata_uuid))
            .limit(1)
            .load::<NotesMetadata>(&self.connection)?;

        let first_metadata = m_data.first().expect("Expected at least one metadata object");

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

        let note_body = ::model::Body::belonging_to(&first_note)
            .load::<Body>(&self.connection)?;

        assert!(&note_body.len() >= &1_usize);

        debug!("This note has {} subnotes ", note_body.len());

        Ok(Some(LocalNote {
            metadata: first_note,
            body: note_body
        }))
    }
}

#[cfg(test)]
mod db_tests {
    use model::NotesMetadata;
    use note::{LocalNote, IdentifyableNote};
    use builder::*;
    use ::model::Body;
    use super::*;
    use mockall::predicate::*;
    use apple_imap::{ImapSession, TlsImapSession, MailServiceImpl, MailService};
    use imap::Session;
    use native_tls::TlsStream;
    use std::net::TcpStream;
    use error::UpdateError;

    #[test]
    pub fn mock_test() {
        let mut mock_db_service = MockDatabaseService::<SqliteDBConnection>::new();

        mock_db_service.expect_fetch_all_notes().returning(|| Err(diesel::result::Error::NotFound));

        let mut mock_imap_service: ::apple_imap::MockMailService<Session<TlsStream<TcpStream>>> =
            ::apple_imap::MockMailService::<_>::new();

        mock_imap_service.expect_fetch_headers().returning(|| Err(imap::error::Error::Append) );


        let err = ::sync::sync(&mut mock_imap_service, &mock_db_service).err();
        assert_eq!(err,Some(::error::UpdateError::SyncError("oops".to_string())))

    }

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

                // Check if the first note with only one body has correct body assigned to metadata object
                let first_note: Vec<&LocalNote> = notes.iter().filter(|e| e.metadata.uuid == body_in_first_note.metadata_uuid).collect();
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
                assert_eq!(body_in_first_note.is_inside_localnote(third_note.first().unwrap()), false);

            },
            _ => panic!("could not fetch notes")
        }


    }

    #[test]
    fn update_single_note() {
        use builder::HeaderBuilder;
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete the db");

        let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(),
                                                                "test".to_string()
        );

        let note_body = Body::new(Some(0), m_data.uuid.clone());

        let note = note!(
        m_data,
        note_body
    );

        let note_2 = note!(
        NotesMetadataBuilder::new().with_uuid("meem".to_string()).build(),
        BodyMetadataBuilder::new().with_text("old text").build()
    );

        let note_3 = note!(
        NotesMetadataBuilder::new().with_uuid("meem".to_string()).build(),
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

        let m_data: ::model::NotesMetadata = NotesMetadata::new(&::builder::HeaderBuilder::new().build(),
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
        use builder::HeaderBuilder;
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete the db");
        let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
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
        use builder::HeaderBuilder;
        //Setup
        dotenv::dotenv().ok();
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");
        let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
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
        use builder::HeaderBuilder;

        dotenv::dotenv().ok();
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");
        let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
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
        use builder::HeaderBuilder;

        //Setup
        dotenv::dotenv().ok();
        let con =  ::db::SqliteDBConnection::new();
        con.delete_everything().expect("Should delete everything");
        let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
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
}