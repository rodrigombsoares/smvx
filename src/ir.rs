// ir.rs
use std::collections::HashMap;
use crate::ast::{Program, Chunk, ArgExpr, AppendElement, DynConstructor};

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
        match chunk {
            Chunk::RawNusmv(raw) => {
                lowered_chunks.push(LoweredChunk::Raw(raw));
            }
            Chunk::ListDef(def) => {
                let name = def.name.strip_suffix('s').unwrap_or(&def.name).to_string();
                list_lengths.insert(name.clone(), def.elements.len());
                for (i, element) in def.elements.into_iter().enumerate() {
                    let var_name = format!("{}{}", name, i);
                    lowered_chunks.push(LoweredChunk::VarDecl { 
                        name: var_name, 
                        value: element,
                    });
                }
            }
            Chunk::ListAppend(append) => {
                let name = singular(&append.name);
                if let Some(&length) = list_lengths.get(&name) {
                    let var_name = format!("{}{}", name, length);
                    let value = render_append_element(&append.element, &list_lengths);
                    lowered_chunks.push(LoweredChunk::VarDecl { 
                        name: var_name, 
                        value,
                    });
                    list_lengths.insert(name, length + 1);
                } else {
                    panic!("List {} not defined before append", append.name);
                }
            }
            Chunk::ListIdx(idx) => {
                let name = singular(&idx.name);
                // TODO: check lenght (index out of range)
                if let Some(&_length) = list_lengths.get(&name) {
                    let var_name = format!("{}{}", name, idx.idx);
                    lowered_chunks.push(LoweredChunk::VarRef(var_name));
                } else {
                    panic!("List {} not defined before indexing", idx.name);
                }
            }
            Chunk::DynConstructor(dc) => {
                let rendered = render_dyn_constructor(&dc, &list_lengths);
                lowered_chunks.push(LoweredChunk::Raw(rendered));
            }
        }
    }

    LoweredProgram { chunks: lowered_chunks }
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
            let name = singular(&idx.name);
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

// singular helper function strips the trailing 's' used to derive a list's flattened variable prefix.
fn singular(name: &str) -> String {
    name.strip_suffix('s').unwrap_or(name).to_string()
}
