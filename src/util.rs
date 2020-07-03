use std::path::{Path, PathBuf};
use note::{NotesMetadata, HeaderParser};
use profile;

pub fn get_hash_path(path: &Path) -> PathBuf {
    let folder = path.parent().unwrap().to_string_lossy().into_owned();
    let new_file_name = format!(".{}_hash",path.file_name().unwrap().to_string_lossy().into_owned());

    let metadata_file_path = format!("{}/{}",&folder,&new_file_name).to_owned();
    std::path::Path::new(&metadata_file_path).to_owned()
}

pub fn get_notes_file_from_metadata(metadata: &NotesMetadata) -> PathBuf {
    let path = format!("{}/{}/{}", profile::get_notes_dir(), metadata.subfolder, metadata.subject_with_identifier());
    Path::new(&path).to_path_buf()
}