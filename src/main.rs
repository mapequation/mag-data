#![feature(str_split_once)]

extern crate clap;

use std::error::Error;
use std::path::Path;

use clap::{App, Arg};

use crate::paper_parser::parse;

mod fields;
mod journals;
mod paper_parser;
mod papers;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Parse mag data")
        .version("1.0")
        .arg(
            Arg::new("INPUT")
                .value_name("FILE")
                .required(true)
                .index(1)
                .takes_value(true),
        )
        .arg(
            Arg::new("OUTPUT")
                .value_name("FILE")
                .required(true)
                .index(2)
                .takes_value(true),
        )
        .get_matches();

    let input = Path::new(matches.value_of("INPUT").unwrap());
    let output = Path::new(matches.value_of("OUTPUT").unwrap());

    parse(input, output).unwrap();

    Ok(())
}
