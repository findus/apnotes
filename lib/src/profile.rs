extern crate xdg;
extern crate regex;
extern crate log;

use std::fs;
use self::regex::Regex;

use std::fs::File;
use self::log::{warn};
use std::path::PathBuf;
use error::ProfileError::*;
use std::str;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(target_family = "unix")]
use self::xdg::BaseDirectories;
use secret_service::{SecretService, EncryptionType};
use error::ProfileError;

pub struct Profile {
    pub(crate) username: String,
    pub(crate) password_type: String,
    pub(crate) imap_server: String,
    pub(crate) email: String,
    pub(crate) editor: String,
    pub(crate) editor_arguments: Vec<String>,
    pub(crate) secret_service_attribute: Option<String>,
    pub(crate) secret_service_value: Option<String>,
    #[allow(dead_code)]
    pub(crate) domain: String,

    pub(crate) password: Option<String>,
}

impl Profile {
    pub fn get_password(&self) -> String {
        if self.password_type == "PLAIN" {
            self.password.as_ref().unwrap().clone()
        } else {
            self.secret_service_get_pw()
        }
    }

    fn secret_service_get_pw(&self) -> String {
        let ss = SecretService::new(EncryptionType::Dh).unwrap();
        let collection = ss.get_default_collection().unwrap();
        collection.unlock().unwrap();

        let attribute = self.secret_service_attribute.as_ref().unwrap();
        let value = self.secret_service_value.as_ref().unwrap();

        let pw = collection.search_items(
            vec![(&attribute, &value)])
            .unwrap()
            .first()
            .unwrap()
            .get_secret()
            .unwrap();

        return str::from_utf8(&pw).unwrap().to_string();
    }
}

#[cfg(target_family = "unix")]
pub(crate)  fn get_config_path() -> Result<PathBuf> {
        let xdg_dir = BaseDirectories::new()?;
        match xdg_dir.find_config_file("apnotes/config") {
            Some(path) => Ok(path),
            None => {
                warn!("Could not detect config file, gonna create empty one");
                let mut path = xdg_dir.create_config_directory("apnotes")?;
                path.push("config");
                File::create(&path).expect("Unable to create file");
                Ok(path.to_path_buf())
            }
        }
}

#[cfg(target_family = "windows")]
pub(crate)  fn get_config_path() -> Result<PathBuf> {
    let config_file_path = PathBuf::from(format!("{}\\{}",env!("APPDATA"),"apnotes\\config".to_string()));
    if config_file_path.exists() {
        Ok(config_file_path)
    } else {
        warn!("Could not detect config file, gonna create empty one");
        std::fs::create_dir(&config_file_path.parent().unwrap())?;
        File::create(&config_file_path)?;
        Ok(config_file_path)
    }
}

#[cfg(target_family = "unix")]
pub(crate)  fn get_db_path() -> PathBuf {
    let xdg_dir = BaseDirectories::new().expect("Could not find xdg dirs");
    #[cfg(test)]
        let db = "notes_db_test";
    #[cfg(not(test))]
        let db = "notes_db";
    match xdg_dir.find_data_file(format!("apnotes/{}",db)) {
        Some(path) => path,
        None => {
            warn!("Could not detect database, gonna create empty one");
            let mut path = xdg_dir.create_data_directory("apnotes").expect("Could not create apple_notes config folder");
            path.push(&db);
            File::create(&path).expect("Unable to create file");
            path.to_path_buf()
        }
    }
}

#[cfg(target_family = "windows")]
pub(crate)  fn get_db_path() -> PathBuf {
    let db_file_path = PathBuf::from(format!("{}\\{}", env!("APPDATA"), "apnotes\\db".to_string()));
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

    trace!("Read config file from {}", &path);
    let creds = fs::read_to_string(&path)?;

    let username_regex = Regex::new(r"username=(.*)")?;
    let password_regex = Regex::new(r"password=(.*)")?;
    let imap_regex = Regex::new(r"imap_server=(.*)")?;
    let email_regex = Regex::new(r"email=(.*)")?;
    let editor_regex = Regex::new(r"editor=(.*)")?;
    let args_regex = Regex::new(r"editor_arguments=(.*)")?;
    let uuid_regex = Regex::new(r".*@(.*)")?;
    let password_type_regex = Regex::new(r"password_type=(.*)")?;

    let username = get_with_regex(username_regex, &creds)?;
    let password = get_with_regex(password_regex, &creds).map(|e| Some(e)).or_else::<ProfileError,_>(|_| Ok(None))?;
    let imap_server = get_with_regex(imap_regex, &creds)?;
    let email = get_with_regex(email_regex, &creds)?;
    let editor = get_with_regex(editor_regex, &creds)?;
    let args = get_with_regex(args_regex, &creds)?.split(" ").map(|s| s.to_string()).filter(|s| s.len() > 0).collect();
    let domain = get_with_regex(uuid_regex, &email)?;
    let password_type = get_with_regex(password_type_regex, &creds).or_else::<ProfileError,_>(|_| Ok("PLAIN".to_string()))?;

    let (secret_service_attribute, secret_service_value) = if password_type == "SECRET_SERVICE".to_string() {
        let secret_service_attribute_regex = Regex::new(r"secret_service_attribute=(.*)")?;
        let secret_service_value_regex = Regex::new(r"secret_service_value=(.*)")?;
        (
            Some(get_with_regex(secret_service_attribute_regex, &creds)?),
            Some(get_with_regex(secret_service_value_regex, &creds)?),
        )
    } else {
        (None, None)
    };

    Ok(
        Profile {
            username,
            password,
            password_type,
            imap_server,
            email,
            editor,
            editor_arguments: args,
            secret_service_attribute,
            secret_service_value,
            domain
        }
    )
}

fn get_with_regex(regex: Regex, creds: &str) -> Result<String> {
    match regex.captures(&creds)
        .and_then(|captured| captured.get(1))
        .and_then(|result| Option::from(result.as_str().to_string())) {
        Some(e) => Ok(e),
        None => {
            let config_entry_name = regex.to_string().replace("=(.*)","");
            Err(
                NotFound(
                    format!("Could not find entry in config file for key: \"{}\"", config_entry_name)
                ).into()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use secret_service::{SecretService, EncryptionType};
    use std::str;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn init_secret_service() {
        // initialize secret service (dbus connection and encryption session)
        let ss = SecretService::new(EncryptionType::Dh).unwrap();

        // get default collection
        let collection = ss.get_default_collection().unwrap();
        collection.unlock().unwrap();
        let pw = collection.search_items(vec![("mail", "uberspace")]).unwrap().first().unwrap().get_secret().unwrap();
        let ud = str::from_utf8(&pw);
        println!("{:?}", ud );
    }
}