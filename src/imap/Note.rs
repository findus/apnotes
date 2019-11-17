extern crate mailparse;

use mailparse::{MailHeader, MailHeaderMap};

pub trait NoteTrait {
    fn subject(&self) -> String;
}

pub struct Note {
    pub mailHeaders: Vec<(String, String)>,
    pub body: String
}

//impl NoteTrait for Note {
//    fn subject(&self) -> String {
//        let subject = match self.mailHeaders.get_first_value("Subject") {
//            Ok(Some(subject)) => subject,
//            Ok(None) => "<no subject>".to_string(),
//            Err(e) => {
//                println!("failed to get message subject: {:?}", e);
//                "".to_string()
//            }
//        };
//        subject
//    }
//}