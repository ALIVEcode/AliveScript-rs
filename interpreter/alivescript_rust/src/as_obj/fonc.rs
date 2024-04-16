use std::{fmt::Display, ptr};

use derive_getters::Getters;
use derive_new::new;

use crate::{
    as_obj::{ASEnv, ASObj, ASType, ASVar},
    ast::{Expr, Stmt},
};

#[derive(Debug, Clone, new, Getters)]
pub struct ASFonc {
    name: Option<String>,
    docs: Option<String>,
    params: Vec<ASFnParam>,
    body: Vec<Box<Stmt>>,
    return_type: ASType,
    env: ASEnv,
}

impl PartialEq for ASFonc {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Display for ASFonc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_string = format!(
            "{}({}) -> {}",
            self.name.as_ref().unwrap_or(&"".into()),
            self.params
                .iter()
                .map(ASFnParam::to_string)
                .collect::<Vec<String>>()
                .join(", "),
            self.return_type
        );
        write!(f, "{}", to_string)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ASFnParam {
    pub name: String,
    pub static_type: ASType,
    pub default_value: Option<Box<Expr>>,
}

impl ASFnParam {
    pub fn new(
        name: impl ToString,
        static_type: Option<ASType>,
        default_value: Option<Box<Expr>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            static_type: static_type.into(),
            default_value,
        }
    }

    pub fn native(name: impl ToString, static_type: ASType, default_value: Option<ASObj>) -> Self {
        Self::new(
            name,
            Some(static_type),
            default_value.map(|val| Expr::literal(val)),
        )
    }

    pub fn to_asvar(&self) -> ASVar {
        ASVar::new(self.name.clone(), Some(self.static_type.clone()), false)
    }
}

impl Display for ASFnParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.static_type)
    }
}
