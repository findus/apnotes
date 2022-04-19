#[cfg(target_family = "unix")]
extern crate xdg;

extern crate regex;
extern crate log;

use std::collections::HashMap;
use self::regex::Regex;
use std::fs::File;
use self::log::{warn};
use std::path::PathBuf;
use error::ProfileError::*;
use std::str;
use error::Result;

use error::ProfileError;

#[cfg(target_family = "unix")]
use self::xdg::BaseDirectories;

#[cfg(target_family = "unix")]
use secret_service::{SecretService, EncryptionType};


#[derive(Debug)]
pub struct Profile {
    pub(crate) username: String,
    pub(crate) password_type: String,
    pub(crate) imap_server: String,
    pub(crate) email: String,
    pub(crate) editor: String,
    pub(crate) editor_arguments: Vec<String>,
    #[allow(dead_code)]
    pub(crate) secret_service_attribute: Option<String>,
    #[allow(dead_code)]
    pub(crate) secret_service_value: Option<String>,
    #[allow(dead_code)]
    pub(crate) domain: String,

    pub(crate) password: Option<String>,
}

impl Profile {

    #[cfg(target_family = "unix")]
    pub fn get_password(&self) -> Result<String> {
        if self.password_type == "PLAIN" {
            Ok(self.password.as_ref().unwrap().clone())
        } else {
            self.secret_service_get_pw()
        }
    }

    #[cfg(target_family = "windows")]
    pub fn get_password(&self) -> Result<String> {
        if self.password_type == "PLAIN" {
            Ok(self.password.as_ref().unwrap().clone())
        } else {
            panic!("Password type {} not supported", self.password_type)
        }
    }

    #[cfg(target_family = "unix")]
    fn secret_service_get_pw(&self) -> Result<String> {
        let ss = SecretService::new(EncryptionType::Dh)?;

        let collection = ss.get_default_collection()?;

        if collection.ensure_unlocked().is_err() {
            return Err(ProfileError::AgentLocked().into());
        }

        let attribute = self.secret_service_attribute.as_ref().unwrap();
        let value = self.secret_service_value.as_ref().unwrap();

        let map =HashMap::from([(attribute.as_str(), value.as_str())]);


        let tuple_vec = HashMap::from([(attribute, value)]);

        let entries = collection.search_items(
            map)
            .unwrap();

        let entry = entries.first().unwrap();

        let attributes = entry.get_attributes().unwrap();

        let entry = entry.unlock().and_then(|_| entry.get_secret()).unwrap();

        return Ok(str::from_utf8(&entry)?.to_string());
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
    #[cfg(not(test))]
    let creds = std::fs::read_to_string(&path)?;
    #[cfg(test)]
    let creds = unsafe { get_test_config() };

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
        if password.is_none() {
            return Err(NoPasswordProvided().into())
        }
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
static mut BASIC_SECRET_SERVICE_CONFIG: &'static str = ""
;

#[cfg(test)]
unsafe fn get_test_config() -> &'static str {
    return BASIC_SECRET_SERVICE_CONFIG
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use profile::{load_profile, BASIC_SECRET_SERVICE_CONFIG};
    #[cfg(target_family = "unix")]
    use secret_service::{SecretService, EncryptionType};

    #[cfg(target_family = "unix")]
    #[ignore]
    #[test]
    fn test_secret_service() {
        let ss = SecretService::new(EncryptionType::Dh).unwrap();
        let collections = ss.get_all_collections().unwrap();
        let size = collections.len();
        let collection = ss.get_default_collection().unwrap();
        //collection.unlock().unwrap();

        collection.ensure_unlocked();

        if collection.is_locked().unwrap() {
            return
        }

        let attribute = "mail";
        let value = "uberspace";

        let tuple_vec = HashMap::from([(attribute, value)]);

        let entries = collection.search_items(
            tuple_vec)
            .unwrap();

        let entry = entries.first().unwrap();

        let attributes = entry.get_attributes().unwrap();

        let entry = entry.unlock().and_then(|_| entry.get_secret()).unwrap();

        println!("test");


    }

    #[test]
    fn test_plain_config() {
        unsafe {
            BASIC_SECRET_SERVICE_CONFIG = "
                username=test@test.de
                imap_server=test.test.de
                email=test@test.de
                editor=nvim-float
                editor_arguments=
                password_type=PLAIN
                password=f
                ";

            let profile = load_profile();
            assert_eq!(profile.as_ref().unwrap().password_type,"PLAIN");
        }
    }

    #[test]
    fn test_no_password_provided() {
        unsafe {
            BASIC_SECRET_SERVICE_CONFIG = "
                username=test@test.de
                imap_server=test.test.de
                email=test@test.de
                editor=nvim-float
                editor_arguments=
                password_type=PLAIN
                ";

            assert!(load_profile().err().is_some());
        }
    }

    #[test]
    fn test_secret_service_config() {
        unsafe {
            BASIC_SECRET_SERVICE_CONFIG = "
                username=test@test.de
                imap_server=test.test.de
                email=test@test.de
                editor=nvim-float
                editor_arguments=
                password_type=SECRET_SERVICE
                secret_service_attribute=mail
                secret_service_value=mailservice
                ";

            let profile = load_profile();
            assert_eq!(profile.as_ref().unwrap().secret_service_value.as_ref().unwrap(),"mailservice");
            assert_eq!(profile.as_ref().unwrap().secret_service_attribute.as_ref().unwrap(),"mail");
            assert_eq!(profile.as_ref().unwrap().password_type,"SECRET_SERVICE");
        }

    }
}