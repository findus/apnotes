extern crate subprocess;
extern crate apple_notes_rs;
extern crate regex;
extern crate log;
extern crate uuid;


use std::env;
use std::fs::File;
use apple_notes_rs::note::{NotesMetadata, HeaderParser};

use self::regex::Regex;
use apple_notes_rs::io;
use log::info;
use apple_notes_rs::util;

pub fn main() {
    let args: Vec<String> = env::args().collect();

    let file = args.get(1).unwrap();
    match subprocess::Exec::cmd("nvim").arg(file).join() {
        Ok(_) => update(file),
        Err(d) => panic!("{}", d.to_string())
    }

    println!("Ayy lmao")

}

fn update(file: &String) {
    info!("Update Message_Id for {}", &file);
    let path = std::path::Path::new(file).to_owned();
    let metadata_file_path = util::get_hash_path(&path);

    let metadata_file = File::open(metadata_file_path).unwrap();
    let metadata: NotesMetadata = serde_json::from_reader(metadata_file).unwrap();

    let metadata_identifier = metadata.message_id();
    let mut new_metadata_headers: Vec<(String,String)> = metadata.header
        .into_iter()
        .filter(|(a,_)| a != "Message-Id")
        .collect();

    if metadata.old_remote_id.is_none() {
        info!("File is changed the for the first time, gonna change uuid");
        let new_uuid_str = replace_uuid(&metadata_identifier);
        new_metadata_headers.push(("Message-Id".to_owned(), new_uuid_str.clone()));

        let new_metadata = NotesMetadata {
            header: new_metadata_headers,
            old_remote_id: Some(metadata_identifier.clone()),
            subfolder: metadata.subfolder,
            locally_deleted: false,
            //TODO get new one
            uid: metadata.uid
        };

        io::save_metadata_to_file(&new_metadata);
    }

}

fn replace_uuid(string: &str) -> String {
    let uuid_regex = Regex::new(r"(.*<)\b[0-9A-F]{8}\b-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-\b[0-9A-F]{12}\b(.*)").unwrap();
    let new_uuid = uuid::Uuid::new_v4().to_string().to_uppercase();
    let dd = format!("${{1}}{}${{2}}",new_uuid);
    uuid_regex.replace(string, dd.as_str()).to_string()
}

#[test]
fn it_works() {
    println!("{}",replace_uuid("Message-Id: <7A41875C-2CCF-4AE4-869E-1F230E1B71BA@f1ndus.de>"));
}