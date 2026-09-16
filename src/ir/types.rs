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
