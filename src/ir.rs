use std::collections::HashMap;
use std::fmt::Display;
use std::error::Error;

use crate::ast::{Program, Chunk, ArgExpr, AppendElement, DynConstructor, IndexExpr, Loop, MathOperator};

#[derive(Debug, Clone)]
pub(crate) struct LoweredProgram {
    pub(crate) chunks: Vec<LoweredChunk>,
}

#[derive(Debug, Clone)]
pub(crate) enum LoweredChunk {
    Raw(String),
    VarDecl { indent: String, name: String, value: String }, // Flattened declarations
    VarRef(String),                                          // Flattened references
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LowerError {
    ListNotDefined { name: String },
    IndexOutOfBounds { list: String, index: i32, length: usize },
    UnknownLoopVariable { name: String },
}

impl Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ListNotDefined { name } => write!(f, "list {} is not defined", name),
            Self::IndexOutOfBounds { list, index, length } => {
                write!(f, "index {} is out of bounds for list {} of length {}", index, list, length)
            }
            Self::UnknownLoopVariable { name } => {
                write!(f, "unknown loop variable in list index: {}", name)
            }
        }
    }
}

impl Error for LowerError {}

// Lower does the lowering process, transforming the AST into a flattened IR representation.
// stores the current state of list lengths and loop variables during the lowering process.
pub(crate) struct Lowerer {
    list_lengths: HashMap<String, usize>,
    loop_vars: HashMap<String, i32>,
}

impl Lowerer {
    pub(crate) fn new() -> Self {
        Lowerer {
            list_lengths: HashMap::new(),
            loop_vars: HashMap::new(),
        }
    }

    pub(crate) fn lower(mut self, ast: Program) -> Result<LoweredProgram, LowerError> {
        let mut lowered_chunks = Vec::new();

        for chunk in ast.chunks {
            lowered_chunks.extend(self.lower_chunk(chunk)?);
        }

        Ok(LoweredProgram { chunks: lowered_chunks })
    }

    fn lower_chunk(&mut self, chunk: Chunk) -> Result<Vec<LoweredChunk>, LowerError> {
        match chunk {
            Chunk::RawNusmv(raw) => Ok(vec![LoweredChunk::Raw(raw)]),
            Chunk::ListDef(def) => {
                let name = singular(&def.name);
                self.list_lengths.insert(name.clone(), def.elements.len());
                let indent = def.indent;
                Ok(def.elements.into_iter().enumerate().map(|(i, element)| {
                    let var_name = format!("{}{}", name, i);
                    LoweredChunk::VarDecl { indent: indent.clone(), name: var_name, value: element }
                }).collect::<Vec<_>>())
            }
            Chunk::ListAppend(append) => {
                let name = singular(&append.name);
                if let Some(&length) = self.list_lengths.get(&name) {
                    let var_name = format!("{}{}", name, length);
                    let value = self.render_append_element(&append.element)?;
                    self.list_lengths.insert(name.clone(), length + 1);
                    Ok(vec![LoweredChunk::VarDecl { indent: append.indent, name: var_name, value }])
                } else {
                    Err(LowerError::ListNotDefined { name: append.name })
                }
            }
            Chunk::ListIdx(idx) => {
                let name = singular(&idx.name);
                if let Some(&length) = self.list_lengths.get(&name) {
                    let index = self.evaluate_index(&idx.idx)?;
                    self.validate_index(&name, index, length)?;
                    let var_name = format!("{}{}", name, index);
                    Ok(vec![LoweredChunk::VarRef(var_name)])
                } else {
                    Err(LowerError::ListNotDefined { name: idx.name })
                }
            }
            Chunk::DynConstructor(dc) => {
                let rendered = self.render_dyn_constructor(&dc)?;
                Ok(vec![LoweredChunk::Raw(rendered)])
            }
            Chunk::Loop(l) => self.lower_loop(&l),
            Chunk::Import(_) => {
                // Imports are handled in the transpiler, so we can ignore them here.
                Ok(vec![])
            }
        }
    }

