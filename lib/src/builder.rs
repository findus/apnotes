use model::{Body, NotesMetadata};
use util::generate_uuid;
use chrono::Utc;
use notes::note_headers::NoteHeaders;
use notes::traits::header_parser::HeaderParser;
#[cfg(not(test))]
use profile::Profile;

pub struct BodyMetadataBuilder {
    body: Body
}
/// Builder for Body Objects, mostly for
/// testing purposes
///
/// If no own message-id gets provided it gets randomly
/// generated
impl BodyMetadataBuilder {

    #[cfg(not(test))]
    pub fn new(profile: &Profile) -> BodyMetadataBuilder {
        BodyMetadataBuilder {
            body: Body {
                old_remote_message_id: None,
                message_id: format!("<{}@{}>", generate_uuid(), &profile.domain()),
                text: None,
                uid: None,
                metadata_uuid: "".to_string()
            }
        }
    }

    #[cfg(test)]
    pub fn new() -> BodyMetadataBuilder {
        BodyMetadataBuilder {
            body: Body {
                old_remote_message_id: None,
                message_id: format!("<{}@{}>", generate_uuid(), "test@test.de".clone()),
                text: None,
                uid: None,
                metadata_uuid: "".to_string()
            }
        }
    }

    pub fn with_uid(mut self, uid: Option<i64>) -> Self {
        self.body.uid = uid;
        self
    }

    #[allow(dead_code)]
    pub fn with_metadata_uuid(mut self, uuid: &str) -> Self {
        self.body.metadata_uuid = uuid.to_string();
        self
    }

    pub fn with_message_id(mut self, message_id: &str) -> Self {
        self.body.message_id = message_id.to_string();
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.body.text = Some(text.to_string());
        self
    }

    pub fn with_old_remote_message_id(mut self, id: &str) -> Self {
        self.body.old_remote_message_id = Some(id.to_string());
        self
    }

    pub fn build(self) -> Body {
        self.body
    }
}

pub struct NotesMetadataBuilder {
    notes_metadata: NotesMetadata
}

/// Builder for Metadata Objects, mostly for
/// testing purposes
///
/// If no own uuid gets provided it gets randomly
/// generated
impl NotesMetadataBuilder {
    pub fn new() -> NotesMetadataBuilder {
        let date = Utc::now().to_rfc2822();
        NotesMetadataBuilder {
            notes_metadata:  NotesMetadata {
                subfolder: "".to_string(),
                locally_deleted: false,
                new: false,
                date,
                uuid: generate_uuid(),
                mime_version: "1.0 (Mac OS X Notes 4.6 \\(879.10\\))".to_string()
            }
        }

    }

    #[allow(dead_code)]
    pub fn with_uuid(mut self, uuid: &str) -> Self {
        self.notes_metadata.uuid = uuid.to_string();
        self
    }

    pub fn is_new(mut self, new: bool) -> Self {
        self.notes_metadata.new = new;
        self
    }

    #[allow(dead_code)]
    pub fn is_flagged_for_deletion(mut self, del: bool) -> Self {
        self.notes_metadata.locally_deleted = del;
        self
    }

    pub fn with_folder(mut self, folder: String) -> Self {
        self.notes_metadata.subfolder = folder;
        self
    }

    pub fn build(self) -> NotesMetadata {
        self.notes_metadata
    }

}

pub struct HeaderBuilder {
    headers: Vec<(String,String)>,
}

impl HeaderBuilder {

    #[cfg(not(test))]
    pub fn new(profile: &Profile) -> HeaderBuilder {
        let mut headers: Vec<(String,String)> = vec![];
        headers.push(("X-Uniform-Type-Identifier".to_string(), "com.apple.mail-note".to_string()));
        headers.push(("Content-Type".to_string(), "text/html; charset=utf-8".to_string()));
        headers.push(("Content-Transfer-Encoding".to_string(), "quoted-printable".to_string()));
        headers.push(("Mime-Version".to_string(), "1.0 (Mac OS X Notes 4.6 \\(879.10\\))".to_string()));
        let date = Utc::now().to_rfc2822();
        headers.push(("Date".to_string(), date.clone()));
        headers.push(("X-Mail-Created-Date".to_string(), date.clone()));
        headers.push(("From".to_string(), (&profile).email.to_string()));

        HeaderBuilder {
            headers
        }
    }

    #[cfg(test)]
    pub fn new() -> HeaderBuilder {
        let mut headers: Vec<(String,String)> = vec![];
        headers.push(("X-Uniform-Type-Identifier".to_string(), "com.apple.mail-note".to_string()));
        headers.push(("Content-Type".to_string(), "text/html; charset=utf-8".to_string()));
        headers.push(("Content-Transfer-Encoding".to_string(), "quoted-printable".to_string()));
        headers.push(("Mime-Version".to_string(), "1.0 (Mac OS X Notes 4.6 \\(879.10\\))".to_string()));
        let date = Utc::now().to_rfc2822();
        headers.push(("Date".to_string(), date.clone()));
        headers.push(("X-Mail-Created-Date".to_string(), date.clone()));
        headers.push(("From".to_string(), "test@test.de".to_string()));

        HeaderBuilder {
            headers
        }
    }

    //TODO reimplement message-id formatting somwhere else
    pub fn with_message_id(mut self, message_id: String) -> Self {
      //  let profile = self::profile::load_profile();
//        self.headers.push(("Message-Id".to_string(), format!("<{}@{}", message_id, profile.domain())));
        self.headers.push(("Message-Id".to_string(), message_id));
        self
    }

    pub fn with_uuid(mut self, uuid: String) -> Self {
        self.headers.push(("X-Universally-Unique-Identifier".to_string(), uuid));
        self
    }

    pub fn with_subject(mut self, subject: &str) -> Self {
        self.headers.push(("Subject".to_string(), subject.to_string()));
        self
    }

    #[cfg(not(test))]
    pub fn build(mut self, profile: &Profile) -> NoteHeaders {

        if None == self.headers.get_header_value("X-Universally-Unique-Identifier") {
            self.headers.push(("X-Universally-Unique-Identifier".to_string(), generate_uuid()));
        }

        if None == self.headers.get_header_value("Message-Id") {
            self.headers.push(("Message-Id".to_string(), format!("<{}@{}>", generate_uuid(), profile.domain())));
        }

        self.headers
    }

    #[cfg(test)]
    pub fn build(mut self) -> NoteHeaders {

        if None == self.headers.get_header_value("X-Universally-Unique-Identifier") {
            self.headers.push(("X-Universally-Unique-Identifier".to_string(), generate_uuid()));
        }

        if None == self.headers.get_header_value("Message-Id") {
            self.headers.push(("Message-Id".to_string(), format!("<{}@{}>", generate_uuid(), "test@test.de".to_string())));
        }

        self.headers
    }
}
