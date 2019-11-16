extern crate regex;
extern crate imap;
extern crate native_tls;

use std::io::{stdout, Write, Read};
use std::{fs, io};

struct Collector(Vec<u8>);

use curl::easy::{Easy, Easy2, WriteError};
use self::regex::Regex;
use std::ops::{DerefMut, Deref};
use std::borrow::Borrow;
use self::imap::Session;
use std::net::TcpStream;
use self::native_tls::TlsStream;
use self::imap::types::{ZeroCopy, Name, Fetch};
use std::str::from_utf8;
use std::error::Error;

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
    let mut imap_session = client
        .login(username, password)
        .map_err(|e| e.0);

    return imap_session.unwrap();
}

fn fetch_inbox_top() -> imap::error::Result<Option<String>> {

    let mut imap_session = login();

    // we want to fetch the first email in the INBOX mailbox

    let count =
        imap_session.list(None, None).iter().next().iter().count();

    let mailbox = imap_session.examine("Notes").unwrap();

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
        let subject = subject_rgex.captures(header.as_str()).unwrap().get(1).unwrap().as_str();
        println!("{}", header);
    });


    // be nice to the server and log out
    imap_session.logout()?;

    Ok(Some("ddd".to_string()))
}

fn get_messages_from_foldersession(session: &mut Session<TlsStream<TcpStream>>, folderName: String) -> Vec<String> {
    session.select(folderName);
    let messages_result = session.fetch("1:*", "RFC822.HEADER");
    let xd = match messages_result {
        Ok(m) => {
            let dd: ZeroCopy<Vec<Fetch>> = m;
            get_headers(dd).unwrap()
        },
        _ => Vec::new()
    };
    xd
}

fn get_headers(fetch_vector: ZeroCopy<Vec<Fetch>>) -> io::Result<Vec<String>> {
    let results: Vec<io::Result<String>> = fetch_vector.iter().map( |fetch| get_header(fetch)).collect();
    let errors = results.iter().filter(|e| e.is_err()).count() > 0;
    let strings = results.into_iter().map(|e| e.unwrap()).collect();
    if errors {
        return Err(io::Error::from_raw_os_error(1))
    } else {
        Ok(strings)
    }
}

fn get_header(fetch: &Fetch) -> io::Result<String> {
    let result = fetch.header();
    if result.is_none() {
        return Err(io::Error::from_raw_os_error(1))
    }
    let res = std::string::String::from_utf8_lossy(result.unwrap()).to_string();
    Ok(res)
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

    #[test]
    fn login() {
        let mut session = imap::login();
        println!("MEEEEEM");
        let folders = imap::list_note_folders(&mut session);
        let foldername = folders.iter().last().unwrap().to_string();
        let messages = imap::get_messages_from_foldersession(&mut session, foldername);
        println!("{:#?}", messages);
    }

}

