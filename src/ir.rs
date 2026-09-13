// ir.rs
use std::collections::HashMap;
use crate::ast::{Program, Chunk, ArgExpr, AppendElement, DynConstructor, IndexExpr, Loop, MathOperator};

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
    let mut loop_vars: HashMap<String, i32> = HashMap::new();

    for chunk in ast.chunks {
        lowered_chunks.extend(lower_chunk(chunk, &mut list_lengths, &mut loop_vars));
    }

    LoweredProgram { chunks: lowered_chunks }
}

fn lower_chunk(
    chunk: Chunk,
    list_lengths: &mut HashMap<String, usize>,
    loop_vars: &mut HashMap<String, i32>,
) -> Vec<LoweredChunk> {
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
                let value = render_append_element(&append.element, list_lengths, loop_vars);
                list_lengths.insert(name.clone(), length + 1);
                vec![LoweredChunk::VarDecl { name: var_name, value }]
            } else {
                panic!("List {} not defined before append", append.name);
            }
        }
        Chunk::ListIdx(idx) => {
            let name = singular(&idx.name);
            if list_lengths.contains_key(&name) {
                let index = evaluate_index(&idx.idx, loop_vars);
                let var_name = format!("{}{}", name, index);
                vec![LoweredChunk::VarRef(var_name)]
            } else {
                panic!("List {} not defined before indexing", idx.name);
            }
        }
        Chunk::DynConstructor(dc) => {
            let rendered = render_dyn_constructor(&dc, list_lengths, loop_vars);
            vec![LoweredChunk::Raw(rendered)]
        }
        Chunk::Loop(l) => lower_loop(&l, list_lengths, loop_vars),
    }
}

// render_dyn_constructor flattens a dynamic constructor call (e.g. server_node(servers[0].active))
// into its final NuSMV text.
fn render_dyn_constructor(
    dc: &DynConstructor,
    list_lengths: &HashMap<String, usize>,
    loop_vars: &HashMap<String, i32>,
) -> String {
    let args: String = dc.args.iter().map(|a| render_arg_expr(a, list_lengths, loop_vars)).collect();
    format!("{}({})", dc.name, args)
}

// render_append_element flattens the value being appended to a list into its final NuSMV text.
fn render_append_element(
    element: &AppendElement,
    list_lengths: &HashMap<String, usize>,
    loop_vars: &HashMap<String, i32>,
) -> String {
    match element {
        AppendElement::DynConstructor(dc) => render_dyn_constructor(dc, list_lengths, loop_vars),
        AppendElement::Exprs(exprs) => exprs.iter().map(|e| render_arg_expr(e, list_lengths, loop_vars)).collect(),
    }
}

// render_arg_expr resolves an argument of a dynamic constructor into its final NuSMV text,
// substituting list indexing (e.g. servers[0]) with the flattened variable name (e.g. server0).
fn render_arg_expr(
    expr: &ArgExpr,
    list_lengths: &HashMap<String, usize>,
    loop_vars: &HashMap<String, i32>,
) -> String {
    match expr {
        // TODO: handle list index out of range
        ArgExpr::ListIdx(idx) => {
            let name: String = singular(&idx.name);
            if !list_lengths.contains_key(&name) {
                panic!("List {} not defined before indexing", idx.name);
            }
            let index = evaluate_index(&idx.idx, loop_vars);
            format!("{}{}", name, index)
        }
        ArgExpr::Paren(exprs) => {
            let inner: String = exprs.iter().map(|e| render_arg_expr(e, list_lengths, loop_vars)).collect();
            format!("({})", inner)
        }
        ArgExpr::Chunk(raw) => raw.clone(),
    }
}

fn evaluate_index(index: &IndexExpr, loop_vars: &HashMap<String, i32>) -> i32 {
    fn operand_value(operand: &str, loop_vars: &HashMap<String, i32>) -> i32 {
        operand.parse::<i32>().unwrap_or_else(|_| {
            *loop_vars.get(operand).unwrap_or_else(|| {
                panic!("Unknown loop variable in list index: {}", operand)
            })
        })
    }

    match index {
        IndexExpr::Operand(operand) => operand_value(operand, loop_vars),
        IndexExpr::Math { left, op, right } => {
            let left = operand_value(left, loop_vars);
            let right = operand_value(right, loop_vars);
            match op {
                MathOperator::Add => left + right,
                MathOperator::Sub => left - right,
                _ => panic!("Unsupported index operator: {:?}", op),
            }
        }
    }
}

fn lower_loop(
    l: &Loop,
    list_lengths: &mut HashMap<String, usize>,
    loop_vars: &mut HashMap<String, i32>,
) -> Vec<LoweredChunk> {
    let mut lowered_chunks = Vec::new();
    for i in l.start..=l.end {
        let previous = loop_vars.insert(l.var_name.clone(), i as i32);
        for chunk in &l.body {
            lowered_chunks.extend(lower_chunk(chunk.clone(), list_lengths, loop_vars));
        }
        match previous {
            Some(value) => {
                loop_vars.insert(l.var_name.clone(), value);
            }
            None => {
                loop_vars.remove(&l.var_name);
            }
        }
    }
    lowered_chunks
}

// singular helper function strips the trailing 's' used to derive a list's flattened variable prefix.
fn singular(name: &str) -> String {
    name.strip_suffix('s').unwrap_or(name).to_string()
}
