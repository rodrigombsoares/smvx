use crate::ir::{LoweredProgram, LoweredChunk};

pub(crate) fn generate_smv(ir: &LoweredProgram) -> String {
    let mut output = String::new();
    
    for chunk in &ir.chunks {
        match chunk {
            LoweredChunk::Raw(raw) => output.push_str(raw),
            LoweredChunk::VarDecl { indent, name, value } => {
                output.push_str(&format!("{}{} : {};\n", indent, name, value));
            }
            LoweredChunk::VarRef(ref_name) => {
                output.push_str(ref_name);
            }
        }
    }

    output
}
