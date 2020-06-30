extern crate xdg;
extern crate regex;
extern crate log;

use std::fs;
use self::regex::Regex;
use self::xdg::*;
use std::fs::File;
use self::log::{info, warn};

pub struct Profile {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) imap_server: String
}

pub fn load_profile() -> Profile {
    let xdg_dir = BaseDirectories::new().expect("Could not find xdg dirs");
    let path = match xdg_dir.find_config_file("apple_notes/config") {
        Some(path) => path,
        None => {
            warn!("Could not detect config file, gonna create empty one");
            let mut path = xdg_dir.create_config_directory("apple_notes").expect("Could not create apple_notes config folder");
            path.push("config");
            File::create(&path).expect("Unable to create file");
            path
        }
    };

    info!("Read config file from {}", &path.as_os_str().to_str().unwrap());
    let creds = fs::read_to_string(&path).expect(format!("error reading config file at {}", path.into_os_string().to_str().unwrap()).as_ref());

    let username_regex = Regex::new(r"^username=(.*)").unwrap();
    let password_regex = Regex::new(r"password=(.*)").unwrap();
    let imap_regex = Regex::new(r"imap_server=(.*)").unwrap();

    fn get_with_regex(regex: Regex, creds: &str) -> String {
        regex.captures(&creds)
            .and_then(|captured| captured.get(1))
            .and_then(|result| Option::from(result.as_str().to_string()))
            .expect(format!("Could not get value for {}", regex.as_str()).as_ref())
    }

    let username = get_with_regex(username_regex, &creds);
    let password = get_with_regex(password_regex, &creds);
    let imap_server = get_with_regex(imap_regex, &creds);

    Profile {
        username,
        password,
        imap_server
    }
}

pub fn get_notes_dir() -> String {
    let xdg = BaseDirectories::new().expect("Could not find xdg data dir");
    if let Some(dir) = xdg.find_data_file("notes") {
        dir.to_string_lossy().to_string() + "/"
    } else {
        info!("No xdg data dir found, create a new one");
        xdg.create_data_directory("notes").expect("Could not create xdg data dir").to_string_lossy().to_string() +"/"
    }
}