#![feature(str_split_once)]

extern crate clap;

use std::error::Error;
use std::path::Path;

use clap::{App, Arg};

use crate::filter_papers::filter;
use crate::journals_wikidata_fields::read_wikidata;
use crate::validate_papers::parse;

mod filter_papers;
mod journals_wikidata_fields;
mod parse_journals;
mod types;
mod validate_papers;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Parse mag data")
        .version("1.0")
        .subcommand(
            App::new("validate")
                .arg(
                    Arg::new("INPUT")
                        .value_name("INPUT")
                        .required(true)
                        .index(1)
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUTPUT")
                        .value_name("OUTPUT")
                        .required(true)
                        .index(2)
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("join")
                .arg(
                    Arg::new("NT_FILE")
                        .value_name("NT_FILE")
                        .required(true)
                        .index(1)
                        .takes_value(true),
                )
                .arg(
                    Arg::new("CSV_FILE")
                        .value_name("CSV_FILE")
                        .required(true)
                        .index(2)
                        .takes_value(true),
                )
                .arg(
                    Arg::new("OUTPUT")
                        .value_name("OUTPUT")
                        .required(true)
                        .index(3)
                        .takes_value(true),
                ),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("validate") => {
            let sub = matches.subcommand_matches("validate").unwrap();

            let input = Path::new(sub.value_of("INPUT").unwrap());
            let output = Path::new(sub.value_of("OUTPUT").unwrap());

            parse(input, output)?;
        }
        Some("join") => {
            let sub = matches.subcommand_matches("join").unwrap();

            let csv_file = Path::new(sub.value_of("CSV_FILE").unwrap());
            let nt_file = Path::new(sub.value_of("NT_FILE").unwrap());
            let output = Path::new(sub.value_of("OUTPUT").unwrap());

            let journals = read_wikidata(csv_file)?;

            filter(nt_file, output, journals)?;
        }
        None => eprintln!("No subcommand given!"),
        _ => {}
    }

    Ok(())
}
