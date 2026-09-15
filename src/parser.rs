use std::fs;
use std::path::Path;

use pest::Parser;
use pest::error::Error;
use pest_derive::Parser;

use crate::ast::{
    Program, Chunk, ListDef, ListIdx, ListAppend, 
    DynConstructor, ArgExpr, AppendElement, 
    IndexExpr, Loop, MathOperator, ImportStmt,
    RangeBound, GlobalDecl,
};


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(crate) struct SMVXParser;

pub(crate) fn parse_program<P: AsRef<Path>>(file_path: P) -> Result<Program, Error<Rule>> {
    let unparsed_file = fs::read_to_string(file_path).expect("cannot read file");

    let mut pairs = SMVXParser::parse(Rule::program, &unparsed_file)?;
    // TODO: debug log here
    // debug the pest tree
    // println!("{:#?}", pairs);
    
    let mut chunks = Vec::new();
    let program_pair = pairs.next().unwrap();
    for chunk_pair in program_pair.into_inner() {
        chunks.push(parse_chunk(chunk_pair)?);
    }

    // TODO: debug log here
    // debug ast
    println!("{:#?}", chunks);

    Ok(Program { chunks })
}

fn parse_chunk(pair: pest::iterators::Pair<Rule>) -> Result<Chunk, Error<Rule>> {
    match pair.as_rule() {
        Rule::global_block => Ok(Chunk::Global(parse_global(pair)?)),
        Rule::import_stmt => Ok(Chunk::Import(parse_import(pair)?)),
        Rule::list => Ok(Chunk::ListDef(parse_list(pair)?)),
        Rule::list_indexing => Ok(Chunk::ListIdx(parse_list_indexing(pair)?)),
        Rule::list_unpack => Ok(Chunk::ListUnpack(pair.into_inner().next().unwrap().as_str().to_string())),
        Rule::list_append => Ok(Chunk::ListAppend(parse_list_append(pair)?)),
        Rule::dyn_constructor => Ok(Chunk::DynConstructor(parse_dyn_constructor(pair)?)),
        Rule::for_loop => Ok(Chunk::Loop(parse_for_loop(pair)?)),
        Rule::raw_nusmv_chunk => Ok(Chunk::RawNusmv(pair.as_str().to_string())),
        _ => Ok(Chunk::RawNusmv(pair.as_str().to_string())),
    }
}

fn parse_global(pair: pest::iterators::Pair<Rule>) -> Result<Vec<GlobalDecl>, Error<Rule>> {
    let inner = pair.into_inner();
    let mut globals = Vec::new();
    for p in inner.clone() {
        let mut p_inner = p.into_inner();
        let name = p_inner.next().unwrap().as_str().to_string();
        let value = p_inner.next().unwrap().as_str().parse::<i64>().unwrap();
        globals.push(GlobalDecl { name, value });
    }
    Ok(globals)
}

fn parse_list(pair: pest::iterators::Pair<Rule>) -> Result<ListDef, Error<Rule>> {
    let mut inner = pair.into_inner();
    let indent = inner.next().unwrap().as_str().to_string();
    let name = inner.next().unwrap().as_str().to_string();
    
    let element_pairs = inner.next().unwrap().into_inner();
    let elements = element_pairs.map(|e| e.as_str().to_string()).collect();

    Ok(ListDef { indent, name, elements })
}

fn parse_import(pair: pest::iterators::Pair<Rule>) -> Result<ImportStmt, Error<Rule>> {
    let mut inner = pair.into_inner();
    let path = inner.find(|p| p.as_rule() == Rule::path).unwrap().as_str().to_string();
    Ok(ImportStmt { path })
}

fn parse_list_indexing(pair: pest::iterators::Pair<Rule>) -> Result<ListIdx, Error<Rule>> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let idx = parse_index_expr(inner.next().unwrap());

    Ok(ListIdx { name, idx })
}

fn parse_index_expr(pair: pest::iterators::Pair<Rule>) -> IndexExpr {
    match pair.as_rule() {
        Rule::index_operand => IndexExpr::Operand(pair.as_str().to_string()),
        Rule::index_math => {
            let mut inner = pair.into_inner();
            let left = inner.next().unwrap().as_str().to_string();
            let op_pair = inner.next().unwrap();
            let op = match op_pair.as_str() {
                "+" => MathOperator::Add,
                "-" => MathOperator::Sub,
                other => panic!("Unknown operator: {}", other),
            };
            let right = inner.next().unwrap().as_str().to_string();
            IndexExpr::Math { left, op, right }
        }
        Rule::list_idx => parse_index_expr(pair.into_inner().next().unwrap()),
        _ => unreachable!("Unexpected rule in index expression: {:?}", pair.as_rule()),
    }
}

pub(crate) fn parse_list_append(pair: pest::iterators::Pair<Rule>) -> Result<ListAppend, pest::error::Error<Rule>> {
    let mut inner = pair.into_inner();
    
    // Get the indentation (e.g., "  ")
    let indent = inner.next().unwrap().as_str().to_string();

    // Get the list name (e.g., "servers")
    let name = inner.next().unwrap().as_str().to_string();
    
    // Pass the append_element Pair to our helper
    let element_pair = inner.next().unwrap(); 
    let element = parse_append_element(element_pair)?;

    Ok(ListAppend { indent, name, element })
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
            let idx = parse_index_expr(inner.next().unwrap());
            
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

fn parse_for_loop(pair: pest::iterators::Pair<Rule>) -> Result<Loop, pest::error::Error<Rule>> {
    let mut inner = pair.into_inner();
    
    // Get the indentation
    let indent = inner.next().unwrap().as_str().to_string();

    // Get the loop variable name
    let var_name = inner.next().unwrap().as_str().to_string();
    
    // Get the start and end values
    let start = parse_range_bound(inner.next().unwrap());
    let end = parse_range_bound(inner.next().unwrap());
    
    // Get the body of the loop (a series of chunks)
    let mut body = Vec::new();
    for chunk_pair in inner {
        body.push(parse_chunk(chunk_pair)?);
    }

    Ok(Loop { indent, var_name, start, end, body })
}

fn parse_range_bound(pair: pest::iterators::Pair<Rule>) -> RangeBound {
    let pair_str = pair.as_str().to_string();
    let rule = pair.into_inner().next().unwrap().as_rule();
    match rule {
        Rule::var_name => RangeBound::Variable(pair_str),
        _ => RangeBound::Number(pair_str.parse::<i64>().unwrap()),
    }
}