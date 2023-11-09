use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, Div, Mul, Rem, Sub},
    str::FromStr,
    sync::Arc,
};

use derive_new::new;

use crate::{
    ast::{Expr, Stmt, StructField},
    data::Data,
    lexer::LexicalError,
    runner::Runner,
};

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
    /// Les éléments du vecteur sont tous garanti d'être des [`ASObj::ASPaire`]
    ASDict(Vec<ASObj>),

    ASFonc {
        name: Option<String>,
        docs: Option<String>,
        params: Vec<ASFnParam>,
        body: Vec<Box<Stmt>>,
        return_type: ASType,
    },

    ASStructure {
        name: String,
        fields: Vec<StructField>,
    },

    ASModule {
        env: Arc<ASScope>,
    },
}

impl ASObj {
    pub fn asfonc(
        name: Option<String>,
        docs: Option<String>,
        params: Vec<ASFnParam>,
        body: Vec<Box<Stmt>>,
        return_type: Option<ASType>,
    ) -> Self {
        Self::ASFonc {
            name,
            docs,
            params,
            body,
            return_type: return_type.into(),
        }
    }

    pub fn native_fn(
        name: &str,
        docs: Option<String>,
        params: Vec<ASFnParam>,
        body: fn(&mut Runner) -> Option<ASObj>,
        return_type: ASType,
    ) -> ASObj {
        Self::ASFonc {
            name: Some(name.into()),
            docs,
            params,
            body: vec![Stmt::native_fn(body)],
            return_type,
        }
    }

    pub fn get_type(&self) -> ASType {
        use ASObj::*;

        match self {
            ASEntier(..) => ASType::Entier,
            ASDecimal(..) => ASType::Decimal,
            ASTexte(..) => ASType::Texte,
            ASNul => ASType::Nul,
            ASBooleen(..) => ASType::Booleen,
            ASListe(..) => ASType::Liste,
            ASFonc { .. } => ASType::Fonction,
            ASPaire { .. } => ASType::Paire,
            ASDict(..) => ASType::Dict,
            as_type => todo!("Type inconnue {:?}", as_type),
        }
    }

    pub fn to_bool(&self) -> bool {
        use ASObj::*;

        match self {
            ASEntier(x) => *x != 0,
            ASDecimal(x) => *x != 0f64,
            ASTexte(s) => !s.is_empty(),
            ASBooleen(b) => *b,
            _ => false,
        }
    }

    pub fn div_int(&self, rhs: Self) -> ASObj {
        use ASObj::*;

        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => ASEntier(x / y),
            (ASDecimal(x), ASEntier(y)) => ASEntier(*x as i64 / y),
            (ASEntier(x), ASDecimal(y)) => ASEntier(x / y as i64),
            (ASDecimal(x), ASDecimal(y)) => ASEntier(*x as i64 / y as i64),
            _ => unimplemented!(),
        }
    }

    pub fn repr(&self) -> String {
        use ASObj::*;

        match self {
            ASEntier(i) => i.to_string(),
            ASDecimal(d) => d.to_string(),
            ASTexte(s) => format!("\"{}\"", s),
            o => o.to_string(),
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

impl Rem for ASObj {
    type Output = ASObj;

    fn rem(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => ASEntier(x % y),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x % y as f64),
            (ASEntier(x), ASDecimal(y)) => ASDecimal(x as f64 % y),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x % y),
            _ => unimplemented!(),
        }
    }
}

impl PartialOrd for ASObj {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use ASObj::*;

        let (x, y) = match (self, other) {
            (ASEntier(x), ASEntier(y)) => (*x as f64, *y as f64),
            (ASDecimal(x), ASEntier(y)) => (*x, *y as f64),
            (ASEntier(x), ASDecimal(y)) => (*x as f64, *y),
            (ASDecimal(x), ASDecimal(y)) => (*x, *y),
            _ => unimplemented!(),
        };

        x.partial_cmp(&y)
    }
}

