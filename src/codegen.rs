use crate::ast::{Program};

pub fn generate_smv(program: &Program) -> String {
    let mut output = String::new();
    
    for chunk in &program.chunks {
        output.push_str(&chunk.to_string());
    }

    output
}
