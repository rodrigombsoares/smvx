use std::{collections::HashMap, vec};

use super::{LowerError, LoweredChunk, LoweredProgram};
use crate::ast::{
    AppendElement, ArgExpr, Chunk, DynConstructor, ExpandExpr, IndexExpr, ListIdx, Loop,
    MathOperator, Program, RangeBound, ReduceExpr,
};

#[derive(Debug, Clone)]
struct ListMetadata {
    length: usize,
    assign_op: String,
}

// Lower does the lowering process, transforming the AST into a flattened IR representation.
// stores the current state of list lengths and loop variables during the lowering process.
pub(crate) struct Lowerer {
    lists: HashMap<String, ListMetadata>,
    list_next_indices: HashMap<String, usize>,
    loop_vars: HashMap<String, i32>,
    global_vars: HashMap<String, i64>,
}

impl Lowerer {
    pub(crate) fn new() -> Self {
        Lowerer {
            lists: HashMap::new(),
            list_next_indices: HashMap::new(),
            loop_vars: HashMap::new(),
            global_vars: HashMap::new(),
        }
    }

    pub(crate) fn lower(mut self, ast: Program) -> Result<LoweredProgram, LowerError> {
        self.collect_variables(&ast.chunks)?;
        self.lower_chunks(ast.chunks)
    }

