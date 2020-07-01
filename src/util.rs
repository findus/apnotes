use std::path::{Path, PathBuf};

pub fn get_hash_path(path: &Path) -> PathBuf {
    let folder = path.parent().unwrap().to_string_lossy().into_owned();
    let new_file_name = format!(".{}_hash",path.file_name().unwrap().to_string_lossy().into_owned());

    let metadata_file_path = format!("{}/{}",&folder,&new_file_name).to_owned();
    std::path::Path::new(&metadata_file_path).to_owned()
}