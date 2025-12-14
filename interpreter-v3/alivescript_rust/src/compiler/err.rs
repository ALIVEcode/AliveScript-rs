use pest::error::{Error as PestError, ErrorVariant as PestErrorVariant};
use thiserror::Error;

use crate::{Rule, compiler::value::BaseType};

#[derive(Debug, Error)]
pub enum CompilationError {
    #[error(transparent)]
    LexerError(PestError<Rule>),

    #[error("{message}. Attendu type {expected}, obtenu {actual}")]
    UnexpectedTypeError {
        expected: BaseType,
        actual: BaseType,
        message: String,
    },

    #[error("{0}")]
    CompilationError(String),
}

impl CompilationError {
    pub fn generic_error(msg: impl ToString) -> Self {
        Self::CompilationError(msg.to_string())
    }
}

impl From<PestError<Rule>> for CompilationError {
    fn from(val: PestError<Rule>) -> Self {
        CompilationError::LexerError(val)
    }
}
