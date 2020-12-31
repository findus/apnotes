extern crate log;

use model::{NotesMetadata, Body};


/*pub fn save_all_notes_to_file(notes: &Vec<NotesMetadata>) {
    notes.into_iter().for_each(|note| {
      match save_note_to_file(note) {
          Err(e) => {
              error!("Could not save file {} {}", note.mail_headers.subject_with_identifier(), e.to_string());
          },
          _ => {}
      }
    });
}*/

/*pub fn save_note_to_file<T: NoteTrait>(note: &T) -> Result<()> {
    let location = profile::get_notes_dir().join(&note.folder()).join(&note.metadata().subject_with_identifier());
    info!("Save to {}", location.to_string_lossy());

    let path = std::path::Path::new(&location);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let mut f = File::create(location).expect("Unable to create file");
    f.write_all(converter::convert2md(&note.body()).as_bytes())
}*/

/*pub fn save_text_to_file(metadata: &NotesMetadata) -> Result<()> {
    let path = get_notes_file_path_from_metadata(&metadata);
    info!("Saving text to {}", path.to_string_lossy().into_owned());
    File::create(path)
        .and_then(|mut file| file.write_all(metadata.subject.as_ref()))
}

pub fn save_metadata_to_file(metadata: &NotesMetadata) -> Result<String> {
    let path = profile::get_notes_dir().join(&metadata.subfolder).join(&( ".".to_string() + &metadata.subject_with_identifier() + "_hash"));
    info!("Save metadata {} to {}",metadata.subject(), path.to_string_lossy().into_owned());
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let f = File::create(&path).expect(format!("Unable to create hash file for {}", path.to_string_lossy()).as_ref());

    serde_json::to_writer(f, &metadata)
        .map(|_| metadata.subject_escaped())
        .map_err(|e| std::io::Error::from(e))
}

pub fn delete_metadata_file(metadata: &NotesMetadata) -> Result<()> {
    let old_location = profile::get_notes_dir().join(&metadata.subfolder).join(&(".".to_string() + &metadata.subject_with_identifier() + "_hash"));
    let old_path = std::path::Path::new(&old_location);
    std::fs::remove_file(old_path)
}

pub fn delete_note(metadata: &NotesMetadata) -> Result<()> {
    let old_location = profile::get_notes_dir().join(&metadata.subfolder).join(&metadata.subject_with_identifier());
    let old_path = std::path::Path::new(&old_location);
    std::fs::remove_file(old_path)
}

pub fn move_note(metadata: &NotesMetadata, old_escaped_subject: &String) -> Result<()> {
    let new_location = profile::get_notes_dir().join(&metadata.subfolder).join(&metadata.subject_with_identifier());
    let new_path = std::path::Path::new(&new_location);

    let old_location = profile::get_notes_dir().join(&metadata.subfolder).join(old_escaped_subject);
    let old_path = std::path::Path::new(&old_location);

    info!("Move {} to {}", old_location.to_string_lossy(), new_location.to_string_lossy());
    std::fs::rename(old_path,new_path)
}*/
