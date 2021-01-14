extern crate regex;

use uuid::Uuid;
use regex::Regex;

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string().to_uppercase()
}


pub fn is_uuid(string: &str) -> bool {
    let uuid_regex: Regex =
        Regex::new(r"\b[0-9A-F]{8}\b-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-\b[0-9A-F]{12}\b").unwrap();
    uuid_regex.is_match(string)
}

pub fn filter_none<S>(e: Option<S>) -> Option<S> {
    if e.is_some() {
        e
    } else {
        None
    }
}
