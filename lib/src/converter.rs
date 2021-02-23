extern crate pulldown_cmark;

use model::Body;
use self::pulldown_cmark::{Parser, html};

pub fn convert2md(input: &String) -> String {
    html2runes::markdown::convert_string(input.as_str())
}

pub fn convert_to_html(input: &Body) -> String {
    let parser = Parser::new(&input.text.as_ref().expect("Expected body with message"));
    let mut html_output: String = String::new();
    html::push_html(&mut html_output, parser);
    quoted_printable::encode_to_str(html_output)
}
