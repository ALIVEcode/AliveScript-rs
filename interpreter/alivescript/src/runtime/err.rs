use thiserror::Error;

use crate::compiler::value::Type;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Erreur de champs: {0}")]
    FieldError(String),

    #[error("Erreur de valeur: {0}")]
    ValueError(String),

    #[error("Erreur type: {0}")]
    TypeError(String),

    #[error("Erreur lors de l'appel de la fonction '{func_name}': {msg}")]
    CallError { func_name: String, msg: String },

    #[error("Erreur lors du chargement du module {module_name}: {message}")]
    ModuleLoadError {
        module_name: String,
        message: String,
    },

    #[error("Impossibe d'affecter la variable '{var_name}' puisque c'est une constante.")]
    AssignToConstVar { var_name: String },

    #[error("Impossibe d'affecter le champs '{field_name}' puisque c'est une constante.")]
    AssignToConstField { field_name: String },

    #[error("Erreur d'affirmation: {0}")]
    AssertionError(String),

    #[error("Explosion de la pile d'appel: {0}")]
    StackOverflow(String),

    #[error("Erreur de permission ({permission_name}): {msg}")]
    PermissionError {
        permission_name: String,
        msg: String,
    },

    #[error("Erreur lors de l'exécution:\n{0}")]
    RuntimeError(String),
}

impl RuntimeError {
    pub fn module_load_error(module_name: impl ToString, msg: impl ToString) -> Self {
        Self::ModuleLoadError {
            module_name: module_name.to_string(),
            message: msg.to_string(),
        }
    }

    pub fn stackoverflow_error(msg: impl ToString) -> Self {
        Self::StackOverflow(msg.to_string())
    }

    pub fn value_error(msg: impl ToString) -> Self {
        Self::ValueError(msg.to_string())
    }

    pub fn call_error(func_name: &str, msg: impl ToString) -> Self {
        Self::CallError {
            func_name: func_name.to_string(),
            msg: msg.to_string(),
        }
    }

    pub fn assertion_error(msg: impl ToString) -> Self {
        Self::AssertionError(msg.to_string())
    }

    pub fn type_error(msg: impl ToString) -> Self {
        Self::ValueError(msg.to_string())
    }

    pub fn invalid_field(obj_str: &str, field_name: &str) -> Self {
        Self::FieldError(format!(
            "le champs {} n'existe pas dans l'objet {}",
            field_name, obj_str
        ))
    }

    pub fn assign_to_const(var_name: impl ToString) -> Self {
        Self::AssignToConstVar {
            var_name: var_name.to_string(),
        }
    }

    pub fn assign_to_const_field(field_name: impl ToString) -> Self {
        Self::AssignToConstField {
            field_name: field_name.to_string(),
        }
    }

    pub fn invalid_struct(ty: Type) -> Self {
        Self::ValueError(format!(
            "impossible de construire une valeur de type '{}'. Seule les structures peuvent être construite",
            ty
        ))
    }

    pub fn missing_struct_fields(struct_name: &str, missing_fields: &[String]) -> Self {
        Self::ValueError(format!(
            "lors de la construction de la structure '{}', certains champs n'ont pas reçu de valeurs: {:?}. Spécifiez une valeur dans la construction ou ajoutez une valeur par défaut à ces champs",
            struct_name, missing_fields
        ))
    }

    pub fn invalid_op(op: &str, lhs: Type, rhs: Type) -> Self {
        Self::TypeError(format!(
            "opération '{}' non supporté pour les opérandes: {} et {}",
            op, lhs, rhs,
        ))
    }

    pub fn invalid_arg_type(
        func: &str,
        param_name: &str,
        param_type: Type,
        arg_type: Type,
    ) -> Self {
        Self::TypeError(format!(
            "dans la fonction '{func}', le paramètre '{param_name}' est de type '{param_type}', mais l'argument passé est de type '{arg_type}'",
        ))
    }

    pub fn permission_error(permission_name: impl ToString, msg: impl ToString) -> Self {
        Self::PermissionError {
            permission_name: permission_name.to_string(),
            msg: msg.to_string(),
        }
    }

    pub fn generic_err(msg: impl ToString) -> Self {
        Self::RuntimeError(msg.to_string())
    }
}
