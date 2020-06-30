extern crate apple_notes_rs;
extern crate log;
extern crate fasthash;


use self::fasthash::{metro};
use std::env;
use std::path::PathBuf;


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let path = PathBuf::from(args.get(1).unwrap());
    println!("Calc Hash for {}", path.to_str().unwrap());
    let s = std::fs::read_to_string(path).unwrap();
    println!("{}", s);
    let d = metro::hash64(s);
    println!("{}", d);
}