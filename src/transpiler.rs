use std::collections::HashSet;
use std::path::PathBuf;
use std::error::Error;

use crate::parser::parse_program;
use crate::codegen::generate_smv;
use crate::ast::Chunk;
use crate::ast::Program;
use crate::ir::Lowerer;

pub struct SMVXTranspiler {
    visited_imports: HashSet<PathBuf>,
}

impl SMVXTranspiler {
    pub fn new() -> Self {
        Self {
            visited_imports: HashSet::new(),
        }
    }

    // collect_chunks recursively collects all chunks from the given file and its imports, 
    // while avoiding duplicate imports
    fn collect_chunks(
        &mut self,
        file_path: PathBuf,
    ) -> Result<Vec<Chunk>, Box<dyn Error>> {
        let file_path = std::fs::canonicalize(file_path)?;

        // Handles duplicate imports
        if !self.visited_imports.insert(file_path.clone()) {
            return Ok(Vec::new());
        }

        let base_path = file_path
            .parent()
            .ok_or("file path has no parent directory")?
            .to_path_buf();

        let parsed = parse_program(&file_path)?;
        let mut chunks = Vec::new();
        let mut import_paths = Vec::new();

        for chunk in parsed.chunks {
            match chunk {
                Chunk::Import(import) => {
                    // adding to a queue so that main module comes first in the output, 
                    // and imported modules come after
                    import_paths.push(base_path.join(import.path));
                }
                chunk => chunks.push(chunk),
            }
        }

        // ad-hoc removal of leading blank lines from the first chunk, if it is a RawNusmv chunk
        if let Some(Chunk::RawNusmv(raw)) = chunks.first_mut() {
            *raw = raw.trim_start_matches(['\r', '\n']).to_string();
        }

        for import_path in import_paths {
            chunks.push(Chunk::RawNusmv("\n".to_string()));
            chunks.extend(self.collect_chunks(import_path)?);
        }

        Ok(chunks)
    }

    pub fn transpile(
        &mut self,
        file_path: PathBuf,
    ) -> Result<String, Box<dyn Error>> {
        let chunks = self.collect_chunks(file_path)?;

        let ast = Program { chunks };

        let lowerer = Lowerer::new();
        let lowered = lowerer.lower(ast)?;

        Ok(generate_smv(&lowered))
    }
}
