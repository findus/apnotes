extern crate man;
extern crate apnotes_bin;

use man::prelude::*;
use man::Manual;
use apnotes_bin::app;
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_generate::{generate, Generator};

fn main() {

    let app = app::app::gen_app();
    

    let page = Manual::new("apnotes")
        .about("Short introduction")
        .author(Author::new("Philipp Hentschel").email("philipp[at]f1ndus[dot]de"))
        .flag(
            Flag::new()
                .short("-d")
                .long("--debug")
                .help("Enable debug mode"),
        )
        .flag(
            Flag::new()
                .short("-v")
                .long("--verbose")
                .help("Enable verbose mode"),
        )
        .option(
            Opt::new("output")
                .short("-o")
                .long("--output")
                .help("The file path to write output to"),
        )
        .example(
            Example::new()
                .text("run basic in debug mode")
                .command("basic -d")
                .output("Debug Mode: basic will print errors to the console")
        )
        .custom(
            Section::new("usage note")
                .paragraph("This program will overwrite any file currently stored at the output path")
        )
        .render();

    println!("{}", page);


}