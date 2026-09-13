#[derive(Debug, Clone)]
pub struct Program {
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub enum Chunk {
    ListDef(ListDef),
    ListIdx(ListIdx),
    ListAppend(ListAppend),
    DynConstructor(DynConstructor),
    Loop(Loop),
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
    pub idx: IndexExpr,
}

#[derive(Debug, Clone)]
pub enum IndexExpr {
    Operand(String), // e.g., "0" or "i"
    Math {
        left: String,
        op: MathOperator,
        right: String,
    },
}

#[derive(Debug, Clone)]
pub enum MathOperator {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub struct ListAppend {
    pub name: String,
    pub element: AppendElement, 
}

#[derive(Debug, Clone)]
pub enum AppendElement {
    DynConstructor(DynConstructor),
    Exprs(Vec<ArgExpr>), 
}

#[derive(Debug, Clone)]
pub struct DynConstructor {
    pub name: String,
    pub args: Vec<ArgExpr>,
}

#[derive(Debug, Clone)]
pub enum ArgExpr {
    ListIdx(ListIdx),          
    Paren(Vec<ArgExpr>),       
    Chunk(String),             
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub var_name: String,
    pub start: usize,
    pub end: usize,
    pub body: Vec<Chunk>,
}