extern crate pulldown_cmark;

use self::pulldown_cmark::{html, Parser};
use util;
use model::NotesMetadata;

pub fn convert2md(input: &String) -> String {
    html2runes::markdown::convert_string(input.as_str())
}

pub fn convert_to_html(input: &NotesMetadata) -> String {
    let path = util::get_notes_file_path_from_metadata(input);
    let text = std::fs::read_to_string(path).unwrap();

    let input = text;

    let parser = Parser::new(&input);
    let mut html_output: String = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}


