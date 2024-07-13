use std::{cell::RefCell, fmt::Display, rc::Rc, str::FromStr};

use crate::as_obj::{ASErreurType, ASObj, ASVar};

use super::ASEnv;

#[derive(Debug, PartialEq, Clone)]
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
    Dict,

    Fonction,
    Classe,

    Module,
    Objet(String),
    ClasseInst,

    Union(Vec<ASType>),
    Array(Vec<ASType>),
    Optional(Box<ASType>),

    Type,

    Lit(Box<ASObj>),
}

macro_rules! decl_types {
    ($env:ident <- { $($name:ident = $value:expr;)* }) => {{
        $($env.declare(
                ASVar::new(format!("@type:{}", stringify!($name)), Some(ASType::Type), true),
                ASObj::ASTypeObj($value),
            );)*
    }}
}

impl ASType {
    pub fn load_builtin_types(env: &mut ASEnv) {
        use ASType as T;

        decl_types!(env <- {
            tout = T::Tout;
            rien = T::Rien;
            nul = T::Nul;

            entier = T::Entier;

            decimal = T::Decimal;
            décimal = T::Decimal;

            nombre = T::nombre();
            iterable = T::iterable();
            itérable = T::iterable();

            booleen = T::Booleen;
            booléen = T::Booleen;

            texte = T::Texte;

            liste = T::Liste;
            dict = T::Dict;

            fonction = T::Fonction;
            classe = T::Classe;
            module = T::Module;

            instance = T::ClasseInst;
            objet = T::union(vec![
                Self::ClasseInst,
                Self::Dict,
                Self::Classe,
            ]);

            type = T::Type;
        });
    }
}

impl ASType {
    pub fn default_value(&self) -> Result<ASObj, ASErreurType> {
        use crate::as_obj::ASDict as ASDictObj;
        use ASObj::*;
        use ASType as T;

        match self {
            T::Rien | T::Nul | T::Optional(_) => Ok(ASNul),
            T::Tout => Ok(ASEntier(0)),
            T::Entier => Ok(ASEntier(0)),
            T::Decimal => Ok(ASDecimal(0f64)),
            T::Booleen => Ok(ASBooleen(false)),
            T::Texte => Ok(ASTexte("".into())),
            T::Liste => Ok(ASListe(Rc::new(RefCell::new(vec![])))),
            T::Dict => Ok(ASDict(Rc::new(RefCell::new(ASDictObj::default())))),
            T::Type => todo!(),
            T::ClasseInst => todo!(),
            T::Fonction => todo!(),
            T::Classe => todo!(),
            T::Module => todo!(),
            T::Objet(_) => todo!(),
            T::Union(_) => todo!(),
            T::Array(_) => todo!(),
            T::Lit(o) => Ok(*o.clone()),
        }
    }

    pub fn nombre() -> ASType {
        ASType::Union(vec![Self::Entier, Self::Decimal])
    }

    pub fn iterable() -> ASType {
        ASType::Union(vec![
            Self::Liste,
            Self::Texte,
            Self::Dict,
            Self::Classe,
            Self::ClasseInst,
        ])
    }

    pub fn iterable_ordonne() -> ASType {
        ASType::Union(vec![
            Self::Liste,
            Self::Texte,
            Self::Classe,
            Self::ClasseInst,
        ])
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

            (Lit(o1), Lit(o2)) => o1 == o2,

            (Optional(t), other) | (other, Optional(t)) => {
                other == &Nul || ASType::type_match(t.as_ref(), other)
            }

            (Classe, Type) | (Type, Classe) => true,

            (Objet(..), ClasseInst) | (ClasseInst, Objet(..)) => true,

            (Union(types), other) | (other, Union(types)) => {
                types.iter().any(|t| ASType::type_match(t, &other))
            }

            (Liste, Array(..)) | (Array(..), Liste) => true,

            (Array(types1), Array(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                return types1
                    .iter()
                    .zip(types2)
                    .all(|(t1, t2)| ASType::type_match(t1, t2));
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
    type Err = ASErreurType;

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
            "fonction" => Ok(Self::Fonction),
            "classe" => Ok(Self::Classe),
            "module" => Ok(Self::Module),
            "instance" => Ok(Self::ClasseInst),
            "objet" => Ok(Self::union(vec![
                Self::ClasseInst,
                Self::Dict,
                Self::Classe,
            ])),
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

            Module => "module".into(),
            Objet(o) => o.clone(),

            Fonction => "fonction".into(),
            Classe => "classe".into(),

            ClasseInst => "objet".into(),

            Union(types) => types
                .iter()
                .map(Self::to_string)
                .collect::<Vec<String>>()
                .join(" | "),

            Array(types) => format!(
                "[{}]",
                types
                    .iter()
                    .map(Self::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),

            Type => "type".into(),

            Lit(o) => o.repr(),
        };
        write!(f, "{}", to_string)
    }
}
