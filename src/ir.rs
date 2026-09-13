// ir.rs
use std::{collections::HashMap, vec};
use crate::ast::{Program, Chunk, ArgExpr, AppendElement, DynConstructor, Loop};

// 1. The IR Data Structures
#[derive(Debug, Clone)]
pub struct LoweredProgram {
    pub chunks: Vec<LoweredChunk>,
}

#[derive(Debug, Clone)]
pub enum LoweredChunk {
    Raw(String),
    VarDecl { name: String, value: String }, // Flattened declarations
    VarRef(String),                          // Flattened references
}

// lower holds transformation logic to convert the AST into a flattened IR representation. 
// TODO: proper error handling, not panic
pub fn lower(ast: Program) -> LoweredProgram {
    let mut lowered_chunks = Vec::new();
    let mut list_lengths: HashMap<String, usize> = HashMap::new();

    for chunk in ast.chunks {
        lowered_chunks.extend(lower_chunk(chunk, &mut list_lengths));
    }

    LoweredProgram { chunks: lowered_chunks }
}

fn lower_chunk(chunk: Chunk, list_lengths: &mut HashMap<String, usize>) -> Vec<LoweredChunk> {
    match chunk {
        Chunk::RawNusmv(raw) => vec![LoweredChunk::Raw(raw)],
        Chunk::ListDef(def) => {
            let name = singular(&def.name);
            list_lengths.insert(name.clone(), def.elements.len());
            def.elements.into_iter().enumerate().map(|(i, element)| {
                let var_name = format!("{}{}", name, i);
                LoweredChunk::VarDecl { name: var_name, value: element }
            }).collect()
        }
        Chunk::ListAppend(append) => {
            let name = singular(&append.name);
            if let Some(&length) = list_lengths.get(&name) {
                let var_name = format!("{}{}", name, length);
                let value = render_append_element(&append.element, list_lengths);
                list_lengths.insert(name.clone(), length + 1);
                vec![LoweredChunk::VarDecl { name: var_name, value }]
            } else {
                panic!("List {} not defined before append", append.name);
            }
        }
        Chunk::ListIdx(idx) => {
            let name = singular(&idx.name);
            if let Some(&_length) = list_lengths.get(&name) {
                let var_name = format!("{}{}", name, idx.idx);
                vec![LoweredChunk::VarRef(var_name)]
            } else {
                panic!("List {} not defined before indexing", idx.name);
            }
        }
        Chunk::DynConstructor(dc) => {
            let rendered = render_dyn_constructor(&dc, list_lengths);
            vec![LoweredChunk::Raw(rendered)]
        }
        Chunk::Loop(l) => lower_loop(&l, list_lengths),
        _ => {
            panic!("Unsupported chunk type in lower_chunk: {:?}", chunk);
        }
    }
}

// render_dyn_constructor flattens a dynamic constructor call (e.g. server_node(servers[0].active))
// into its final NuSMV text.
fn render_dyn_constructor(dc: &DynConstructor, list_lengths: &HashMap<String, usize>) -> String {
    let args: String = dc.args.iter().map(|a| render_arg_expr(a, list_lengths)).collect();
    format!("{}({})", dc.name, args)
}

// render_append_element flattens the value being appended to a list into its final NuSMV text.
fn render_append_element(element: &AppendElement, list_lengths: &HashMap<String, usize>) -> String {
    match element {
        AppendElement::DynConstructor(dc) => render_dyn_constructor(dc, list_lengths),
        AppendElement::Exprs(exprs) => exprs.iter().map(|e| render_arg_expr(e, list_lengths)).collect(),
    }
}

// render_arg_expr resolves an argument of a dynamic constructor into its final NuSMV text,
// substituting list indexing (e.g. servers[0]) with the flattened variable name (e.g. server0).
fn render_arg_expr(expr: &ArgExpr, list_lengths: &HashMap<String, usize>) -> String {
    match expr {
        ArgExpr::ListIdx(idx) => {
            // TODO: code duplication here. `Chunk::ListIdx` on lower() uses the same logic. 
            let name: String = singular(&idx.name);
            if !list_lengths.contains_key(&name) {
                panic!("List {} not defined before indexing", idx.name);
            }
            format!("{}{}", name, idx.idx)
        }
        ArgExpr::Paren(exprs) => {
            let inner: String = exprs.iter().map(|e| render_arg_expr(e, list_lengths)).collect();
            format!("({})", inner)
        }
        ArgExpr::Chunk(raw) => raw.clone(),
    }
}

fn lower_loop(l: &Loop, list_lengths: &mut HashMap<String, usize>) -> Vec<LoweredChunk> {
    let mut lowered_chunks = Vec::new();
    for i in l.start..=l.end {
        for chunk in &l.body {
            let mut chunk_clone = chunk.clone();
            // TODO: fix list assignment with variables
            // // Replace loop variable with current index in the body
            // replace_loop_var(&mut chunk_clone, &l.var_name, i);
            lowered_chunks.extend(lower_chunk(chunk_clone, list_lengths));
        }
    }
    lowered_chunks
}

// singular helper function strips the trailing 's' used to derive a list's flattened variable prefix.
fn singular(name: &str) -> String {
    name.strip_suffix('s').unwrap_or(name).to_string()
}