impl Display for ASObj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ASObj::*;

        let to_string = match self {
            ASEntier(i) => i.to_string(),
            ASDecimal(d) => d.to_string(),
            ASTexte(s) => s.clone(),
            ASPaire { key, val } => format!("{}: {}", key.repr(), val.repr()),
            ASListe(v) => format!(
                "[{}]",
                v.iter().map(Self::repr).collect::<Vec<String>>().join(", ")
            ),
            ASDict(v) => format!(
                "{{{}}}",
                v.iter().map(Self::repr).collect::<Vec<String>>().join(", ")
            ),
            ASFonc {
                name,
                docs,
                params,
                body,
                return_type,
            } => {
                format!(
                    "{}({}) -> {}",
                    name.as_ref().unwrap_or(&"".to_string()),
                    params
                        .iter()
                        .map(ASFnParam::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                    return_type
                )
            }
            _ => String::from("ASObj sans to_string"),
        };
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

    pub fn to_asvar(&self) -> ASVar {
        ASVar::new(self.name.clone(), Some(self.static_type.clone()), false)
    }
}

impl Display for ASFnParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.static_type)
    }
}

#[derive(Debug, Hash, Eq, Clone, PartialEq)]
pub struct ASVar {
    name: String,
    static_type: ASType,
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
            static_type: static_type.into(),
            is_const,
        }
    }

    pub fn new_with_value(
        name: impl ToString,
        static_type: Option<ASType>,
        is_const: bool,
        value: ASObj,
    ) -> (Self, ASObj) {
        (Self::new(name.to_string(), static_type, is_const), value)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_type(&self) -> &ASType {
        &self.static_type
    }

    pub fn is_const(&self) -> bool {
        self.is_const
    }

    pub fn type_match(&self, static_type: &ASType) -> bool {
        ASType::type_match(&self.static_type, static_type)
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum ASType {
    /// Type englobant tous les autres types, sauf [`ASType::Rien`] et [`ASType::Nul`]
    Tout,
    /// Type de retour d'une fonction qui ne retourne rien.
    /// Peut seulement être placé sur un retour de fonction.
    Rien,

    /// Type représentant l'absence d'une valeur.
    Nul,

    Entier,
    Decimal,
    Booleen,
    Texte,

    Liste,
    Paire,
    Dict,

    Fonction,

    Module,
    Objet(String),

    Union(Vec<ASType>),
}

impl ASType {
    pub fn nombre() -> ASType {
        ASType::Union(vec![Self::Entier, Self::Decimal])
    }

    pub fn iterable() -> ASType {
        ASType::Union(vec![Self::Liste, Self::Texte])
    }

    pub fn union(types: Vec<ASType>) -> ASType {
        if types.len() == 1 {
            types.into_iter().nth(0).unwrap()
        } else {
            ASType::Union(
                types
                    .into_iter()
                    .flat_map(|t| match t {
                        ASType::Union(as_types) => as_types,
                        t => vec![t],
                    })
                    .collect(),
            )
        }
    }

    pub fn union_of(type1: ASType, type2: ASType) -> ASType {
        ASType::union(vec![type1, type2])
    }

    fn is_tout(&self) -> bool {
        use ASType::*;

        match self {
            Union(types) => types.contains(&ASType::Tout),
            _ => self == &ASType::Tout,
        }
    }

    pub fn type_match(type1: &ASType, type2: &ASType) -> bool {
        use ASType::*;

        if type1.is_tout() || type2.is_tout() || type1 == type2 {
            return true;
        }

        match (type1, type2) {
            (Union(types), other) | (other, Union(types)) => {
                types.iter().any(|t| ASType::type_match(t, &other))
            }
            (Entier, Decimal) => true,
            _ => false,
        }
    }
}

impl From<Option<ASType>> for ASType {
    fn from(value: Option<ASType>) -> Self {
        value.unwrap_or(ASType::Tout)
    }
}

impl FromStr for ASType {
    type Err = LexicalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "entier" => Ok(Self::Entier),
            "decimal" => Ok(Self::Decimal),
            "nombre" => Ok(Self::nombre()),
            "iterable" => Ok(Self::iterable()),
            "texte" => Ok(Self::Texte),
            "liste" => Ok(Self::Liste),
            "rien" => Ok(Self::Nul),
            "nul" => Ok(Self::Nul),
            "tout" => Ok(Self::Tout),
            other => Ok(Self::Objet(other.into())),
        }
    }
}

