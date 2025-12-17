use colored::Colorize;
use std::fmt::Display;

use pest::{
    Span,
    error::{Error as PestError, InputLocation, LineColLocation},
};
use thiserror::Error;

use crate::{Rule, compiler::value::Type};

#[derive(Debug, Error)]
pub struct CompilationError {
    kind: CompilationErrorKind,
    location: InputLocation,
    /// Line/column within the input string
    line_col: LineColLocation,
    path: Option<String>,
    line: String,
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start_ln, start_col, end_ln, end_col) = match &self.line_col {
            LineColLocation::Pos(line_col) => (line_col.0, line_col.1, line_col.0, line_col.1),
            LineColLocation::Span(start_line_col, end_line_col) => (
                start_line_col.0,
                start_line_col.1,
                end_line_col.0,
                end_line_col.1,
            ),
        };
        write!(
            f,
            "{}",
            format!("erreur: {}\n", self.kind.to_string()).red().bold()
        )?;
        for (i, line) in self.line.split("\n").enumerate() {
            write!(f, "{}\n", line)?;
            if i + start_ln > end_ln {
                continue;
            }
            let line_lead_whitespace = line.len() - line.trim_start().len();
            let line_end_whitespace = line.len() - line.trim_end().len();
            if i == 0 {
                write!(
                    f,
                    "{}{}\n",
                    " ".repeat(start_col + line_lead_whitespace),
                    "^".repeat(
                        (if start_ln == end_ln {
                            end_col
                        } else {
                            line.len()
                        }) - start_col
                            - line_lead_whitespace
                            - line_end_whitespace
                    )
                    .yellow()
                    .bold()
                )?;
            } else if i + start_ln == end_ln {
                write!(
                    f,
                    "{}{}\n",
                    " ".repeat(line_lead_whitespace),
                    "^".repeat(end_col - line_lead_whitespace - line_end_whitespace)
                        .yellow()
                        .bold()
                )?;
            } else {
                write!(
                    f,
                    "{}{}\n",
                    " ".repeat(line_lead_whitespace),
                    "^".repeat(line.len() - line_lead_whitespace - line_end_whitespace)
                        .yellow()
                        .bold()
                )?;
            }
        }
        // write!(f, "{}{}", " ".repeat(start), "^".repeat(length))
        Ok(())
    }
}

impl<'a> From<PestError<Rule>> for CompilationError {
    fn from(val: PestError<Rule>) -> Self {
        Self {
            kind: CompilationErrorKind::LexerError(val.clone()),
            path: val.path().map(|p| p.to_string()),
            line: val.line().to_string(),
            location: val.location,
            line_col: val.line_col,
        }
    }
}

#[derive(Debug, Error)]
pub enum CompilationErrorKind {
    #[error(transparent)]
    LexerError(PestError<Rule>),

    #[error("{message}. Attendu type {expected}, obtenu {actual}")]
    UnexpectedTypeError {
        expected: Type,
        actual: Type,
        message: String,
    },

    #[error("Impossibe d'affecter la variable '{var_name}' puisque c'est une constante.")]
    AssignToConst { var_name: String },

    #[error("{0}")]
    CompilationError(String),
}

impl CompilationErrorKind {
    pub fn to_error(self, span: Span) -> CompilationError {
        CompilationError {
            kind: self,
            location: InputLocation::Span((span.start(), span.end_pos().pos())),
            line_col: LineColLocation::Span(span.start_pos().line_col(), span.end_pos().line_col()),
            path: None,
            line: span.lines().collect::<Vec<_>>().join(""),
        }
    }

    pub fn generic_error(msg: impl ToString) -> Self {
        Self::CompilationError(msg.to_string())
    }

    pub fn assign_to_const(var_name: impl ToString) -> Self {
        Self::AssignToConst {
            var_name: var_name.to_string(),
        }
    }
}

impl From<PestError<Rule>> for CompilationErrorKind {
    fn from(val: PestError<Rule>) -> Self {
        CompilationErrorKind::LexerError(val)
    }
}
