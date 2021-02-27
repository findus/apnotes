extern crate xdg;
extern crate regex;
extern crate log;

use std::fs;
use self::regex::Regex;

use std::fs::File;
use self::log::{info, warn};
use std::path::PathBuf;
use error::ProfileError::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(target_family = "unix")]
use self::xdg::BaseDirectories;

pub struct Profile {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) imap_server: String,
    pub(crate) email: String,
    pub(crate) editor: String,
    pub(crate) editor_arguments: Vec<String>,
    #[allow(dead_code)]
    pub(crate) domain: String
}

impl Profile {

}

#[cfg(target_family = "unix")]
pub(crate)  fn get_config_path() -> Result<PathBuf> {
        let xdg_dir = BaseDirectories::new()?;
        match xdg_dir.find_config_file("apple_notes/config") {
            Some(path) => Ok(path),
            None => {
                warn!("Could not detect config file, gonna create empty one");
                let mut path = xdg_dir.create_config_directory("apple_notes")?;
                path.push("config");
                File::create(&path).expect("Unable to create file");
                Ok(path.to_path_buf())
            }
        }
}

#[cfg(target_family = "windows")]
pub(crate)  fn get_config_path() -> PathBuf {
    let config_file_path = PathBuf::from(format!("{}\\{}",env!("APPDATA"),"rs-notes\\config".to_string()));
    if config_file_path.exists() {
        config_file_path
    } else {
        warn!("Could not detect config file, gonna create empty one");
        std::fs::create_dir(&config_file_path.parent().unwrap()).expect("Unable to create config folder");
        File::create(&config_file_path).expect("Unable to create config file");
        config_file_path
    }
}

#[cfg(target_family = "unix")]
pub(crate)  fn get_db_path() -> PathBuf {
    let xdg_dir = BaseDirectories::new().expect("Could not find xdg dirs");
    #[cfg(test)]
        let db = "notes_db_test";
    #[cfg(not(test))]
        let db = "notes_db";
    match xdg_dir.find_data_file(format!("apple_notes/{}",db)) {
        Some(path) => path,
        None => {
            warn!("Could not detect database, gonna create empty one");
            let mut path = xdg_dir.create_data_directory("apple_notes").expect("Could not create apple_notes config folder");
            path.push(&db);
            File::create(&path).expect("Unable to create file");
            path.to_path_buf()
        }
    }
}

#[cfg(target_family = "windows")]
pub(crate)  fn get_db_path() -> PathBuf {
    let db_file_path = PathBuf::from(format!("{}\\{}", env!("APPDATA"), "rs-notes\\db".to_string()));
    if db_file_path.exists() {
        db_file_path
    } else {
        warn!("Could not detect database, gonna create empty one");
        if std::fs::create_dir(&db_file_path.parent().unwrap()).is_err() {
            error!("Folder does already exist")
        }
        File::create(&db_file_path).expect("Unable to create config file");
        db_file_path
    }
}


pub(crate) fn load_profile() -> Result<Profile> {
    let path = get_config_path()?;
    let path = path.into_os_string().to_string_lossy().to_string();

    info!("Read config file from {}", &path);
    let creds = fs::read_to_string(&path)?;

    let username_regex = Regex::new(r"username=(.*)")?;
    let password_regex = Regex::new(r"password=(.*)")?;
    let imap_regex = Regex::new(r"imap_server=(.*)")?;
    let email_regex = Regex::new(r"email=(.*)")?;
    let editor_regex = Regex::new(r"editor=(.*)")?;
    let args_regex = Regex::new(r"editor_arguments=(.*)")?;
    let uuid_regex = Regex::new(r".*@(.*)")?;

    let username = get_with_regex(username_regex, &creds)?;
    let password = get_with_regex(password_regex, &creds)?;
    let imap_server = get_with_regex(imap_regex, &creds)?;
    let email = get_with_regex(email_regex, &creds)?;
    let editor = get_with_regex(editor_regex, &creds)?;
    let args = get_with_regex(args_regex, &creds)?.split(" ").map(|s| s.to_string()).filter(|s| s.len() > 0).collect();
    let domain = get_with_regex(uuid_regex, &email)?;

    Ok(
        Profile {
            username,
            password,
            imap_server,
            email,
            editor,
            editor_arguments: args,
            domain
        }
    )
}

fn get_with_regex(regex: Regex, creds: &str) -> Result<String> {
    match regex.captures(&creds)
        .and_then(|captured| captured.get(1))
        .and_then(|result| Option::from(result.as_str().to_string())) {
        Some(e) => Ok(e),
        None => Err(NotFound(regex.to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}