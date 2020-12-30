extern crate subprocess;
extern crate regex;
extern crate log;
extern crate uuid;


use std::fs::File;
use self::regex::Regex;
use self::log::info;

use self::log::debug;
use std::io::{BufReader, BufRead};
use std::time;
use error::UpdateError::EditError;
use error::UpdateError;
use ::{util, io};
use note::{HeaderParser};
use model::NotesMetadata;

pub fn edit(metadata: &NotesMetadata, new: bool) -> Result<String, UpdateError> {
    let path = util::get_notes_file_path_from_metadata(metadata);
    let path = path.to_string_lossy().into_owned();
    info!("Opening File for editing: {}", path);

    #[cfg(target_family = "unix")]
        let open_with = "xdg-open".to_owned();
    #[cfg(target_family = "windows")]
        let open_with = (std::env::var_os("WINDIR").unwrap().to_string_lossy().to_owned() + "\\system32\\notepad.exe").into_owned();

    subprocess::Exec::cmd(open_with).arg(&path)
        .join()
        .map_err(|e| EditError(e.to_string()))
        .and_then(|_| std::fs::metadata(&path).map_err(|e| EditError(e.to_string())))
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
        .and_then(|_| self::update(&path, new))
}

fn update(file: &str, new: bool) -> Result<String, UpdateError> {
    info!("Update Message_Id for {}", &file);
    let path = std::path::Path::new(file).to_owned();
    let metadata_file_path = util::get_hash_path(&path);

    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut first_line= String::new();
    reader.read_line(&mut first_line).expect("Could not read first line");
    let len = first_line.len();

    if len > 0 {
        first_line.truncate(len - 1);
    }

    let metadata_file = File::open(&metadata_file_path)
        .expect(&format!("Could not open {}", &metadata_file_path.to_string_lossy()));

    let metadata: NotesMetadata = serde_json::from_reader(metadata_file).unwrap();

    let old_subject = metadata.subject();

    if old_subject != first_line {
        info!("Title of note changed, will update metadata_file subject and file-name");
    }

    let metadata_identifier = metadata.message_id();
   /* let new_metadata_headers_iterator =
            metadata.header.clone()
                .into_iter()
                .filter(|(a,_)| a != "Message-Id")
                .filter(|(a,_)| a != "Subject");

    let mut new_metadata_headers: Vec<(String,String)> = new_metadata_headers_iterator.collect();
*/
    if old_subject != first_line {
        info!("Title has changed, file is getting renamed");
       // new_metadata_headers.push(("Subject".to_owned(), first_line.to_owned()));
    } else {
      //  new_metadata_headers.push(("Subject".to_owned(), old_subject.to_owned()));
    }

    //if there already is an "old" remote id,use that instead of using the current one
    let old_remote_id = metadata.clone().old_remote_id.unwrap_or(metadata_identifier.clone());

    let mut new_metadata = NotesMetadata {
        old_remote_id: Some(old_remote_id.clone()),
        subfolder: metadata.subfolder.to_string(),
        locally_deleted: false,
        uid: metadata.uid,
        // check if ok
        new: if new { true } else { false },
        date: Default::default(),
        uuid: "".to_string(),
        mime_version: "".to_string()
    };

    debug!("Changing files message id...");
    let new_uuid_str = replace_uuid(&metadata_identifier);
    //new_metadata_headers.push(("Message-Id".to_owned(), new_uuid_str.clone()));

    //new_metadata.header = new_metadata_headers.clone();

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