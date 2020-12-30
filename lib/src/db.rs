use std::env;

/**
Windows: Register sqlite dll with "lib /MACHINE:X64 /def:sqlite3.def /out:sqlite3.lib" on x64
**/

use diesel::*;
use diesel::prelude::*;
use model::NotesMetadata;
use diesel::sqlite::Sqlite;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[test]
fn test() {

    use schema::metadata::dsl::*;

    let connection = establish_connection();
    let results = metadata
        .load::<NotesMetadata>(&connection)
        .expect("Error loading Metadata");

    for result in results {
        println!("{}", result.date)
    }

}