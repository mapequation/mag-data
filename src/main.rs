#![feature(str_split_once)]

use std::error::Error;

mod fields;
mod papers;
mod journals;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
