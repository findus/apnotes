extern crate subprocess;
extern crate regex;
extern crate log;
extern crate uuid;

use error::NoteError;
use error::NoteError::{EditError, ContentNotChanged};
use self::log::*;
use std::io::{Write};
use ::model::Body;
use builder::{BodyMetadataBuilder};
use notes::localnote::LocalNote;

#[cfg(test)]
use self::regex::Regex;




/// Edits the passed note and alters the metadata if successful
pub fn edit_note(local_note: &LocalNote, new: bool) -> Result<LocalNote, NoteError> {


    if local_note.needs_merge() {
        return Err(NoteError::NeedsMerge);
    }

    let note = local_note.body.first()
        .expect("Expected at least 1 note body");

    let _env = std::env::var("RS_NOTES_EDITOR");

    let profile = ::profile::load_profile();

    #[cfg(target_family = "unix")]
        let file_path = format!("/tmp/{}_{}", note.metadata_uuid , note.subject_escaped());

    #[cfg(target_family = "windows")]
        let file_path = format!("{}\\{}_{}",std::env::var_os("TEMP").unwrap().to_string_lossy().to_owned(), note.metadata_uuid , note.subject_escaped());

    info!("Opening Note for editing: {} new file: {} path: {}", note.subject(), new,  file_path);

    let mut file = std::fs::File::create(&file_path).expect("Could not create file");
    file.write_all(note.text.as_ref().unwrap_or(&"".to_string()).as_bytes())
        .expect("Could not write to file");

    info!("Exec: {} {}", &profile.editor, &file_path);

    subprocess::Exec::cmd(&profile.editor).args(&profile.editor_arguments).arg(&file_path)
        .join()
        .map_err(|e| EditError(e.to_string()))
        .and_then(|_| read_edited_text(local_note, note, &file_path))
        .and_then(|localnote| remove_temp_file(&file_path).map(|_| localnote))
}

fn remove_temp_file(file_path: &String) -> Result<(), NoteError> {
    println!("Removing temp file {}", &file_path);
    std::fs::remove_file(&file_path)
        .map_err(|e| NoteError::EditError(e.to_string()))
}

fn read_edited_text(local_note: &LocalNote, note: &Body, file_path: &str) -> Result<LocalNote, NoteError> {
    //Read content and save to body.text
    let file_content = std::fs::read_to_string(&file_path)
        .map_err(|e| NoteError::EditError(e.to_string()))?;


    if &file_content == note.text.as_ref().unwrap_or(&"".to_string())
        && local_note.metadata.new == false {
        return Err(ContentNotChanged);
    } else {
        Ok(
            note!(
            // note: bodymetadatabuilder generates a new message-id here
                  local_note.metadata.clone(),
                  BodyMetadataBuilder::new()
                  .with_old_remote_message_id(&note.message_id)
                  .with_uid(note.uid.expect("Expected UID").clone())
                  .with_text(&file_content)
                  .build()
            )
        )
    }
}

#[cfg(test)]
fn replace_uuid(string: &str) -> String {
    let uuid_regex = Regex::new(r"(.*<)\b[0-9A-F]{8}\b-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-\b[0-9A-F]{12}\b(.*)").unwrap();
    let new_uuid = uuid::Uuid::new_v4().to_string().to_uppercase();
    let dd = format!("${{1}}{}${{2}}",new_uuid);
    uuid_regex.replace(string, dd.as_str()).to_string()
}

#[cfg(test)]
mod edit_tests {
    use error::NoteError;
    use edit::{edit_note, replace_uuid};
    use builder::*;

    #[test]
    fn should_generate_new_uuid() {
        let old_uuid = "Message-Id: <7A41875C-2CCF-4AE4-869E-1F230E1B71BA@test.mail>";
        assert_ne!(old_uuid.to_string(), replace_uuid(old_uuid));
    }

    /// A note should not be able to be edited if it is not merged
    #[test]
    fn edit_note_merge() {
        let note = note!(
        NotesMetadataBuilder::new().build(),
        BodyMetadataBuilder::new().build(),
        BodyMetadataBuilder::new().build()
    );

        match edit_note(&note, false) {
            Err(e) => { assert_eq!(e, NoteError::NeedsMerge) }
            Ok(_) => panic!("Should be error")
        }
    }
}
