extern crate regex;
extern crate imap;
extern crate native_tls;

use std::io::{stdout, Write};
use std::fs;

struct Collector(Vec<u8>);

use curl::easy::{Easy, Easy2, Handler, WriteError};
use self::regex::Regex;
use std::ops::{DerefMut, Deref};
use std::borrow::Borrow;
use self::imap::Session;
use std::net::TcpStream;
use self::native_tls::TlsStream;
use self::imap::types::{ZeroCopy, Name};

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

// Print a web page onto stdout
pub fn hello_world_curl() {
    let mut easy = Easy::new();
    let domain = "https://www.rust-lang.org/";
    easy.url(domain).unwrap();
    easy.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    }).unwrap();
    easy.perform().unwrap();

    println!("{}", easy.response_code().unwrap());
}

pub fn imap_framework_test() {

/*
    let creds = fs::read_to_string("./cred").expect("error");

    let usernameRegex = Regex::new(r"^username=(.*)").unwrap();
    let passwordRegex = Regex::new(r"password=(.*)").unwrap();

    let username = usernameRegex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();
    let password = passwordRegex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();

    let domain = "imaps://imap.ankaa.uberspace.de";
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login(username, password)
        .map_err(|e| e.0)?;

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages = imap_session.fetch("1", "RFC822")?;
    let message = if let Some(m) = messages.iter().next() {
        m
    } else {
        return Ok(None);
    };

    // extract the message's body
    let body = message.body().expect("message did not have a body!");
    let body = std::str::from_utf8(body)
        .expect("message was not valid utf-8")
        .to_string();

    // be nice to the server and log out
    imap_session.logout()?;
*/


}

pub fn imap_list_notes() {

    let creds = fs::read_to_string("./cred").expect("error");

    let username_regex = Regex::new(r"^username=(.*)").unwrap();
    let password_regex = Regex::new(r"password=(.*)").unwrap();

    let username = username_regex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();
    let password = password_regex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();

    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.url("imaps://imap.ankaa.uberspace.de/Notes").unwrap();
    easy.username(username).unwrap();
    easy.password(password).unwrap();
    easy.port(993).unwrap();
    easy.perform().unwrap();
}

fn login() -> Session<TlsStream<TcpStream>> {
    let creds = fs::read_to_string("./cred").expect("error");

    let username_regex = Regex::new(r"^username=(.*)").unwrap();
    let password_regex = Regex::new(r"password=(.*)").unwrap();

    let username = username_regex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();
    let password = password_regex.captures(creds.as_str()).unwrap().get(1).unwrap().as_str();

    let domain = "imap.ankaa.uberspace.de";
    let tls = native_tls::TlsConnector::builder().build().unwrap();

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

fn list_note_folders() -> Vec<String> {
    let mut imap = login();
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
        //imap::hello_world_curl();
        println!("{:#?}",imap::list_note_folders());
    }

}

