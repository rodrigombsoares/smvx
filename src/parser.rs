use std::fs;
use std::path::Path;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SMVXParser;

pub fn parse_smvx<P: AsRef<Path>>(file_path: P) {
    let unparsed_file = fs::read_to_string(file_path).expect("cannot read file");

    let file = SMVXParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    // print
    println!("{:#?}", file);
}