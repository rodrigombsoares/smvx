use std::fs;
use std::path::Path;

use pest::Parser;
use pest::error::Error;
use pest_derive::Parser;

use crate::ast::{Program, Chunk, ListDef, ListIdx, ListAppend, DynConstructor, ArgExpr, AppendElement};


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SMVXParser;

pub fn parse_program<P: AsRef<Path>>(file_path: P) -> Result<Program, Error<Rule>> {
    let unparsed_file = fs::read_to_string(file_path).expect("cannot read file");

    let mut pairs = SMVXParser::parse(Rule::program, &unparsed_file)?;
    // debug the pest tree
    // println!("{:#?}", pairs);
    
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
            Rule::dyn_constructor => {
                chunks.push(Chunk::DynConstructor(parse_dyn_constructor(chunk_pair)?));
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

pub fn parse_list_append(pair: pest::iterators::Pair<Rule>) -> Result<ListAppend, pest::error::Error<Rule>> {
    let mut inner = pair.into_inner();
    
    // 1. Get the list name (e.g., "servers")
    let name = inner.next().unwrap().as_str().to_string();
    
    // 2. Pass the append_element Pair to our helper
    let element_pair = inner.next().unwrap(); 
    let element = parse_append_element(element_pair)?;

    Ok(ListAppend { name, element })
}

fn parse_append_element(pair: pest::iterators::Pair<Rule>) -> Result<AppendElement, pest::error::Error<Rule>> {
    let mut inner = pair.into_inner();
    let first = inner.peek().unwrap();

    // Check if it's a DynConstructor or just a list of ArgExprs
    if first.as_rule() == Rule::dyn_constructor {
        let dyn_const = parse_dyn_constructor(inner.next().unwrap())?;
        Ok(AppendElement::DynConstructor(dyn_const))
    } else {
        let mut exprs = Vec::new();
        for p in inner {
            exprs.push(parse_arg_expr(p)?);
        }
        Ok(AppendElement::Exprs(exprs))
    }
}

fn parse_dyn_constructor(pair: pest::iterators::Pair<Rule>) -> Result<DynConstructor, pest::error::Error<Rule>> {
    let mut inner = pair.into_inner();
    
    // First element is the constructor_name
    let name = inner.next().unwrap().as_str().to_string();
    
    // The rest are arg_exprs (list_indexing, paren_expr, or arg_chunk)
    let mut args = Vec::new();
    for p in inner {
        args.push(parse_arg_expr(p)?);
    }
    
    Ok(DynConstructor { name, args })
}

fn parse_arg_expr(pair: pest::iterators::Pair<Rule>) -> Result<ArgExpr, pest::error::Error<Rule>> {
    // Because arg_expr is silent (_), pair.as_rule() will be one of its inner variants
    match pair.as_rule() {
        Rule::list_indexing => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string(); // list_name
            let idx = inner.next().unwrap().as_str().parse::<usize>().unwrap();  // list_idx (e.g. "0" or "i-1")
            
            Ok(ArgExpr::ListIdx(ListIdx { name, idx }))
        }
        Rule::paren_expr => {
            // Recursively parse arguments inside parentheses
            let mut exprs = Vec::new();
            for p in pair.into_inner() {
                exprs.push(parse_arg_expr(p)?);
            }
            Ok(ArgExpr::Paren(exprs))
        }
        Rule::arg_chunk => {
            // Raw text like ".active"
            Ok(ArgExpr::Chunk(pair.as_str().to_string()))
        }
        _ => unreachable!("Unexpected rule in arg_expr: {:?}", pair.as_rule()),
    }
}
