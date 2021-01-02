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

/**
From:
X-Uniform-Type-Identifier
Content-Type

**/

pub struct HeaderBuilder {
     headers: Vec<(String,String)>
}

impl HeaderBuilder {

    pub fn new() -> HeaderBuilder {
        let mut headers: Vec<(String,String)> = vec![];
        let profile = self::profile::load_profile();
        headers.push(("X-Uniform-Type-Identifier".to_string(), "com.apple.mail-note".to_string()));
        headers.push(("Content-Type".to_string(), "text/html; charset=utf-8".to_string()));
        headers.push(("Content-Transfer-Encoding".to_string(), "quoted-printable".to_string()));
        headers.push(("Mime-Version".to_string(), "1.0 (Mac OS X Notes 4.6 \\(879.10\\))".to_string()));
        let date = Utc::now().to_rfc2822();
        headers.push(("Date".to_string(), date.clone()));
        headers.push(("X-Mail-Created-Date".to_string(), date.clone()));
        headers.push(("From".to_string(), profile.email));

        HeaderBuilder {
            headers
        }
    }

    pub fn with_message_id(mut self, message_id: String) -> Self {
        let profile = self::profile::load_profile();
        self.headers.push(("Message-Id".to_string(), format!("<{}@{}", message_id, profile.domain())));
        self
    }

    pub fn with_uuid(mut self, uuid: String) -> Self {
        self.headers.push(("X-Universally-Unique-Identifier".to_string(), uuid));
        self
    }

    pub fn with_subject(mut self, subject: String) -> Self {
        self.headers.push(("Subject".to_string(), subject));
        self
    }

    pub fn build(mut self) -> NoteHeaders {
        let profile = self::profile::load_profile();

        if None == self.headers.get_header_value("X-Universally-Unique-Identifier") {
            self.headers.push(("X-Universally-Unique-Identifier".to_string(), generate_uuid()));
        }

        if None == self.headers.get_header_value("Message-Id") {
            self.headers.push(("Message-Id".to_string(), format!("<{}@{}", generate_uuid(), profile.domain())));
        }

        self.headers
    }
}

pub fn generate_mail_headers(subject: String) -> Vec<(String,String)> {
    HeaderBuilder::new().with_subject(subject).build()
}