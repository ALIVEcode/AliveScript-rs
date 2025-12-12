use pest::error::{Error as PestError, ErrorVariant as PestErrorVariant};
use thiserror::Error;

use crate::Rule;

#[derive(Debug, Error)]
pub enum CompilationError {
    #[error(transparent)]
    LexerError(PestError<Rule>),
}

impl From<PestError<Rule>> for CompilationError {
    fn from(val: PestError<Rule>) -> Self {
        CompilationError::LexerError(val)
    }
}
