use crate::ir::{LoweredChunk, LoweredProgram};

pub(crate) fn generate_smv(ir: &LoweredProgram) -> String {
    let mut output = String::new();

    for chunk in &ir.chunks {
        match chunk {
            LoweredChunk::Raw(raw) => output.push_str(raw),
            LoweredChunk::VarDecl {
                indent,
                name,
                assign_op,
                value,
            } => {
                output.push_str(&format!("{}{} {} {};\n", indent, name, assign_op, value));
            }
            LoweredChunk::VarRef(ref_name) => {
                output.push_str(ref_name);
            }
        }
    }

    output
}

