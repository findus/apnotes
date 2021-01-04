use std::path::{Path, PathBuf};

use profile;
use uuid::Uuid;
use chrono::{Utc};
use note::{NoteHeaders, HeaderParser};

pub fn get_hash_path(path: &Path) -> PathBuf {
    let folder = path.parent().unwrap().to_string_lossy().into_owned();
    let new_file_name = format!(".{}_hash",path.file_name().unwrap().to_string_lossy().into_owned());
    std::path::Path::new(&folder).join(&new_file_name).to_owned()
}

/*pub fn get_notes_file_path_from_metadata(metadata: &NotesMetadata) -> PathBuf {
    let pathbuf = PathBuf::new()
        .join(profile::get_notes_dir())
        .join(PathBuf::from(&metadata.subfolder))
        .join(PathBuf::from(metadata.subject_with_identifier()));
    pathbuf
}
*/
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string().to_uppercase()
}

pub fn generate_mail_headers(subject: &str) -> Vec<(String,String)> {
    ::builder::HeaderBuilder::new().with_subject(subject).build()
}