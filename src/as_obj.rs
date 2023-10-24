use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

use crate::{ast::Stmt, lexer::LexicalError};

#[derive(Debug, PartialEq, Clone)]
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

    ASFoncInstance {
        base: Box<ASObj>, // ASFonc
        env: HashMap<String, (ASVar, ASObj)>,
    },
}

impl ASObj {
    pub fn get_type(&self) -> ASType {
        use ASObj::*;

        match self {
            ASEntier(..) => ASType::Entier,
            ASDecimal(..) => ASType::Decimal,
            ASTexte(..) => ASType::Texte,
            _ => todo!(),
        }
    }
}

impl Add for ASObj {
    type Output = ASObj;

    fn add(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (ASTexte(s), any) => ASTexte(format!("{}{}", s, any.to_string())),
            (any, ASTexte(s)) => ASTexte(format!("{}{}", any.to_string(), s)),
            (ASEntier(x), ASEntier(y)) => ASEntier(x + y),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x + y as f64),
            (ASEntier(x), ASDecimal(y)) => ASDecimal(x as f64 + y),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x + y),
            _ => unimplemented!(),
        }
    }
}

impl Sub for ASObj {
    type Output = ASObj;

    fn sub(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (ASTexte(s), ASTexte(s2)) => ASTexte(s.replace(s2.as_str(), "")),
            (ASEntier(x), ASEntier(y)) => ASEntier(x - y),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x - y as f64),
            (ASEntier(x), ASDecimal(y)) => ASDecimal(x as f64 - y),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x - y),
            _ => unimplemented!(),
        }
    }
}

impl Mul for ASObj {
    type Output = ASObj;

    fn mul(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (ASTexte(s), ASEntier(n)) => ASTexte(s.repeat(if n >= 0 { n as usize } else { 0 })),
            (ASEntier(x), ASEntier(y)) => ASEntier(x * y),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x * y as f64),
            (ASEntier(x), ASDecimal(y)) => ASDecimal(x as f64 * y),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x * y),
            _ => unimplemented!(),
        }
    }
}

impl Div for ASObj {
    type Output = ASObj;

    fn div(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => ASDecimal(x as f64 / y as f64),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x / y as f64),
            (ASEntier(x), ASDecimal(y)) => ASDecimal(x as f64 / y),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x / y),
            _ => unimplemented!(),
        }
    }
}

impl Display for ASObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ASObj::*;

        let to_string = match self {
            ASEntier(i) => i.to_string(),
            ASDecimal(d) => d.to_string(),
            ASTexte(s) => s.clone(),
            ASListe(v) => format!(
                "[{}]",
                v.iter()
                    .map(Self::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),

            _ => String::from("ASObj sans to_string"),
        };
        write!(f, "{}", to_string)
    }
}

#[derive(Debug, Hash, Eq, Clone, PartialEq)]
pub struct ASVar {
    name: String,
    static_type: Option<ASType>,
    is_const: bool,
}

impl PartialEq<String> for ASVar {
    fn eq(&self, other: &String) -> bool {
        &self.name == other
    }
}

impl ASVar {
    pub fn new(name: String, static_type: Option<ASType>, is_const: bool) -> Self {
        Self {
            name,
            static_type,
            is_const,
        }
    }

    pub fn type_match(&self, static_type: &ASType) -> bool {
        self.static_type.is_none() || self.static_type.as_ref().unwrap() == static_type
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
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
            _ => Err(LexicalError::InvalidToken),
        }
    }
}