    // render_append_element flattens the value being appended to a list into its final NuSMV text.
    fn render_append_element(
        &mut self,
        element: &AppendElement,
    ) -> Result<String, LowerError> {
        match element {
            AppendElement::DynConstructor(dc) => self.render_dyn_constructor(dc),
            AppendElement::Exprs(exprs) => exprs.iter().map(|e| self.render_arg_expr(e)).collect::<Result<String, LowerError>>(),
        }
    }

    // render_dyn_constructor flattens a dynamic constructor call (e.g. server_node(servers[0].active))
    // into its final NuSMV text.
    fn render_dyn_constructor(
        &mut self,
        dc: &DynConstructor,
    ) -> Result<String, LowerError> {
        let args = dc.args.iter().map(|a| self.render_arg_expr(a)).collect::<Result<String, LowerError>>()?;
        Ok(format!("{}({})", dc.name, args))
    }

    // render_arg_expr resolves an argument of a dynamic constructor into its final NuSMV text,
    // substituting list indexing (e.g. servers[0]) with the flattened variable name (e.g. server0).
    fn render_arg_expr(
        &mut self,
        expr: &ArgExpr,
    ) -> Result<String, LowerError> {
        match expr {
            ArgExpr::ListIdx(idx) => {
                let name: String = singular(&idx.name);
                let Some(&length) = self.list_lengths.get(&name) else {
                    return Err(LowerError::ListNotDefined { name: idx.name.clone() });
                };
                let index = self.evaluate_index(&idx.idx)?;
                self.validate_index(&name, index, length)?;
                Ok(format!("{}{}", name, index))
            }
            ArgExpr::Paren(exprs) => {
                let inner = exprs.iter().map(|e| self.render_arg_expr(e)).collect::<Result<String, _>>()?;
                Ok(format!("({})", inner))
            }
            ArgExpr::Chunk(raw) => Ok(raw.clone()),
        }
    }

    fn validate_index(&self, list: &str, index: i32, length: usize) -> Result<(), LowerError> {
        if index < 0 || index as usize >= length {
            return Err(LowerError::IndexOutOfBounds {
                list: list.to_string(),
                index,
                length,
            });
        }
        Ok(())
    }

    fn evaluate_index(&self, index: &IndexExpr) -> Result<i32, LowerError> {
        let operand_value = |operand: &str| {
            operand.parse::<i32>().ok().or_else(|| self.loop_vars.get(operand).copied())
                .ok_or_else(|| LowerError::UnknownLoopVariable { name: operand.to_string() })
        };

        match index {
            IndexExpr::Operand(operand) => operand_value(operand),
            IndexExpr::Math { left, op, right } => {
                let left = operand_value(left)?;
                let right = operand_value(right)?;
                match op {
                    MathOperator::Add => Ok(left + right),
                    MathOperator::Sub => Ok(left - right),
                }
            }
        }
    }

    fn lower_loop(&mut self, l: &Loop) -> Result<Vec<LoweredChunk>, LowerError> {
        let mut lowered_chunks = Vec::new();
        for i in l.start..=l.end {
            let previous = self.loop_vars.insert(l.var_name.clone(), i as i32);
            let result = l.body.iter().cloned().try_for_each(|mut chunk| {
                if let Chunk::ListAppend(ref mut append) = chunk {
                    append.indent = l.indent.clone();
                } else if let Chunk::ListDef(ref mut def) = chunk {
                    def.indent = l.indent.clone();
                }
                self.lower_chunk(chunk).map(|chunks| lowered_chunks.extend(chunks))
            });
            match previous {
                Some(value) => {
                    self.loop_vars.insert(l.var_name.clone(), value);
                }
                None => {
                    self.loop_vars.remove(&l.var_name);
                }
            }
            result?;
        }
        Ok(lowered_chunks)
    }

}

// singular helper function strips the trailing 's' used to derive a list's flattened variable prefix.
fn singular(name: &str) -> String {
    name.strip_suffix('s').unwrap_or(name).to_string()
}