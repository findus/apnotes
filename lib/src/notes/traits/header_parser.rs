pub trait HeaderParser {
    fn get_header_value(&self, search_string: &str) -> Option<String>;
    fn subject(&self) -> String;
    fn uuid(&self) -> String;
    fn subject_escaped(&self) -> String;
    fn message_id(&self) -> String;
    fn date(&self) -> String;
    fn mime_version(&self) -> String;
    fn folder(&self) -> String;
    fn imap_uid(&self) -> i64;
}