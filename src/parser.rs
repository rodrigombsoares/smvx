use std::fs;
use std::path::Path;

use pest::Parser;
use pest::error::Error;
use pest_derive::Parser;

use crate::ast::{Program, Chunk, ListDef, ListIdx, ListAppend};


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SMVXParser;

pub fn parse_program<P: AsRef<Path>>(file_path: P) -> Result<Program, Error<Rule>> {
    let unparsed_file = fs::read_to_string(file_path).expect("cannot read file");

    let mut pairs = SMVXParser::parse(Rule::program, &unparsed_file)?;
    // debug the pest tree
    println!("{:#?}", pairs);
    
    let mut chunks = Vec::new();
    let program_pair = pairs.next().unwrap();
    for chunk_pair in program_pair.into_inner() {
        match chunk_pair.as_rule() {
            Rule::list => {
                chunks.push(Chunk::ListDef(parse_list(chunk_pair)?));
            }
            Rule::list_indexing => {
                chunks.push(Chunk::ListIdx(parse_list_indexing(chunk_pair)?));
            }
            Rule::list_append => {
                chunks.push(Chunk::ListAppend(parse_list_append(chunk_pair)?));
            }
            Rule::raw_nusmv_chunk => {
                chunks.push(Chunk::RawNusmv(chunk_pair.as_str().to_string()));
            }
            _ => {}
        }
    }

    // debug ast
    println!("{:#?}", chunks);

    Ok(Program { chunks })
}

fn parse_list(pair: pest::iterators::Pair<Rule>) -> Result<ListDef, Error<Rule>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    
    let element_pairs = inner.next().unwrap().into_inner();
    let elements = element_pairs.map(|e| e.as_str().to_string()).collect();

    Ok(ListDef { name, elements })
}

fn parse_list_indexing(pair: pest::iterators::Pair<Rule>) -> Result<ListIdx, Error<Rule>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let idx = inner.next().unwrap().as_str().parse::<usize>().unwrap();

    Ok(ListIdx { name, idx })
}

fn parse_list_append(pair: pest::iterators::Pair<Rule>) -> Result<ListAppend, Error<Rule>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let element = inner.next().unwrap().as_str().to_string();

    Ok(ListAppend { name, element })
}