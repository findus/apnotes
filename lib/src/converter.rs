extern crate pulldown_cmark;

use crate::model::Body;
use self::pulldown_cmark::{Parser, html};

pub fn convert2md(input: &String) -> String {
    html2runes::markdown::convert_string(input.as_str())
}

pub fn convert_to_html(input: &Body) -> String {
    let content = input.text.as_ref().expect("Expected body with message");
    let content = htmlescape::encode_minimal(&content);
    let parser = Parser::new(&content);
    let mut html_output: String = String::new();
    html::push_html(&mut html_output, parser);
    let output = format!("{}{}{}",
            "<html><head></head><body style=\"word-wrap: break-word; -webkit-nbsp-mode: space; line-break: after-white-space;\">",
            html_output,
            "</body></html>")
        .replace("<ul>", "<ul class=\"Apple-dash-list\">");

    quoted_printable::encode_to_str(output).replace("=0A", "")
}
