extern crate log;

/**
Windows: Register sqlite dll with "lib /MACHINE:X64 /def:sqlite3.def /out:sqlite3.lib" on x64
**/

use ::{schema};

//use util::get_notes_file_path_from_metadata;
use diesel::{SqliteConnection, Connection};
use std::env;
use diesel::*;
use diesel::result::Error;
use model::{NotesMetadata, Body};
use schema::metadata::dsl::metadata;
use self::log::*;
use schema::body::columns::metadata_uuid;
use std::collections::HashSet;
use note::{LocalNote, IdentifyableNote};
use builder::*;

pub fn delete_everything(connection: &SqliteConnection) -> Result<(), Error> {
    connection.transaction::<_,Error,_>(|| {
        diesel::delete(schema::metadata::dsl::metadata)
            .execute(connection)?;

        diesel::delete(schema::body::dsl::body)
            .execute(connection)?;

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
pub fn append_note(connection: &SqliteConnection, body: &Body) -> Result<(), Error> {
    connection.transaction::<_,Error,_>(|| {
        diesel::insert_into(schema::body::table)
            .values(body)
            .execute(connection)?;

        Ok(())
    })
}

/// In case of a successful merge this method replaces all unmerged notes with a single
/// merged note
pub fn update_merged_note(connection: &SqliteConnection, body: &Body) -> Result<(), Error> {
    connection.transaction::<_,Error,_>(|| {
        diesel::delete(schema::body::dsl::body.filter(metadata_uuid.eq(body.metadata_uuid.clone())))
            .execute(connection)?;
        append_note(connection,body)?;
        Ok(())
    })
}

pub fn delete(connection: &SqliteConnection, local_note: &LocalNote) -> Result<(), Error> {
    connection.transaction::<_, Error, _>(|| {

        diesel::delete(schema::metadata::dsl::metadata)
            .filter(schema::metadata::dsl::uuid.eq(&local_note.metadata.uuid))
            .execute(connection)?;

        diesel::delete(schema::body::dsl::body)
            .filter(schema::body::dsl::metadata_uuid.eq(&local_note.metadata.uuid))
            .execute(connection)?;

        Ok(())
    })
}

pub fn update(connection: &SqliteConnection, local_note: &LocalNote) -> Result<(), Error> {
    connection.transaction::<_, Error, _>(|| {

        delete(connection, local_note);
        insert_into_db(connection, local_note);

        Ok(())
    })
}

/// Inserts the provided post into the sqlite db
pub fn insert_into_db(connection: &SqliteConnection, note: &LocalNote) -> Result<(), Error> {
    connection.transaction::<_,Error,_>(|| {
        diesel::insert_into(schema::metadata::table)
            .values(&note.metadata)
            .execute(connection)?;

        for note_content in &note.body {
            diesel::insert_into(schema::body::table)
                .values(note_content)
                .execute(connection)?;
        }

        Ok(())
    })
}

pub fn fetch_all_notes(connection: &SqliteConnection) -> Result<HashSet<LocalNote>,Error> {
    let notes: Vec<NotesMetadata> = metadata
        .load::<NotesMetadata>(connection)?;

    let note_bodies: Vec<Body> = ::model::Body::belonging_to(&notes)
        .load::<Body>(connection)?;

    let grouped = note_bodies.grouped_by(&notes);

    let d = notes.into_iter().zip(grouped).map(|(m_data,bodies)| {
        LocalNote {
            metadata: m_data,
            body: bodies
        }
    }).collect();

    Ok(d)
}

pub fn fetch_single_note(connection: &SqliteConnection, id: String) -> Result<Option<(NotesMetadata, Vec<Body>)>, Error> {

    let mut notes: Vec<NotesMetadata> = metadata
        .filter(schema::metadata::dsl::uuid.eq(&id))
        .load::<NotesMetadata>(connection)?;

    assert!(notes.len() <= 1);

    if notes.len() == 0 {
        return Ok(None)
    }

    let first_note = notes.remove(0);

    debug!("Fetched note with uuid {}", first_note.uuid.clone());

    let body = ::model::Body::belonging_to(&first_note)
        .load::<Body>(connection)?;

    assert!(&body.len() >= &1_usize);

    debug!("This note has {} subnotes ", body.len());

    Ok(Some((first_note,body)))
}

pub fn establish_connection() -> SqliteConnection {

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
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

    let con = establish_connection();
    delete_everything(&con).expect("Should delete the db");
    insert_into_db(&con, &note).expect("Should insert note into the db");
    insert_into_db(&con, &note_with_2_bodies).expect("Should insert note into the db");

    match fetch_all_notes(&con) {
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



/// Should insert a single metadata object with a body
///
/// This test should return this note correctly after it got
/// saved.
#[test]
fn insert_single_note() {
    use util::HeaderBuilder;
    let con = establish_connection();
    delete_everything(&con).expect("Should delete the db");
    let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
    let body = Body::new(Some(0), m_data.uuid.clone());

    let note = note!(
        m_data,
        body
    );

    insert_into_db(&con, &note).expect("Should insert note into the db");

    match fetch_single_note(&con, note.uuid().clone()) {
        Ok(Some((fetched_note, mut bodies))) => {
            assert_eq!(note.metadata,fetched_note);
            assert_eq!(bodies.len(),1);

            let first_note = bodies.pop().unwrap();
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
    use util::HeaderBuilder;
    //Setup
    dotenv::dotenv().ok();
    let con = establish_connection();
    delete_everything(&con).expect("Should delete everything");
    let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
    let body = Body::new(Some(0), m_data.uuid.clone());

    let note = note!(
        m_data,
        body
    );

    match insert_into_db(&con,&note)
        .and_then(|_| insert_into_db(&con,&note)) {
        Err(e) => assert_eq!(e.to_string(),"UNIQUE constraint failed: metadata.uuid") ,
        _ => panic!("This insert operation should panic"),
    };
}

/// Appends an additional note to a super-note and checks if both are there
#[test]
fn append_additional_note() {
    use util::HeaderBuilder;

    dotenv::dotenv().ok();
    let con = establish_connection();
    delete_everything(&con).expect("Should delete everything");
    let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
    let body = Body::new(Some(0), m_data.uuid.clone());
    let additional_body = Body::new(Some(1), m_data.uuid.clone());

    let note = note!(
        m_data,
        body.clone()
    );

    match insert_into_db(&con,&note)
        .and_then(|_| append_note(&con, &additional_body))
        .and_then(|_| fetch_single_note(&con, note.metadata.uuid.clone())) {
        Ok(Some((fetched_note, mut bodies))) => {
            assert_eq!(fetched_note,note.metadata);
            assert_eq!(bodies.len(),2);

            let first_note = bodies.pop().unwrap();
            let second_note = bodies.pop().unwrap();

            //TODO check if order is always the same
            assert_eq!(second_note,body);
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
    use util::HeaderBuilder;


    //Setup
    dotenv::dotenv().ok();
    let con = establish_connection();
    delete_everything(&con).expect("Should delete everything");
    let m_data: ::model::NotesMetadata = NotesMetadata::new(&HeaderBuilder::new().build(), "test".to_string());
    let body = Body::new(Some(0), m_data.uuid.clone());
    let additional_body = Body::new(Some(1), m_data.uuid.clone());

    let note = note!(
        m_data,
        body.clone()
    );

    match insert_into_db(&con,&note)
        .and_then(|_| append_note(&con, &additional_body))
        .and_then(|_| fetch_single_note(&con, note.metadata.uuid.clone())) {
        Ok(Some((fetched_note, mut bodies))) => {
            assert_eq!(fetched_note, note.metadata);
            assert_eq!(bodies.len(), 2);

            let first_note = bodies.pop().unwrap();
            let second_note = bodies.pop().unwrap();

            //TODO check if order is always the same
            assert_eq!(second_note, body);
            assert_eq!(first_note, additional_body);
        },
        Ok(None) => panic!("No Note found, should at least find one"),
        Err(e) => panic!("DB Transaction failed: {}", e.to_string())
    }

    //Actual test

    let merged_body = Body::new(None, note.metadata.uuid.clone());
    match update_merged_note(&con,&merged_body).
        and_then(|_| fetch_single_note(&con, note.metadata.uuid.clone())) {
        Ok(Some((fetched_note, mut bodies))) => {
            assert_eq!(note.metadata,fetched_note);
            assert_eq!(bodies.len(),1_usize);
            assert_eq!(bodies.pop().unwrap(),merged_body);
        },
        Ok(None) => panic!("No Note found, should at least find one"),
        Err(e) => {
            panic!("Error while updating merged body: {}", e.to_string());
        }
    }
}