extern crate mailparse;
extern crate html2runes;




pub trait NoteTrait {
    fn subject(&self) -> String;
}

pub struct Note {
    pub mail_headers: Vec<(String, String)>,
    pub body: String,
    pub hash: u64,
    pub uid: u32
}

impl NoteTrait for Note {
    fn subject(&self) -> String {
        let subject = match self.mail_headers.iter().find(|(x, _y)| x.eq("Subject")) {
            Some((_subject, name)) => format!("{}-{}", self.uid, name),
            _ => "<no subject>".to_string()
        };
        subject
    }
}