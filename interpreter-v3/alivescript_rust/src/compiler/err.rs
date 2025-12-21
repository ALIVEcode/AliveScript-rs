use colored::Colorize;
use logos::Source;
use std::fmt::{Display, format};

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

impl CompilationError {
    pub fn set_source_if_none(mut self, source: String) -> Self {
        self.path = Some(source);
        self
    }
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start_ln, start_col, mut end_ln, mut end_col) = match &self.line_col {
            LineColLocation::Pos(line_col) => (line_col.0, line_col.1, line_col.0, line_col.1),
            LineColLocation::Span(start_line_col, end_line_col) => (
                start_line_col.0,
                start_line_col.1 - 1,
                end_line_col.0,
                end_line_col.1 - 1,
            ),
        };
        if self
            .line
            .split("\n")
            .nth(end_ln - start_ln)
            .is_some_and(|s| s == "")
        {
            end_ln -= 1;
            end_col = self
                .line
                .strip_suffix("\n")
                .and_then(|s| s.split("\n").last().map(|ln| ln.len()))
                .unwrap_or(0);
        }

        write!(
            f,
            "{}{}{}{}",
            "erreur de compilation".red().bold(),
            format!(": {}\n", self.kind.to_string()).bold(),
            "  --> ".blue().bold(),
            format!(
                "{}:{}:{}\n",
                self.path.as_ref().unwrap_or(&"script".to_string()),
                start_ln,
                start_col + 1
            )
        )?;

        let max_nb_digits = end_ln.to_string().len();

        let line_prefix = format!("{} |\n", " ".repeat(max_nb_digits))
            .bright_blue()
            .bold();
        write!(f, "{}", line_prefix)?;

        for (i, line) in self.line.split("\n").enumerate() {
            if i + start_ln > end_ln {
                break;
            }

            let real_line = i + start_ln;

            let line_prefix = format!("{:>max_nb_digits$} |    ", real_line)
                .bright_blue()
                .bold();
            write!(f, "{}{}\n", line_prefix, line)?;

            let line_prefix = format!("{} |    ", " ".repeat(max_nb_digits))
                .bright_blue()
                .bold();
            write!(f, "{}", line_prefix)?;

            let line_lead_whitespace = line.len() - line.trim_start().len();
            let line_end_whitespace = line.len() - line.trim_end().len();
            if i == 0 {
                write!(
                    f,
                    "{}{}\n",
                    " ".repeat(start_col + line_lead_whitespace),
                    "^".repeat(
                        ((if start_ln == end_ln {
                            end_col
                        } else {
                            line.len()
                        })
                        .max(start_col + line_lead_whitespace + line_end_whitespace))
                            - start_col
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

    #[error(
        "Bloc d'implémentation invalide pour la structure '{struct_name}' (il doit se retrouver au même endroit que la définition de la structure)."
    )]
    InvalidImplBlock { struct_name: String },

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

    pub fn invalid_impl_block(struct_name: impl ToString) -> Self {
        Self::InvalidImplBlock {
            struct_name: struct_name.to_string(),
        }
    }
}

impl From<PestError<Rule>> for CompilationErrorKind {
    fn from(val: PestError<Rule>) -> Self {
        CompilationErrorKind::LexerError(val)
    }
}
