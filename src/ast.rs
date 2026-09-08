use std::fmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub enum Chunk {
    ListDef(ListDef),
    ListIdx(ListIdx),
    RawNusmv(String),
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chunk::RawNusmv(raw) => write!(f, "{}", raw),
            Chunk::ListDef(list_def) => write!(f, "{}", list_def),
            Chunk::ListIdx(list_idx) => write!(f, "{}", list_idx),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListDef {
    pub name: String,
    pub elements: Vec<String>,
}

impl fmt::Display for ListDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.strip_suffix('s').unwrap_or(&self.name).to_string();
        // Expand the list into individual standard NuSMV variables
        for (i, element) in self.elements.iter().enumerate() {
            writeln!(f, "{}{} : {};", name, i, element)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ListIdx {
    pub name: String,
    pub idx: usize,
}

impl fmt::Display for ListIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.strip_suffix('s').unwrap_or(&self.name).to_string();
        write!(f, "{}{}", name, self.idx)
    }
}