// ir.rs
use std::collections::HashMap;
use crate::ast::{Program, Chunk};

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

// 2. The Transformation Logic (Stateful)
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
                let name = append.name.strip_suffix('s').unwrap_or(&append.name).to_string();
                if let Some(&length) = list_lengths.get(&name) {
                    let var_name = format!("{}{}", name, length);
                    lowered_chunks.push(LoweredChunk::VarDecl { 
                        name: var_name, 
                        value: append.element,
                    });
                    list_lengths.insert(append.name.clone(), length + 1);
                } else {
                    panic!("List {} not defined before append", append.name);
                }
            }
            Chunk::ListIdx(idx) => {
                let name = idx.name.strip_suffix('s').unwrap_or(&idx.name).to_string();
                // TODO: check lenght (index out of range)
                if let Some(&length) = list_lengths.get(&name) {
                    let var_name = format!("{}{}", idx.name, idx.idx);
                    lowered_chunks.push(LoweredChunk::VarRef(var_name));
                } else {
                    panic!("List {} not defined before indexing", idx.name);
                }
            }
        }
    }

    LoweredProgram { chunks: lowered_chunks }
}