mod classe;
mod dict;
mod env;
mod err;
mod fonc;
mod r#type;

pub use classe::*;
pub use dict::*;
pub use env::*;
pub use err::*;
pub use fonc::*;
pub use r#type::*;

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub},
    rc::Rc,
};

use derive_new::new;

use crate::{
    as_obj_utils::{Label, ObjPtr, RecursiveRepr, Seen},
    ast::Stmt,
    runner::Runner,
};

pub type ASResult<T> = Result<T, ASErreurType>;

/// w_lit means "With literal"
macro_rules! w_lit {
    ($type:expr, $self:ident) => {
        ASType::union_of($type, ASType::Lit(Box::new($self.clone())))
    };
}

#[derive(Debug, new)]
pub enum ASObj {
    // A placeholder value for representing the absence of values
    ASNoValue,

    ASEntier(i64),
    ASDecimal(f64),
    ASBooleen(bool),
    ASNul,

    ASTuple(Vec<ASObj>),

    ASTexte(String),
    ASListe(Rc<RefCell<Vec<ASObj>>>),

    ASDict(Rc<RefCell<ASDict>>),

    ASFonc(Rc<ASFonc>),

    ASMethode(Rc<ASMethode>),

    ASClasse(Rc<ASClasse>),

    ASModule {
        name: String,
        alias: Option<String>,
        env: Rc<RefCell<ASScope>>,
    },

    ASTypeObj(ASType),

    ASClasseInst(Rc<ASClasseInst>),

    ASErreur(Box<ASErreur>),
}

impl ASObj {
    pub fn liste(l: Vec<ASObj>) -> ASObj {
        ASObj::ASListe(Rc::new(RefCell::new(l)))
    }

    pub fn texte(s: impl ToString) -> ASObj {
        ASObj::ASTexte(s.to_string())
    }

    pub fn dict(d: ASDict) -> ASObj {
        ASObj::ASDict(Rc::new(RefCell::new(d)))
    }

    pub fn native_fn(
        name: &str,
        docs: Option<&str>,
        params: Vec<ASFnParam>,
        body: Rc<dyn Fn(&mut Runner) -> Result<Option<ASObj>, ASErreurType>>,
        return_type: ASType,
    ) -> ASObj {
        Self::ASFonc(Rc::new(ASFonc::new(
            Some(name.into()),
            docs.map(|docs| docs.into()),
            params,
            vec![Stmt::native_fn(Rc::clone(&body))],
            return_type,
            ASEnv::new(),
        )))
    }

