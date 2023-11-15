use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, BitXor, Div, Mul, Rem, Sub},
    str::FromStr,
    sync::Arc,
};

use derive_new::new;

use crate::{
    ast::{Expr, Stmt},
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
    // A placeholder value for representing the absence of values
    ASNoValue,

    ASPaire {
        key: Box<ASObj>,
        val: Box<ASObj>,
    },

    ASTuple(Vec<ASObj>),

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
        fields: Vec<ASStructField>,
    },

    ASModule {
        env: Arc<ASScope>,
    },

    ASStructInst {
        struct_parent: Box<ASObj>,
        env: ASScope,
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
        docs: Option<&str>,
        params: Vec<ASFnParam>,
        body: fn(&mut Runner) -> Result<Option<ASObj>, ASErreurType>,
        return_type: ASType,
    ) -> ASObj {
        Self::ASFonc {
            name: Some(name.into()),
            docs: docs.map(|docs| docs.into()),
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
            ASNul => false,
            ASListe(l) => !l.is_empty(),
            ASDict(d) => !d.is_empty(),
            _ => true,
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

    pub fn contains(&self, rhs: &Self) -> Result<bool, ASErreurType> {
        use ASObj::*;

        match (self, rhs) {
            (ASPaire { key, val }, rhs) => todo!(),
            (ASTexte(s), ASTexte(sub_s)) => Ok(s.contains(sub_s)),
            (ASListe(l), rhs) => Ok(l.contains(rhs)),
            (ASDict(d), rhs) => Ok(d
                .into_iter()
                .find(|el| matches!(el, ASPaire { key, val } if key.as_ref() == rhs))
                .is_some()),

            (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            (ASStructure { name, fields }, _) => todo!("Check présense du field?"),
            (ASModule { env }, _) => todo!(),
            _ => Err(ASErreurType::new_erreur_operation(
                "dans".into(),
                self.get_type(),
                rhs.get_type(),
            )),
        }
    }

    pub fn repr(&self) -> String {
        use ASObj::*;

        match self {
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

impl BitXor for ASObj {
    type Output = Result<ASObj, ASErreurType>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x ^ y)),
            (ASBooleen(x), ASBooleen(y)) => Ok(ASBooleen(x ^ y)),
            _ => Err(ASErreurType::new_erreur_operation(
                "xor".into(),
                type_1,
                type_2,
            )),
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
            ASBooleen(b) => if *b { "vrai" } else { "faux" }.into(),
            ASNul => "nul".into(),
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
                    name.as_ref().unwrap_or(&"".into()),
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

#[derive(Debug, PartialEq, Clone)]
pub struct ASStructField {
    pub name: String,
    pub vis: ASStructFieldVis,
    pub static_type: ASType,
    pub default_value: Option<Box<Expr>>,
    pub is_const: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ASStructFieldVis {
    Publique,
    Privee,
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
    Structure,

    Module,
    Objet(String),

    Union(Vec<ASType>),
    Tuple(Vec<ASType>),
    Optional(Box<ASType>),
}

impl ASType {
    pub fn default_value(&self) -> ASObj {
        use ASObj::*;
        use ASType::*;

        match self {
            Tout => ASEntier(0),
            Rien => todo!(),
            Nul => ASNul,
            Entier => ASEntier(0),
            Decimal => ASDecimal(0f64),
            Booleen => ASBooleen(false),
            Texte => ASTexte("".into()),
            Liste => ASListe(vec![]),
            Paire => ASPaire {
                key: Box::new(ASNul),
                val: Box::new(ASNul),
            },
            Dict => ASDict(vec![]),
            Fonction => todo!(),
            Structure => todo!(),
            Module => todo!(),
            Objet(_) => todo!(),
            Union(_) => todo!(),
            Tuple(_) => todo!(),
            Optional(_) => todo!(),
        }
    }

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

    pub fn optional(type1: ASType) -> ASType {
        ASType::Optional(Box::new(type1))
    }

    pub fn any() -> ASType {
        ASType::optional(ASType::Tout)
    }

    pub fn is_tout(&self) -> bool {
        use ASType::*;

        match self {
            Union(types) => types.contains(&ASType::Tout),
            Optional(t) => t.is_tout(),
            _ => self == &ASType::Tout,
        }
    }

    pub fn is_primitif(&self) -> bool {
        use ASType::*;
        matches!(self, Entier | Decimal | Texte | Booleen | Nul | Optional(_))
    }

    pub fn type_match(type1: &ASType, type2: &ASType) -> bool {
        use ASType::*;

        match (type1, type2) {
            (t1, t2) if t1 == t2 => true,
            (Tout, other) | (other, Tout) => other != &Rien && other != &Nul,

            (Optional(t), other) | (other, Optional(t)) => {
                other == &Nul || ASType::type_match(t.as_ref(), other)
            }

            (Union(types), other) | (other, Union(types)) => {
                types.iter().any(|t| ASType::type_match(t, &other))
            }

            (Decimal, Entier) => true,
            _ => false,
        }
    }

    pub fn convert_to_obj(&self, s: String) -> Result<ASObj, ASErreurType> {
        use ASType::*;

        let s = s.trim().to_string();

        match self {
            Texte | Tout => Ok(ASObj::ASTexte(s)),
            Optional(t) if t.as_ref() == &Tout => Ok(ASObj::ASTexte(s)),
            Nul => {
                if s.eq_ignore_ascii_case("nul") {
                    Ok(ASObj::ASNul)
                } else {
                    Err(ASErreurType::ErreurConversionType {
                        type_cible: Nul,
                        texte: s.clone(),
                    })
                }
            }
            Entier => {
                if let Ok(i) = s.parse() {
                    Ok(ASObj::ASEntier(i))
                } else {
                    Err(ASErreurType::ErreurConversionType {
                        type_cible: Entier,
                        texte: s.clone(),
                    })
                }
            }
            Decimal => {
                if let Ok(i) = s.parse() {
                    Ok(ASObj::ASDecimal(i))
                } else {
                    Err(ASErreurType::ErreurConversionType {
                        type_cible: Decimal,
                        texte: s.clone(),
                    })
                }
            }
            Booleen => {
                if s.eq_ignore_ascii_case("vrai") {
                    Ok(ASObj::ASBooleen(true))
                } else if s.eq_ignore_ascii_case("faux") {
                    Ok(ASObj::ASBooleen(false))
                } else {
                    Err(ASErreurType::ErreurConversionType {
                        type_cible: Booleen,
                        texte: s.clone(),
                    })
                }
            }
            Optional(t) => {
                if s == "" {
                    Ok(ASObj::ASNul)
                } else {
                    t.convert_to_obj(s)
                }
            }
            t => Err(ASErreurType::ErreurType {
                type_attendu: ASType::union(vec![Entier, Decimal, Nul, Booleen, Texte]),
                type_obtenu: t.clone(),
            }),
        }
    }
}

impl From<Option<ASType>> for ASType {
    fn from(value: Option<ASType>) -> Self {
        value.unwrap_or(ASType::any())
    }
}

impl FromStr for ASType {
    type Err = LexicalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "entier" => Ok(Self::Entier),
            "decimal" | "décimal" => Ok(Self::Decimal),
            "booleen" | "booléen" => Ok(Self::Booleen),
            "nombre" => Ok(Self::nombre()),
            "iterable" | "itérable" => Ok(Self::iterable()),
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
            Rien => "rien".into(),

            Nul => "nul".into(),
            Optional(t) => format!("{}?", t),

            Entier => "entier".into(),
            Decimal => "décimal".into(),
            Booleen => "booléen".into(),
            Texte => "texte".into(),

            Liste => "liste".into(),
            Dict => "dict".into(),
            Paire => "paire".into(),

            Module => "module".into(),
            Objet(o) => o.clone(),

            Fonction => "fonction".into(),
            Structure => "structure".into(),

            Union(types) => types
                .iter()
                .map(Self::to_string)
                .collect::<Vec<String>>()
                .join(" | "),

            Tuple(types) => format!(
                "({})",
                types
                    .iter()
                    .map(Self::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
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

    pub fn declare(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.insert(var, val)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, (ASVar, ASObj)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> std::collections::hash_map::IntoIter<String, (ASVar, ASObj)> {
        self.0.into_iter()
    }

    pub fn assign(&mut self, var_name: &String, val: ASObj) -> Option<(ASVar, ASObj)> {
        let var = self.get(var_name).unwrap().0.clone();
        self.insert(var, val)
    }

    pub fn assign_strict(
        &mut self,
        var_name: &String,
        val: ASObj,
    ) -> Result<Option<(ASVar, ASObj)>, ASErreurType> {
        let var = &self.get(var_name).unwrap().0;
        if var.is_const() {
            Err(ASErreurType::new_affectation_constante(var_name.clone()))
        } else if !var.type_match(&val.get_type()) {
            Err(ASErreurType::new_erreur_type(
                var.get_type().clone(),
                val.get_type(),
            ))
        } else {
            Ok(self.insert(var.clone(), val))
        }
    }

    pub fn assign_type_strict(
        &mut self,
        var_name: &String,
        val: ASObj,
    ) -> Result<Option<(ASVar, ASObj)>, ASErreurType> {
        let Some(var) = &self.get(var_name) else {
            return Err(ASErreurType::new_variable_inconnue(var_name.clone()));
        };
        let var = &var.0;
        if !var.type_match(&val.get_type()) {
            Err(ASErreurType::new_erreur_type(
                var.get_type().clone(),
                val.get_type(),
            ))
        } else {
            Ok(self.insert(var.clone(), val))
        }
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

    pub fn declare_strict(
        &mut self,
        var: ASVar,
        val: ASObj,
    ) -> Result<Option<(ASVar, ASObj)>, ASErreurType> {
        let var_name = var.get_name();
        if self
            .0
            .last_mut()
            .unwrap()
            .get(var_name)
            .is_some_and(|(v, _)| v.is_const())
        {
            Err(ASErreurType::new_affectation_constante(var_name.clone()))
        } else {
            Ok(self.0.last_mut().unwrap().insert(var, val))
        }
    }

    pub fn assign(&mut self, var_name: &String, val: ASObj) -> Option<(ASVar, ASObj)> {
        let scope = self.get_env_of_var(var_name);
        let var = scope.get(var_name).unwrap().0.clone();
        self.get_env_of_var(var_name).insert(var, val)
    }

    pub fn assign_strict(
        &mut self,
        var_name: &String,
        val: ASObj,
    ) -> Result<Option<(ASVar, ASObj)>, ASErreurType> {
        let scope = self.get_env_of_var(var_name);
        scope.assign_strict(var_name, val)
    }
}

#[derive(Debug, PartialEq, Clone, new)]
pub enum ASErreurType {
    VariableInconnue {
        var_name: String,
    },
    AffectationConstante {
        var_name: String,
    },
    ErreurType {
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    ErreurTypeRetour {
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    ErreurConversionType {
        type_cible: ASType,
        texte: String,
    },
    ErreurTypeAppel {
        func_name: Option<String>,
        param_name: String,
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    ErreurOperation {
        op: String,
        lhs_type: ASType,
        rhs_type: ASType,
    },
    ErreurClef {
        mauvaise_clef: ASObj,
    },
    ErreurAccessPropriete {
        obj: ASObj,
        prop: String,
    },
    ErreurProprietePasInit {
        obj: ASObj,
        prop: String,
    },
    SuiteInvalide {
        start: ASObj,
        end: ASObj,
        step: ASObj,
    },
    ErreurValeur {
        raison: Option<String>,
        valeur: ASObj,
    },
}

impl Display for ASErreurType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ASErreurType::*;

        let to_string = match self {
            VariableInconnue { var_name } => format!("Variable inconnue '{}'", var_name),
            AffectationConstante { var_name } => format!("Impossible de changer la valeur d'une constante: '{}'", var_name),

            ErreurConversionType { type_cible, texte } => format!("Impossible de convertir \"{}\" en {}", texte, type_cible),

            ErreurValeur { raison, valeur } => format!("Valeur invalide: {}. {}", valeur, raison.clone().unwrap_or_default()),

            ErreurType {
                type_obtenu,
                type_attendu,
            } => format!(
                "Erreur de type. Type attendu: '{}', type obtenu: '{}'",
                type_attendu, type_obtenu,
            ),

            ErreurTypeRetour {
                type_obtenu,
                type_attendu,
            } => format!(
                "Mauvais type de retour. Attendu: '{}', Obtenu: '{}'",
                type_attendu, type_obtenu
            ),

            ErreurTypeAppel {
                func_name,
                param_name,
                type_obtenu,
                type_attendu,
            } => format!(
                "Dans la fonction {}: Type de l'argument invalide pour le paramètre '{}'. Attendu: '{}', obtenu: '{}'",
                func_name.as_ref().unwrap_or(&"<sans-nom>".to_string()), 
                param_name,
                type_attendu,
                type_obtenu,
            ),

            ErreurOperation {
                op,
                lhs_type,
                rhs_type,
            } => format!(
                 "Opération '{}' non définie pour les valeurs de type '{}' et de type '{}'",
                 op,
                 lhs_type,
                 rhs_type,
            ),

            ErreurClef { mauvaise_clef } => format!("La clef {} n'est pas dans le dictionnaire", mauvaise_clef.repr()),

            ErreurAccessPropriete { obj, prop } => format!("La propriété {} n'existe pas dans {}", obj, prop),

            ErreurProprietePasInit { obj, prop } => format!("La propriété {} n'est pas initialisé dans {}", obj, prop),

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
