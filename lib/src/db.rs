use std::env;

/**
Windows: Register sqlite dll with "lib /MACHINE:X64 /def:sqlite3.def /out:sqlite3.lib" on x64
**/

use diesel::*;
use io::establish_connection;

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