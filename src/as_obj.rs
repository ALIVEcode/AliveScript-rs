use std::str::FromStr;

use crate::{ast::Stmt, lexer::LexicalError};

#[derive(Debug, PartialEq)]
pub enum ASObj {
    ASEntier(i64),
    ASDecimal(f64),
    ASBooleen(bool),
    ASNul,

    ASPaire {
        key: Box<ASObj>,
        val: Box<ASObj>,
    },

    ASTexte(String),
    ASListe(Vec<ASObj>),

    ASDict(Vec<ASObj>),

    ASFonc {
        name: String,
        params: Vec<(
            ASVar,         // Variable
            Option<ASObj>, // Valeur par défaut
        )>,
        body: Vec<Stmt>,
        return_type: Option<ASType>,
    },
}

#[derive(Debug, PartialEq)]
pub struct ASVar {
    name: String,
    var_type: Option<ASType>,
    is_const: bool,
}

#[derive(Debug, PartialEq)]
pub enum ASType {
    Entier,
    Decimal,
    Texte,
    Objet(String),
    Union(Vec<ASType>),
}

impl FromStr for ASType {
    type Err = LexicalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "entier" => Ok(Self::Entier),
            "decimal" => Ok(Self::Decimal),
            "nombre" => Ok(Self::Union(vec![Self::Entier, Self::Decimal])),
            "texte" => Ok(Self::Texte),
            _ => Err(LexicalError::InvalidToken)
        }
    }
}
