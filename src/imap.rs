extern crate regex;
extern crate imap;
extern crate native_tls;
extern crate mailparse;


use std::{fs};

struct Collector(Vec<u8>);

use self::regex::Regex;
use self::imap::Session;
use std::net::TcpStream;
use self::native_tls::TlsStream;
use self::imap::types::{ZeroCopy, Fetch};
use mailparse::MailHeader;
use std::ptr::null;

mod Note;

fn login() -> Session<TlsStream<TcpStream>> {
    println!("{}",std::env::current_dir().unwrap().display());
    let creds = fs::read_to_string("cred").expect("error");

    let username_regex = Regex::new(r"^username=(.*)").unwrap();
    let password_regex = Regex::new(r"password=(.*)").unwrap();

    let username = username_regex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();
    let password = password_regex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();

    let domain = "localhost";
    let tls = native_tls::TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let imap_session = client
        .login(username, password)
        .map_err(|e| e.0);

    return imap_session.unwrap();
}

fn fetch_inbox_top() -> imap::error::Result<Option<String>> {

    let mut imap_session = login();

    // we want to fetch the first email in the INBOX mailbox

    let _count =
        imap_session.list(None, None).iter().next().iter().count();

    let _mailbox = imap_session.examine("Notes").unwrap();

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages = imap_session.fetch("1:*", "RFC822.HEADER")?;

    let folders = imap_session.list(None,None);

    println!("{}", folders.iter().count());

    folders.iter().for_each( |folder|  {
        folder.iter().for_each( |d| {
            println!("{}", d.name().to_string());
        })
    });

    let iterator = messages.iter();

    iterator.for_each( |message| {

        let subject_rgex = Regex::new(r"Subject:(.*)").unwrap();

        // extract the message's body
        let header = message.header().expect("message did not have a body!");
        let header = std::str::from_utf8(header)
            .expect("message was not valid utf-8")
            .to_string();
        let _subject = subject_rgex.captures(header.as_str()).unwrap().get(1).unwrap().as_str();
        println!("{}", header);
    });


    // be nice to the server and log out
    imap_session.logout()?;

    Ok(Some("ddd".to_string()))
}

fn get_messages_from_foldersession(session: &mut Session<TlsStream<TcpStream>>, folderName: String) -> Vec<Note::Note> {
    session.select(folderName);
    let messages_result = session.fetch("1:*", "RFC822.HEADER");
    let _xd = match messages_result {
        Ok(m) => {
            let dd: ZeroCopy<Vec<Fetch>> = m;
            get_notes(dd)
        },
        _ => Vec::new()
    };
    _xd
}

fn get_notes(fetch_vector: ZeroCopy<Vec<Fetch>>) -> Vec<Note::Note> {
    let d: Vec<Note::Note> = fetch_vector.into_iter().map(|fetch| {
        let f = fetch;
        let q = get_headers(f);
        let note = Note::Note {
            mailHeaders: q,
            body: "".to_string()
        };
        note
    }).collect();
    d
}
/**
Returns empty vector if something fails
*/
fn get_headers(fetch: &Fetch) -> Vec<(String, String)> {
    match mailparse::parse_headers(fetch.header().unwrap()) {
        Ok((header, _)) => header.into_iter().map( |h| (h.get_key().unwrap(), h.get_value().unwrap())).collect(),
        _ => Vec::new()
    }
}

fn list_note_folders(imap: &mut Session<TlsStream<TcpStream>>) -> Vec<String> {
    let folders_result = imap.list(None, Some("Notes*"));
     let result: Vec<String> = match folders_result {
        Ok(result) => {
            let names: Vec<String> = result.iter().map( |name| name.name().to_string()).collect();
            names
        }
        _ => Vec::new()
    };

    return result
}

#[cfg(test)]
mod tests {
    use imap;
    use imap::Note::NoteTrait;

    #[test]
    fn login() {
        let mut session = imap::login();
        println!("MEEEEEM");
        let folders = imap::list_note_folders(&mut session);
        let foldername = folders.iter().last().unwrap().to_string();
        let _messages = imap::get_messages_from_foldersession(&mut session, "Notes".to_string());
        _messages.iter().for_each(|b| println!("{}", b.subject()));
    }

}

