#![warn(missing_docs)]
#![feature(pattern)]

//! Alternative to `cd`. Navigate by typing abbreviations of paths.

#[macro_use]
mod utils;
mod abbr;
mod error;
mod query;

#[allow(missing_docs)]
fn main() {
    let abbr = std::env::args_os().nth(1).unwrap();

    dbg!(query::query(abbr).unwrap());
}
