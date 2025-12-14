use pest::error::{Error as PestError, ErrorVariant as PestErrorVariant};
use thiserror::Error;

use crate::{Rule, compiler::value::BaseType};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Erreur type: {0}")]
    TypeError(String),

    #[error("Erreur lors de l'exécution: {0}")]
    RuntimeError(String),
}

impl RuntimeError {
    pub fn invalid_op(op: &str, lhs: BaseType, rhs: BaseType) -> Self {
        Self::TypeError(format!(
            "opération '{}' non supporté pour les opérandes: {} et {}",
            op, lhs, rhs,
        ))
    }

    pub fn invalid_arg_type(
        func: &str,
        param_name: &str,
        param_type: BaseType,
        arg_type: BaseType,
    ) -> Self {
        Self::TypeError(format!(
            "dans la fonction '{func}', le paramètre '{param_name}' est de type '{param_type}', mais l'argument passé est de type '{arg_type}'",
        ))
    }

    pub fn generic_err(msg: impl ToString) -> Self {
        Self::RuntimeError(msg.to_string())
    }
}
