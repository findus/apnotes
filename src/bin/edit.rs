use std::process::Command;
use std::env;

extern crate subprocess;

pub fn main() {
    let args: Vec<String> = env::args().collect();

    let file = args.get(1).unwrap();
    match subprocess::Exec::cmd("nvim").arg(file).join() {
        Ok(_) => println!("Noice"),
        Err(d) => panic!("{}", d.to_string())
    }

    println!("Ayy lmao")

}