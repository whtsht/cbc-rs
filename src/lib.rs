extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "scanner.pest"]
pub struct CBCScanner;

pub mod ir;
pub mod node;
pub mod resolve;
pub mod gen;
