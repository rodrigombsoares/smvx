use std::fmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub enum Chunk {
    List(ListDef),
    RawNusmv(String),
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chunk::RawNusmv(raw) => writeln!(f, "{}", raw),
            Chunk::List(list_def) => write!(f, "{}", list_def),
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