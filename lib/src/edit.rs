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
use model::NotesMetadata;

use chrono::Utc;
use profile::Profile;


/// Edits the passed note and alters the metadata if successful
pub fn edit_note(local_note: &LocalNote, new: bool, profile: &Profile) -> Result<LocalNote, NoteError> {


    if local_note.needs_merge() {
        return Err(NoteError::NeedsMerge);
    }

    let note = local_note.body.first()
        .expect("Expected at least 1 note body");

    let environment_editor = std::env::var("RS_NOTES_EDITOR");

    #[cfg(target_family = "unix")]
        let file_path = format!("/tmp/{}_{}", note.metadata_uuid , note.subject_escaped());

    #[cfg(target_family = "windows")]
        let file_path = format!("{}\\{}",std::env::var_os("TEMP").unwrap().to_string_lossy().to_owned(), note.metadata_uuid);

    info!("Opening Note for editing: {} new file: {} path: {}", note.subject(), new,  file_path);

    {
        let mut file = std::fs::File::create(&file_path).expect("Could not create file");
        file.write_all(note.text.as_ref().unwrap_or(&"".to_string()).as_bytes())
            .expect("Could not write to file");
    }

    let (editor, args) = if environment_editor.is_ok() {
        (environment_editor.unwrap(),vec![])
    } else {
        (profile.editor.clone(),profile.editor_arguments.clone())
    };

    info!("Exec: {} {}", editor, &file_path);

    let proc = if args.len() > 0 {
        subprocess::Exec::cmd(&editor).args(&profile.editor_arguments).arg(&file_path)
    } else {
        subprocess::Exec::cmd(&editor).arg(&file_path)
    };

    proc
        .join()
        .map_err(|e| EditError(e.to_string()))
        .and_then(|_| read_edited_text(local_note, note, &file_path, profile))
        .and_then(|localnote| remove_temp_file(&file_path).map(|_| localnote))
}

fn remove_temp_file(file_path: &String) -> Result<(), NoteError> {
    info!("Removing temp file {}", &file_path);
    std::fs::remove_file(&file_path)
        .map_err(|e| NoteError::EditError(e.to_string()))
}

fn read_edited_text(local_note: &LocalNote, note: &Body, file_path: &str, _profile: &Profile) -> Result<LocalNote, NoteError> {
    //Read content and save to body.text
    let file_content = std::fs::read_to_string(&file_path)
        .map_err(|e| NoteError::EditError(e.to_string()))?;

    if &file_content == note.text.as_ref().unwrap_or(&"".to_string())
        && local_note.metadata.new == false {
        return Err(ContentNotChanged);
    } else {
        // Create new edited date that matches current date
        let local_note_metadata = NotesMetadata {
            subfolder: local_note.metadata.subfolder.clone(),
            locally_deleted: local_note.metadata.locally_deleted,
            new: local_note.metadata.new,
            date: Utc::now().to_rfc2822(),
            uuid: local_note.metadata.uuid.clone(),
            mime_version: local_note.metadata.mime_version.clone()
        };

        #[cfg(not(test))]
        let mut body = BodyMetadataBuilder::new(_profile)
            .with_uid(note.uid.clone())
            .with_text(&file_content);

        #[cfg(test)]
            let mut body = BodyMetadataBuilder::new()
            .with_uid(note.uid.clone())
            .with_text(&file_content);

        if local_note.metadata.new == false {
            body = body.with_old_remote_message_id(&note.message_id);
        }

        let body = body.build();

        Ok(
            note!(
                  local_note_metadata,
                  body
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
    use profile::Profile;

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

        let profile = Profile {
            username: "".to_string(),
            password: Option::from("".to_string()),
            imap_server: "".to_string(),
            email: "".to_string(),
            editor: "".to_string(),
            editor_arguments: vec![],
            secret_service_attribute: None,
            secret_service_value: None,
            domain: "".to_string(),
            password_type: "".to_string()
        };

        match edit_note(&note, false, &profile) {
            Err(e) => { assert_eq!(e, NoteError::NeedsMerge) }
            Ok(_) => panic!("Should be error")
        }
    }
}
