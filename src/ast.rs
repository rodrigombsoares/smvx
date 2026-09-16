#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub(crate) enum Chunk {
    Global(Vec<GlobalDecl>),
    Import(ImportStmt),
    ListDef(ListDef),
    ListIdx(ListIdx),
    ListUnpack(String),
    ListAppend(ListAppend),
    ReduceExp(ReduceExpr),
    DynConstructor(DynConstructor),
    Loop(Loop),
    RawNusmv(String),
}

#[derive(Debug, Clone)]
pub(crate) struct ImportStmt {
    pub(crate) path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ListDef {
    pub(crate) indent: String,
    pub(crate) name: String,
    pub(crate) elements: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ListIdx {
    pub(crate) name: String,
    pub(crate) idx: IndexExpr,
}

#[derive(Debug, Clone)]
pub(crate) enum IndexExpr {
    Operand(String), // e.g., "0" or "i"
    Math {
        left: String,
        op: MathOperator,
        right: String,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum MathOperator {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub(crate) struct ListAppend {
    pub(crate) indent: String,
    pub(crate) name: String,
    pub(crate) element: AppendElement, 
}

#[derive(Debug, Clone)]
pub(crate) enum AppendElement {
    DynConstructor(DynConstructor),
    Exprs(Vec<ArgExpr>), 
}

#[derive(Debug, Clone)]
pub(crate) struct ReduceExpr {
    pub(crate) op: String,
    pub(crate) loop_var: String,
    pub(crate) range_start: RangeBound,
    pub(crate) range_end: RangeBound,
    pub(crate) args: Vec<ArgExpr>,
}

#[derive(Debug, Clone)]
pub(crate) struct DynConstructor {
    pub(crate) name: String,
    pub(crate) args: Vec<ArgExpr>,
}

#[derive(Debug, Clone)]
pub(crate) enum ArgExpr {
    ListIdx(ListIdx),          
    Paren(Vec<ArgExpr>),       
    Chunk(String),             
}

#[derive(Debug, Clone)]
pub(crate) struct Loop {
    pub(crate) indent: String,
    pub(crate) var_name: String,
    pub(crate) start: RangeBound,
    pub(crate) end: RangeBound,
    pub(crate) body: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub(crate) enum RangeBound {
    Number(i64),
    Variable(String),
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalDecl {
    pub(crate) name: String,
    pub(crate) value: i64,
}