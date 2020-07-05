extern crate subprocess;
extern crate apple_notes_rs;
extern crate regex;
extern crate log;
extern crate uuid;

use std::env;
use std::fs::File;
use apple_notes_rs::note::{NotesMetadata, HeaderParser};
use self::regex::Regex;
use apple_notes_rs::io;
use log::info;
use log::error;
use log::debug;
use apple_notes_rs::util;
use std::io::{BufReader, BufRead};
use apple_notes_rs::error::UpdateError::EditError;
use apple_notes_rs::error::UpdateError;
use std::time;

pub fn main() {
    simple_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();

    let file = args.get(1).unwrap();
    let result = subprocess::Exec::cmd("xdg-open").arg(file)
        .join()
        .map_err(|e| EditError(e.to_string()))
        .and_then(|_| std::fs::metadata(file).map_err(|e| EditError(e.to_string())))
        .and_then(|metadata| {
            let change_duration =
                time::SystemTime::now()
                    .duration_since(metadata.modified()
                        .expect("No System time found"))
                    .unwrap();

            if change_duration.as_secs() > 10 {
                Err(EditError("File not changed".to_string()))
            } else {
                Ok(())
            }
        })
        .and_then(|_| self::update(&file));

    match result {
        Ok(_) => info!("Note changed successfully"),
        Err(e) => error!("{}" ,e)
    }

}

fn update(file: &String) -> Result<String, UpdateError> {
    info!("Update Message_Id for {}", &file);
    let path = std::path::Path::new(file).to_owned();
    let metadata_file_path = util::get_hash_path(&path);

    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut first_line= String::new();
    reader.read_line(&mut first_line).expect("Could not read first line");
    let len = first_line.len();
    first_line.truncate(len - 1);

    let metadata_file = File::open(&metadata_file_path)
        .expect(&format!("Could not open {}", &metadata_file_path.to_string_lossy()));

    let metadata: NotesMetadata = serde_json::from_reader(metadata_file).unwrap();

    let old_subject = metadata.subject();

    if old_subject != first_line {
        info!("Title of note changed, will update metadata_file subject and file-name");
    }

    let metadata_identifier = metadata.message_id();
    let new_metadata_headers_iterator =
            metadata.header.clone()
                .into_iter()
                .filter(|(a,_)| a != "Message-Id")
                .filter(|(a,_)| a != "Subject");

    let mut new_metadata_headers: Vec<(String,String)> = new_metadata_headers_iterator.collect();

    if old_subject != first_line {
        info!("Title has changed, file is getting renamed");
        new_metadata_headers.push(("Subject".to_owned(), first_line.to_owned()));
    } else {
        new_metadata_headers.push(("Subject".to_owned(), old_subject.to_owned()));
    }

    let mut new_metadata = NotesMetadata {
        header: new_metadata_headers.clone(),
        old_remote_id: Some(metadata_identifier.clone()),
        subfolder: metadata.subfolder.to_string(),
        locally_deleted: false,
        uid: metadata.uid,
        // check if ok
        new: true
    };

    debug!("Changing files message id...");
    let new_uuid_str = replace_uuid(&metadata_identifier);
    new_metadata_headers.push(("Message-Id".to_owned(), new_uuid_str.clone()));

    new_metadata.header = new_metadata_headers.clone();

    if old_subject != first_line {
        io::save_metadata_to_file(&new_metadata)
            .map_err(|e| std::io::Error::from(e))
            .and_then(|_| io::move_note(&new_metadata, &metadata.subject_with_identifier()))
            .and_then(|_| io::delete_metadata_file(&metadata))
            .map(|_| new_metadata.subject_escaped())
            .map_err(|e| EditError(e.to_string()))
    } else {
        io::save_metadata_to_file(&new_metadata)
            .map(|_| new_metadata.subject_escaped())
            .map_err(|e| EditError(e.to_string()))
    }

}

fn replace_uuid(string: &str) -> String {
    let uuid_regex = Regex::new(r"(.*<)\b[0-9A-F]{8}\b-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-\b[0-9A-F]{12}\b(.*)").unwrap();
    let new_uuid = uuid::Uuid::new_v4().to_string().to_uppercase();
    let dd = format!("${{1}}{}${{2}}",new_uuid);
    uuid_regex.replace(string, dd.as_str()).to_string()
}

#[test]
fn should_generate_new_uuid() {
    let old_uuid = "Message-Id: <7A41875C-2CCF-4AE4-869E-1F230E1B71BA@test.mail>";
    assert_ne!(old_uuid.to_string(), replace_uuid(old_uuid));
}