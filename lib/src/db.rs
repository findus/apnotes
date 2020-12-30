use std::env;

/**
Windows: Register sqlite dll with "lib /MACHINE:X64 /def:sqlite3.def /out:sqlite3.lib" on x64
**/

use diesel::*;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[test]
fn test() {

    use schema::metadata::dsl::*;
    use model::NotesMetadata;

    let connection = establish_connection();
    let results = metadata
        .load::<NotesMetadata>(&connection)
        .expect("Error loading Metadata");

    for result in results {
        println!("{}", result.date)
    }

}