impl Display for ASType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ASType::*;

        let to_string = match self {
            Tout => "tout".into(),
            Nul => "nul".into(),
            Entier => "entier".into(),
            Decimal => "decimal".into(),
            Texte => "texte".into(),
            Liste => "liste".into(),
            Fonction => "fonction".into(),
            Union(types) if types.len() == 2 && types[1] == Nul => {
                format!("{}?", types[0])
            }
            Union(types) => types
                .iter()
                .map(Self::to_string)
                .collect::<Vec<String>>()
                .join(" | "),
            _ => todo!(),
        };
        write!(f, "{}", to_string)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASScope(HashMap<String, (ASVar, ASObj)>);

impl ASScope {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from(vars: Vec<(ASVar, ASObj)>) -> Self {
        Self(HashMap::from_iter(
            vars.into_iter()
                .map(|(var, val)| (var.get_name().clone(), (var, val))),
        ))
    }

    pub fn get(&self, var_name: &String) -> Option<&(ASVar, ASObj)> {
        self.0.get(var_name)
    }

    pub fn insert(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.0.insert(var.get_name().clone(), (var, val))
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, (ASVar, ASObj)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> std::collections::hash_map::IntoIter<String, (ASVar, ASObj)> {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASEnv(Vec<ASScope>);

impl ASEnv {
    pub fn new() -> Self {
        Self(vec![ASScope::new()])
    }

    fn get_env_of_var(&mut self, var_name: &String) -> &mut ASScope {
        self.0
            .iter_mut()
            .rev()
            .find(|env| env.get(var_name).is_some())
            .unwrap()
    }

    pub fn get_curr_scope(&mut self) -> &mut ASScope {
        self.0.last_mut().unwrap()
    }

    pub fn push_scope(&mut self, scope: ASScope) {
        self.0.push(scope);
    }

    pub fn pop_scope(&mut self) -> Option<ASScope> {
        self.0.pop()
    }

    pub fn get_var(&self, var_name: &String) -> Option<&(ASVar, ASObj)> {
        self.0.iter().rev().find_map(|env| env.get(var_name))
    }

    pub fn get_value(&self, var_name: &String) -> Option<&ASObj> {
        Some(&self.0.iter().rev().find_map(|env| env.get(var_name))?.1)
    }

    pub fn declare(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.0.last_mut().unwrap().insert(var, val)
    }

    pub fn assign(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.get_env_of_var(var.get_name()).insert(var, val)
    }
}

#[derive(Debug, PartialEq, Clone, new)]
pub enum ASErreurType {
    VariableInconnue {
        var_name: String,
    },
    ErreurType {
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    SuiteInvalide {
        start: ASObj,
        end: ASObj,
        step: ASObj,
    },
}

impl Display for ASErreurType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ASErreurType::*;

        let to_string = match self {
            VariableInconnue { var_name } => format!("Variable inconnue '{}'", var_name),
            ErreurType {
                type_obtenu,
                type_attendu,
            } => format!(
                "Erreur de type. Type attendu: '{}', type obtenu: '{}'",
                type_attendu, type_obtenu,
            ),
            SuiteInvalide { start, end, step } => {
                format!("Suite invalide: {} .. {} bond {}", start, end, step)
            }
        };

        write!(f, "{}", to_string)
    }
}

#[derive(Debug, PartialEq, Clone, new)]
pub struct ASErreur {
    err_type: ASErreurType,
    ligne: usize,
}

impl Into<Data> for ASErreur {
    fn into(self) -> Data {
        Data::Erreur {
            texte: self.err_type.to_string(),
            ligne: self.ligne,
        }
    }
}
