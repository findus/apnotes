extern crate pulldown_cmark;

use note::{NotesMetadata, HeaderParser};
use profile;
use std::path::Path;
use self::pulldown_cmark::{html, Parser};
use std::io;

pub fn convert2md(input: &String) -> String {
    html2runes::markdown::convert_string(input.as_str())
}

pub fn convert2Html(input: &NotesMetadata) -> String {
    let path = format!("{}/{}/{}", profile::get_notes_dir(), input.subfolder, input.subject_with_identifier());
    let path = Path::new(&path);
    let text = std::fs::read_to_string(path).unwrap();


    let mut input = text;

    let mut parser = Parser::new(&input);
    let mut html_output: String = String::new();
    html::push_html(&mut html_output, parser);
    println!("{}",html_output);
    html_output
}