    fn collect_variables(&mut self, chunks: &Vec<Chunk>) -> Result<(), LowerError> {
        for chunk in chunks {
            match chunk {
                Chunk::ListDef(def) => {
                    let name: String = singular(&def.name);
                    self.lists.insert(
                        name,
                        ListMetadata {
                            length: def.elements.len(),
                            assign_op: def.assign_op.clone(),
                        },
                    );
                }
                Chunk::ListAppend(ap) => {
                    let name: String = singular(&ap.name);
                    if let Some(metadata) = self.lists.get_mut(&name) {
                        metadata.length += 1;
                    } else {
                        return Err(LowerError::ListNotDefined {
                            name: ap.name.clone(),
                        });
                    }
                }
                Chunk::Global(def) => {
                    for g in def {
                        self.global_vars.insert(g.name.clone(), g.value);
                    }
                }
                Chunk::Loop(loop_) => {
                    let start = self.evaluate_range_bound(&loop_.start)?;
                    let end = self.evaluate_range_bound(&loop_.end)?;

                    for i in start..end {
                        let previous = self.loop_vars.insert(loop_.var_name.clone(), i as i32);
                        let result = self.collect_variables(&loop_.body);
                        match previous {
                            Some(value) => {
                                self.loop_vars.insert(loop_.var_name.clone(), value);
                            }
                            None => {
                                self.loop_vars.remove(&loop_.var_name);
                            }
                        }
                        result?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn lower_chunks(&mut self, chunks: Vec<Chunk>) -> Result<LoweredProgram, LowerError> {
        let mut lowered_chunks = Vec::new();

        for chunk in chunks {
            lowered_chunks.extend(self.lower_chunk(chunk)?);
        }

        Ok(LoweredProgram {
            chunks: lowered_chunks,
        })
    }

    fn lower_chunk(&mut self, chunk: Chunk) -> Result<Vec<LoweredChunk>, LowerError> {
        match chunk {
            Chunk::RawNusmv(raw) => Ok(vec![LoweredChunk::Raw(raw)]),
            Chunk::Global(_) => Ok(vec![]),
            Chunk::ListDef(def) => {
                let name = singular(&def.name);
                let indent = def.indent;
                let assign_op = def.assign_op;
                self.list_next_indices
                    .insert(name.clone(), def.elements.len());
                let mut declarations = Vec::with_capacity(def.elements.len());
                for (i, element) in def.elements.into_iter().enumerate() {
                    let var_name = format!("{}{}", name, i);
                    let value = element
                        .iter()
                        .map(|expr| self.render_arg_expr(expr))
                        .collect::<Result<String, _>>()?
                        .trim()
                        .to_string();
                    declarations.push(LoweredChunk::VarDecl {
                        indent: indent.clone(),
                        name: var_name,
                        assign_op: assign_op.clone(),
                        value,
                    });
                }
                Ok(declarations)
            }
            Chunk::ListAppend(append) => {
                let name = singular(&append.name);
                if let Some(&next_index) = self.list_next_indices.get(&name) {
                    let assign_op = self
                        .lists
                        .get(&name)
                        .ok_or_else(|| LowerError::ListNotDefined {
                            name: append.name.clone(),
                        })?
                        .assign_op
                        .clone();
                    let var_name = format!("{}{}", name, next_index);
                    let value = self.render_append_element(&append.element)?;
                    self.list_next_indices.insert(name, next_index + 1);
                    Ok(vec![LoweredChunk::VarDecl {
                        indent: append.indent,
                        name: var_name,
                        assign_op,
                        value,
                    }])
                } else {
                    Err(LowerError::ListNotDefined { name: append.name })
                }
            }
            Chunk::ListUnpack(list) => {
                let base_name = singular(&list);

                self.lists
                    .get(&base_name)
                    .ok_or(LowerError::ListNotDefined { name: list })
                    .map(|metadata| {
                        let length = metadata.length;
                        // Pre-allocate the exact capacity needed (length * 2 - 1)
                        let mut unpacked =
                            Vec::with_capacity(length.saturating_mul(2).saturating_sub(1));

                        for i in 0..length {
                            // Prepend the comma for all elements AFTER the first
                            if i > 0 {
                                unpacked.push(LoweredChunk::Raw(", ".to_string()));
                            }
                            unpacked.push(LoweredChunk::VarRef(format!("{}{}", base_name, i)));
                        }

                        unpacked
                    })
            }
            Chunk::ListIdx(idx) => {
                let name = singular(&idx.name);
                if let Some(metadata) = self.lists.get(&name) {
                    let length = metadata.length;
                    let index = self.evaluate_index(&idx.idx)?;
                    self.validate_index(&name, index, length)?;
                    let var_name = format!("{}{}", name, index);
                    Ok(vec![LoweredChunk::VarRef(var_name)])
                } else {
                    Err(LowerError::ListNotDefined { name: idx.name })
                }
            }
            Chunk::ReduceExp(reduce) => {
                let rendered = self.render_reduce_expr(&reduce)?;
                Ok(vec![LoweredChunk::Raw(rendered)])
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
            Chunk::IdInterpolation(id_interpol) => {
                let rendered = self.render_id_interpol(&id_interpol)?;
                Ok(vec![LoweredChunk::Raw(rendered)])
            }
            Chunk::ValueInterpolation(index) => Ok(vec![LoweredChunk::Raw(
                self.evaluate_index(&index)?.to_string(),
            )]),
            Chunk::ExpandExp(expand) => {
                let rendered = self.render_expand_expr(&expand)?;
                Ok(vec![LoweredChunk::Raw(rendered)])
            }
        }
    }

    // render_append_element flattens the value being appended to a list into its final NuSMV text.
    fn render_append_element(&mut self, element: &AppendElement) -> Result<String, LowerError> {
        match element {
            AppendElement::DynConstructor(dc) => self.render_dyn_constructor(dc),
            AppendElement::Exprs(exprs) => exprs
                .iter()
                .map(|e| self.render_arg_expr(e))
                .collect::<Result<String, LowerError>>(),
        }
    }

    fn render_reduce_expr(&mut self, reduce: &ReduceExpr) -> Result<String, LowerError> {
        let start = self.evaluate_range_bound(&reduce.range_start)?;
        let end = self.evaluate_range_bound(&reduce.range_end)?;
        let mut reduced = String::new();
        for i in start..end {
            let previous = self.loop_vars.insert(reduce.loop_var.clone(), i as i32);
            // Prepend the operator for all elements AFTER the first
            if i > 0 {
                reduced.push_str(&format!(" {} ", reduce.op));
            }
            let args = reduce
                .args
                .iter()
                .map(|a| self.render_arg_expr(a))
                .collect::<Result<String, LowerError>>()?;
            reduced.push_str(&args);
            match previous {
                Some(value) => {
                    self.loop_vars.insert(reduce.loop_var.clone(), value);
                }
                None => {
                    self.loop_vars.remove(&reduce.loop_var);
                }
            }
        }

        Ok(reduced)
    }

    // render_dyn_constructor flattens a dynamic constructor call (e.g. server_node(servers[0].active))
    // into its final NuSMV text.
    fn render_dyn_constructor(&mut self, dc: &DynConstructor) -> Result<String, LowerError> {
        let args = dc
            .args
            .iter()
            .map(|a| self.render_arg_expr(a))
            .collect::<Result<String, LowerError>>()?;
        Ok(format!("{}({})", dc.name, args))
    }

    // render_arg_expr resolves an argument of a dynamic constructor into its final NuSMV text,
    // substituting list indexing (e.g. servers[0]) with the flattened variable name (e.g. server0).
    fn render_arg_expr(&mut self, expr: &ArgExpr) -> Result<String, LowerError> {
        // TODO: dedup this code
        match expr {
            ArgExpr::ListIdx(idx) => {
                let name: String = singular(&idx.name);
                let Some(metadata) = self.lists.get(&name) else {
                    return Err(LowerError::ListNotDefined {
                        name: idx.name.clone(),
                    });
                };
                let length = metadata.length;
                let index = self.evaluate_index(&idx.idx)?;
                self.validate_index(&name, index, length)?;
                Ok(format!("{}{}", name, index))
            }
            ArgExpr::ValueInterpolation(index) => Ok(self.evaluate_index(index)?.to_string()),
            ArgExpr::Paren(exprs) => {
                let inner = exprs
                    .iter()
                    .map(|e| self.render_arg_expr(e))
                    .collect::<Result<String, _>>()?;
                Ok(format!("({})", inner))
            }
            // Whitespace-only chunks are formatting artifacts from multi-line reduce!/loop bodies; drop them.
            ArgExpr::Chunk(raw) => Ok(if raw.trim().is_empty() {
                String::new()
            } else {
                raw.clone()
            }),
            ArgExpr::Reduce(reduce) => self.render_reduce_expr(reduce),
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

    fn lower_loop(&mut self, l: &Loop) -> Result<Vec<LoweredChunk>, LowerError> {
        let mut lowered_chunks = Vec::new();
        let start = self.evaluate_range_bound(&l.start)?;
        let end = self.evaluate_range_bound(&l.end)?;
        let body_len = l.body.len();
        // If the body's first real content is a self-indenting chunk (it renders its own
        // indent), the leading whitespace-only raw chunk before it must be dropped entirely
        // instead of replaced with `l.indent`, otherwise the line gets indented twice.
        let first_is_self_indenting = matches!(
            l.body.get(1),
            Some(Chunk::ListAppend(_)) | Some(Chunk::ListDef(_)) | Some(Chunk::Loop(_))
        );

        for i in start..end {
            let is_last_iter = i == end - 1;
            let previous = self.loop_vars.insert(l.var_name.clone(), i as i32);
            let result = l
                .body
                .iter()
                .cloned()
                .enumerate()
                .try_for_each(|(idx, mut chunk)| {
                    // TODO: make this generic
                    if let Chunk::ListAppend(ref mut append) = chunk {
                        append.indent = l.indent.clone();
                    } else if let Chunk::ListDef(ref mut def) = chunk {
                        def.indent = l.indent.clone();
                    } else if let Chunk::Loop(ref mut inner) = chunk {
                        // Nested loops must render at the outermost loop's indent, since all the
                        // surrounding `for`/`od` scaffolding disappears from the emitted output.
                        inner.indent = l.indent.clone();
                    } else if let Chunk::RawNusmv(ref mut raw) = chunk {
                        // The leading whitespace/newline captured before the body's first raw text
                        // is the loop's own indentation, not the emitted line's; restore it explicitly.
                        if idx == 0 {
                            let trimmed = raw.trim_start_matches(['\n', ' ', '\t']);
                            *raw = if trimmed.is_empty() && first_is_self_indenting {
                                String::new()
                            } else {
                                format!("{}{}", l.indent, trimmed)
                            };
                        }
                        // Only the final iteration's trailing newline should survive, otherwise the
                        // following top-level content's own leading newline creates a blank line.
                        if idx == body_len - 1 && is_last_iter {
                            *raw = raw.trim_end_matches('\n').to_string();
                        }
                    }
                    self.lower_chunk(chunk)
                        .map(|chunks| lowered_chunks.extend(chunks))
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

    // render_id_interpol flattens identifier interpolation (e.g. D{i}) into its final NuSMV text,
    // concatenating the base name with the evaluated index without any list-bounds validation.
    fn render_id_interpol(&self, id_interpol: &ListIdx) -> Result<String, LowerError> {
        let index = self.evaluate_index(&id_interpol.idx)?;
        Ok(format!("{}{}", id_interpol.name, index))
    }

    fn render_expand_expr(&mut self, expand: &ExpandExpr) -> Result<String, LowerError> {
        let start = self.evaluate_range_bound(&expand.range_start)?;
        let end = self.evaluate_range_bound(&expand.range_end)?;

        let mut expanded = String::new();
        for i in start..end {
            let previous = self.loop_vars.insert(expand.loop_var.clone(), i as i32);
            if i > start {
                expanded.push_str(", ");
            }
            let rendered = self.render_id_interpol(&expand.id_interpol);
            match previous {
                Some(value) => {
                    self.loop_vars.insert(expand.loop_var.clone(), value);
                }
                None => {
                    self.loop_vars.remove(&expand.loop_var);
                }
            }
            expanded.push_str(&rendered?);
        }

        Ok(expanded)
    }

    fn evaluate_range_bound(&self, bound: &RangeBound) -> Result<i64, LowerError> {
        match bound {
            RangeBound::Number(value) => Ok(*value),
            RangeBound::Variable(name) => self
                .global_vars
                .get(name)
                .copied()
                .ok_or_else(|| LowerError::UnknownLoopVariable { name: name.clone() }),
            RangeBound::Math(index) => Ok(i64::from(self.evaluate_index(index)?)),
        }
    }

    fn evaluate_index(&self, index: &IndexExpr) -> Result<i32, LowerError> {
        let operand_value = |operand: &str| {
            operand
                .parse::<i32>()
                .ok()
                .or_else(|| self.loop_vars.get(operand).copied())
                .or_else(|| self.global_vars.get(operand).map(|&value| value as i32))
                .ok_or_else(|| LowerError::UnknownLoopVariable {
                    name: operand.to_string(),
                })
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
}

// singular helper function strips the trailing 's' used to derive a list's flattened variable prefix.
fn singular(name: &str) -> String {
    name.strip_suffix('s').unwrap_or(name).to_string()
}
