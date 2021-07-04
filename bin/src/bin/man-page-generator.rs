extern crate man;
extern crate apnotes_bin;
extern crate clap;
extern crate clap_generate;

use man::prelude::*;
use man::Manual;

fn main() {

  //  generate::<Zsh, _>(&mut app, "apnotes", &mut io::stdout());

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
                .text("list all present note subjects")
                .command("apnotes list --names")
                .output("Prints the subject name of all stored notes")
        )
        .custom(
            Section::new("usage note")
                .paragraph("This program will overwrite any file currently stored at the output path")
        )
        .render();

    println!("{}", page);


}