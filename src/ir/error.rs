use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LowerError {
    ListNotDefined {
        name: String,
    },
    IndexOutOfBounds {
        list: String,
        index: i32,
        length: usize,
    },
    UnknownLoopVariable {
        name: String,
    },
}

impl Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ListNotDefined { name } => write!(f, "list {} is not defined", name),
            Self::IndexOutOfBounds {
                list,
                index,
                length,
            } => {
                write!(
                    f,
                    "index {} is out of bounds for list {} of length {}",
                    index, list, length
                )
            }
            Self::UnknownLoopVariable { name } => {
                write!(f, "unknown loop variable in list index: {}", name)
            }
        }
    }
}

impl Error for LowerError {}
