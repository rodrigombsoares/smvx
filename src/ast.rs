#[derive(Debug, Clone)]
pub struct Program {
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub enum Chunk {
    ListDef(ListDef),
    ListIdx(ListIdx),
    ListAppend(ListAppend),
    RawNusmv(String),
}

#[derive(Debug, Clone)]
pub struct ListDef {
    pub name: String,
    pub elements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ListIdx {
    pub name: String,
    pub idx: usize,
}

#[derive(Debug, Clone)]
pub struct ListAppend {
    pub name: String,
    pub element: String,
}
