extern crate xdg;
extern crate regex;
extern crate log;

use std::fs;
use self::regex::Regex;

use std::fs::File;
use self::log::{info, warn};
use std::path::PathBuf;

#[cfg(target_family = "unix")]
use self::xdg::BaseDirectories;

pub struct Profile {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) imap_server: String,
    pub(crate) email: String
}

impl Profile {
    pub(crate) fn domain(&self) -> String {
        let uuid_regex = Regex::new(r".*@(.*)").unwrap();
        get_with_regex(uuid_regex, &self.email)
    }
}

#[cfg(target_family = "unix")]
pub fn get_config_path() -> PathBuf {
        let xdg_dir = BaseDirectories::new().expect("Could not find xdg dirs");
        match xdg_dir.find_config_file("apple_notes/config") {
            Some(path) => path,
            None => {
                warn!("Could not detect config file, gonna create empty one");
                let mut path = xdg_dir.create_config_directory("apple_notes").expect("Could not create apple_notes config folder");
                path.push("config");
                File::create(&path).expect("Unable to create file");
                path.to_path_buf()
            }
        }
}

#[cfg(target_family = "windows")]
pub fn get_config_path() -> PathBuf {
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
pub fn get_db_path() -> PathBuf {
    let xdg_dir = BaseDirectories::new().expect("Could not find xdg dirs");
    match xdg_dir.find_data_file("apple_notes/notes_db") {
        Some(path) => path,
        None => {
            warn!("Could not detect database, gonna create empty one");
            let mut path = xdg_dir.create_data_directory("apple_notes").expect("Could not create apple_notes config folder");
            path.push("notes_db");
            File::create(&path).expect("Unable to create file");
            path.to_path_buf()
        }
    }
}

#[cfg(target_family = "windows")]
pub fn get_db_path() -> PathBuf {
    let db_file_path = PathBuf::from(format!("{}\\{}", env!("APPDATA"), "rs-notes\\db".to_string()));
    if db_file_path.exists() {
        db_file_path
    } else {
        warn!("Could not detect database, gonna create empty one");
        std::fs::create_dir(&db_file_path.parent().unwrap());
        File::create(&db_file_path).expect("Unable to create config file");
        db_file_path
    }
}


pub fn load_profile() -> Profile {
    let path = get_config_path();
    info!("Read config file from {}", &path.as_os_str().to_str().unwrap());
    let creds = fs::read_to_string(&path).expect(format!("error reading config file at {}", path.into_os_string().to_str().unwrap()).as_ref());

    let username_regex = Regex::new(r"username=(.*)").unwrap();
    let password_regex = Regex::new(r"password=(.*)").unwrap();
    let imap_regex = Regex::new(r"imap_server=(.*)").unwrap();
    let email_regex = Regex::new(r"email=(.*)").unwrap();

    let username = get_with_regex(username_regex, &creds);
    let password = get_with_regex(password_regex, &creds);
    let imap_server = get_with_regex(imap_regex, &creds);
    let email = get_with_regex(email_regex, &creds);

    Profile {
        username,
        password,
        imap_server,
        email
    }
}

#[cfg(target_family = "unix")]
pub fn get_notes_dir() -> PathBuf {
    let xdg = BaseDirectories::new().expect("Could not find xdg data dir");
    if let Some(dir) = xdg.find_data_file("notes") {
        dir
    } else {
        info!("No xdg data dir found, create a new one");
        xdg.create_data_directory("notes").expect("Could not create xdg data dir")
    }
}

#[cfg(target_family = "windows")]
pub fn get_notes_dir() -> PathBuf {
    let notes_dir_path = PathBuf::from(format!("{}\\{}",env!("APPDATA"),"rs-notes\\notes".to_string()));
    if notes_dir_path.exists() {
        notes_dir_path
    } else {
        info!("No notes dir found, will create a new one");
        std::fs::create_dir(&notes_dir_path).expect("Could not create notes dir");
        notes_dir_path
    }
}

fn get_with_regex(regex: Regex, creds: &str) -> String {
    regex.captures(&creds)
        .and_then(|captured| captured.get(1))
        .and_then(|result| Option::from(result.as_str().to_string()))
        .expect(format!("Could not get value for {}", regex.as_str()).as_ref())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}