    pub fn get_type(&self) -> ASType {
        use ASObj as A;

        match self {
            A::ASEntier(..) => w_lit!(ASType::Entier, self),
            A::ASDecimal(..) => w_lit!(ASType::Decimal, self),
            A::ASTexte(..) => w_lit!(ASType::Texte, self),
            A::ASNul => w_lit!(ASType::Nul, self),
            A::ASBooleen(..) => w_lit!(ASType::Booleen, self),
            A::ASListe(ls) => ASType::Array(ls.borrow().iter().map(|e| e.get_type()).collect()),
            A::ASFonc { .. } => ASType::Fonction,
            A::ASMethode(..) => ASType::Fonction,
            A::ASDict(..) => ASType::Dict,
            A::ASClasse(..) => ASType::Classe,
            A::ASClasseInst(inst) => ASType::Objet(inst.classe_parent().name().clone()),
            A::ASTypeObj(..) => ASType::Type,
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
            ASListe(l) => !l.borrow().is_empty(),
            ASDict(d) => !d.borrow().is_empty(),
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

    pub fn pow(&self, rhs: Self) -> ASObj {
        use ASObj::*;

        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => ASEntier(x.pow(y as u32)),
            (ASDecimal(x), ASEntier(y)) => ASDecimal(x.powi(y as i32)),
            (ASEntier(x), ASDecimal(y)) => ASDecimal((*x as f64).powf(y)),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal(x.powf(y)),
            _ => unimplemented!(),
        }
    }

    pub fn extend(&self, rhs: Self) -> Result<ASObj, ASErreurType> {
        use ASObj::*;

        match (self, rhs) {
            (ASTexte(s), rhs @ ASTexte(..)) => Ok(self.clone() + rhs),
            (ASListe(l), ASListe(l2)) => {
                let mut l3 = l.borrow().clone();
                l3.extend(l2.borrow().to_owned());
                Ok(ASObj::liste(l3))
            }

            (ASDict(d), ASDict(d2)) => {
                let mut d3 = d.borrow().clone();
                for e in d2.borrow().items() {
                    d3.insert(e.key().to_owned(), e.val().to_owned());
                }
                Ok(ASObj::dict(d3))
            }

            (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            (ASClasse(classe), _) => todo!("Check présense du field?"),
            (ASModule { name, alias, env }, _) => todo!(),
            (_, rhs) => Err(ASErreurType::new_erreur_operation(
                "++".into(),
                self.get_type(),
                rhs.get_type(),
            )),
        }
    }

    pub fn contains(&self, rhs: &Self) -> Result<bool, ASErreurType> {
        use ASObj::*;

        match (self, rhs) {
            (ASTexte(s), ASTexte(sub_s)) => Ok(s.contains(sub_s)),
            (ASListe(l), rhs) => Ok(l.borrow().contains(rhs)),
            (ASDict(d), rhs) => Ok(d.borrow().contains(rhs)),

            (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            (ASClasse(classe), _) => todo!("Check présense du field?"),
            (ASModule { name, alias, env }, _) => todo!(),
            _ => Err(ASErreurType::new_erreur_operation(
                "dans".into(),
                self.get_type(),
                rhs.get_type(),
            )),
        }
    }

    pub fn get_prop(&self, prop: &String) -> Result<ASObj, ASErreurType> {
        let result = match self {
            ASObj::ASModule { name, alias, env } => {
                let env_borrow = env.borrow();
                let obj = env_borrow.get(prop);
                match obj {
                    Some(obj) => obj.1.clone(),
                    None => {
                        return Err(ASErreurType::new_erreur_access_propriete(
                            self.clone(),
                            prop.clone(),
                        ))
                    }
                }
            }
            ASObj::ASClasse(classe) => {
                let env_borrow = classe.static_env().borrow();
                let Some(value) = env_borrow.get_value(prop) else {
                    return Err(ASErreurType::new_erreur_access_propriete(
                        self.clone(),
                        prop.clone(),
                    ));
                };
                value.clone()
            }
            ASObj::ASClasseInst(inst) => {
                let env_borrow = inst.env().borrow();
                let Some(value) = env_borrow.get_value(prop) else {
                    return Err(ASErreurType::new_erreur_access_propriete(
                        self.clone(),
                        prop.clone(),
                    ));
                };
                value.clone()
            }
            ASObj::ASDict(d) => {
                let d = d.borrow();
                let Some(value) = d.get_val(&ASObj::ASTexte(prop.clone())) else {
                    return Err(ASErreurType::new_erreur_access_propriete(
                        self.clone(),
                        prop.clone(),
                    ));
                };
                value.clone()
            }
            ASObj::ASErreur(err) => {
                if prop == "nom" {
                    ASObj::texte(err.err_type().error_name().to_string())
                } else {
                    return Err(ASErreurType::new_erreur_access_propriete(
                        self.clone(),
                        prop.clone(),
                    ));
                }
            }
            obj => {
                return Err(ASErreurType::new_erreur_access_propriete(
                    obj.clone(),
                    prop.clone(),
                ));
            }
        };

        Ok(result)
    }

    pub fn repr(&self) -> String {
        self.recursive_repr(None)
    }
}

impl RecursiveRepr for ASObj {
    /// Repr récursif, utilisé pour les listes, les dicts, etc.
    fn recursive_repr(
        &self,
        seen_map: Option<Rc<RefCell<HashMap<ObjPtr, (Label, Seen)>>>>,
    ) -> String {
        use ASObj::*;

        let seen_map = seen_map.unwrap_or_else(|| Rc::new(RefCell::new(HashMap::new())));

        match self {
            ASTexte(s) => format!("\"{}\"", s),
            ASListe(l) => {
                let hash = l.as_ptr() as usize;
                let maybe_label = {
                    let seen_map_borrow = seen_map.borrow();
                    seen_map_borrow.get(&hash).map(|(label, seen)| *label)
                };
                if let Some(label) = maybe_label {
                    seen_map.borrow_mut().insert(hash, (label, true));
                    return format!("[<{}>]", label);
                }

                let label = seen_map.borrow().len() + 1;

                {
                    let mut seen_t = seen_map.borrow_mut();
                    seen_t.insert(hash, (label, false));
                }

                let res = l
                    .borrow()
                    .iter()
                    .map(|el| el.recursive_repr(Some(Rc::clone(&seen_map))))
                    .collect::<Vec<_>>();

                let seen = seen_map.borrow()[&hash].1;

                format!(
                    "{}[{}]",
                    if seen {
                        format!("<{}>@", label)
                    } else {
                        "".into()
                    },
                    res.join(", ")
                )
            }
            ASDict(d) => d.borrow().recursive_repr(Some(Rc::clone(&seen_map))),
            // ASPaire { key, val } => format!(
            //     "{}: {}",
            //     key.recursive_repr(Some(Rc::clone(&seen_map))),
            //     val.recursive_repr(Some(Rc::clone(&seen_map)))
            // ),
            ASClasseInst(inst) => inst.recursive_repr(Some(Rc::clone(&seen_map))),
            o => o.to_string(),
        }
    }
}

impl Clone for ASObj {
    fn clone(&self) -> Self {
        use ASObj as A;

        match self {
            A::ASEntier(i) => A::ASEntier(*i),
            A::ASDecimal(d) => A::ASDecimal(*d),
            A::ASBooleen(b) => A::ASBooleen(*b),
            A::ASNul => A::ASNul,
            A::ASNoValue => A::ASNoValue,
            A::ASTexte(t) => A::ASTexte(t.clone()),
            A::ASListe(l) => A::ASListe(Rc::clone(&l)),
            A::ASDict(d) => A::ASDict(Rc::clone(&d)),
            A::ASFonc(fonc) => A::ASFonc(fonc.clone()),
            A::ASClasse(classe) => A::ASClasse(Rc::clone(classe)),
            A::ASModule { name, alias, env } => A::ASModule {
                name: name.clone(),
                alias: alias.clone(),
                env: Rc::clone(env),
            },

            A::ASClasseInst(inst) => A::ASClasseInst(Rc::clone(inst)),
            A::ASMethode(methode) => A::ASMethode(methode.clone()),
            A::ASTuple(_) => todo!(),
            A::ASTypeObj(t) => A::ASTypeObj(t.clone()),
            A::ASErreur(e) => A::ASErreur(e.clone()),
        }
    }
}

impl PartialEq for ASObj {
    fn eq(&self, other: &Self) -> bool {
        use ASObj::*;

        match (self, other) {
            (ASEntier(i), ASDecimal(d)) | (ASDecimal(d), ASEntier(i)) => *d == *i as f64,
            (ASEntier(i1), ASEntier(i2)) => i1 == i2,
            (ASTexte(t1), ASTexte(t2)) => t1 == t2,
            (ASBooleen(b1), ASBooleen(b2)) => b1 == b2,
            (ASListe(l1), ASListe(l2)) => {
                l1.borrow().as_ref() as &Vec<ASObj> == l2.borrow().as_ref() as &Vec<ASObj>
            }
            (ASDict(d1), ASDict(d2)) => d1 == d2,
            (ASFonc(f1), ASFonc(f2)) => f1 == f2,
            (ASClasse(classe1), ASClasse(classe2)) => classe1 == classe2,
            (ASClasseInst(inst1), ASClasseInst(inst2)) => inst1 == inst2,
            (ASNul, ASNul) => true,
            (ASNoValue, ASNoValue) => true,
            _ => false,
        }
    }
}

impl Add for ASObj {
    type Output = ASObj;

    fn add(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        match (self, rhs) {
            (ASListe(l), any) => ASListe({
                let mut l = l.borrow().clone();
                l.push(any);
                Rc::new(RefCell::new(l))
            }),
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
            (ASListe(l), ASEntier(n)) => ASListe(if n <= 0 {
                Rc::new(RefCell::new(vec![]))
            } else {
                let n = n as usize;
                let l = l.borrow();
                let len = l.len();
                let mut new_vec = Vec::with_capacity(n * len);
                for i in 0..n * len {
                    new_vec.push(l[i % len].clone());
                }
                Rc::new(RefCell::new(new_vec))
            }),
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
            (ASEntier(x), ASEntier(y)) => ASEntier((x % y + y) % y),
            (ASDecimal(x), ASEntier(y)) => ASDecimal((x % y as f64 + y as f64) % y as f64),
            (ASEntier(x), ASDecimal(y)) => ASDecimal((x as f64 % y + y) % y),
            (ASDecimal(x), ASDecimal(y)) => ASDecimal((x % y + y) % y),
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

impl BitAnd for ASObj {
    type Output = Result<ASObj, ASErreurType>;

    fn bitand(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x & y)),
            (ASBooleen(x), ASBooleen(y)) => Ok(ASBooleen(x & y)),
            _ => Err(ASErreurType::new_erreur_operation(
                "&".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl BitOr for ASObj {
    type Output = Result<ASObj, ASErreurType>;

    fn bitor(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x | y)),
            (ASBooleen(x), ASBooleen(y)) => Ok(ASBooleen(x | y)),
            _ => Err(ASErreurType::new_erreur_operation(
                "|".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl Shl for ASObj {
    type Output = Result<ASObj, ASErreurType>;

    fn shl(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x << y)),
            _ => Err(ASErreurType::new_erreur_operation(
                "<<".into(),
                type_1,
                type_2,
            )),
        }
    }
}

impl Shr for ASObj {
    type Output = Result<ASObj, ASErreurType>;

    fn shr(self, rhs: Self) -> Self::Output {
        use ASObj::*;

        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();
        match (self, rhs) {
            (ASEntier(x), ASEntier(y)) => Ok(ASEntier(x >> y)),
            _ => Err(ASErreurType::new_erreur_operation(
                ">>".into(),
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
            _ => None?,
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
            ASListe(_) | ASDict(_) | ASClasseInst(_) => self.repr(),
            ASClasse(classe) => format!("classe {}", classe.name()),
            ASFonc(fonc) => fonc.to_string(),
            ASModule {
                name, alias, env, ..
            } => format!(
                "module {}{} {{{}}}",
                name,
                if let Some(alias) = alias {
                    format!(" alias {}", alias)
                } else {
                    "".into()
                },
                env.borrow().0.keys().cloned().collect::<Vec<_>>()[..].join(", ")
            ),
            ASNoValue => String::from("<pas-de-valeur>"),
            ASTypeObj(t) => t.to_string(),
            ASErreur(e) => e.err_type().to_string(),
            _ => String::from("ASObj sans to_string"),
        };
        write!(f, "{}", to_string)
    }
}
