use std::process::{Command};
use std::env;

macro_rules! get(($name:expr) => (ok!(env::var($name))));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("Build man pages");

    build_man_pages();
}

fn build_man_pages() {
    if cfg!(target_os = "linux") {
        build_man_page("apnotes.1");
        build_man_page("apnotes.5");
    };

    fn build_man_page(name: &str) {
        if Command::new("bash")
            .arg("-c")
            .arg(format!("set -o pipefail ;scdoc < ../contrib/man/{}.scd | gzip > ../target/{}/{}.gz",name,get_profile(),name))
            .output()
            .expect("failed to execute process")
            .status.success() == false {
            panic!(format!("Could not create {}", name));
        };
    }

    fn get_profile() -> String {
        get!("PROFILE")
    }
}