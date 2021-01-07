use uuid::Uuid;

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

