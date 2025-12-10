use std::rc::Rc;

use derive_getters::Getters;
use derive_new::new;

use crate::{
    as_obj::{ASClasseFieldVis, ASErreurType, ASObj},
    ast::{BinOpcode, CallRust, Expr, Type},
    runner::Runner,
    visitor::{Visitable, Visitor},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// Expression seule
    Expr(Box<Expr>),

    Utiliser {
        module: String,
        /// None signifie utiliser le nom du module. "*" signifie pas de nom
        alias: Option<String>,
        // None signifie tout utiliser
        vars: Option<Vec<String>>,
        is_path: bool,
        public: bool,
    },

    /// Afficher
    Afficher(Vec<Box<Expr>>),

    /// Lire
    Lire {
        factory: Option<Box<Expr>>,
        var: LireVar,
        prompt: Option<Box<Expr>>,
    },

    /// Déclaration
    Decl {
        var: DeclVar,
        val: Box<Expr>,
    },

    /// Affectation
    Assign {
        var: AssignVar,
        val: Box<Expr>,
    },

    TypeDecl {
        var: Box<Type>,
        val: Box<Type>,
    },

    OpAssign {
        var: AssignVar,
        op: BinOpcode,
        val: Box<Expr>,
    },

    /// Conditionnel
    Si {
        cond: Box<Expr>,
        then_br: Vec<Box<Stmt>>,
        elif_brs: Vec<(Box<Expr>, Vec<Box<Stmt>>)>,
        else_br: Option<Vec<Box<Stmt>>>,
    },

    /// Stmt Conditionnel
    CondStmt {
        cond: Box<Expr>,
        then_stmt: Box<Stmt>,
    },

    /// Boucle repeter
    Repeter {
        n: Option<Box<Expr>>,
        body: Vec<Box<Stmt>>,
    },

    /// Boucle tant que
    TantQue {
        cond: Box<Expr>,
        body: Vec<Box<Stmt>>,
    },

    /// Boucle pour
    Pour {
        var: DeclVar,         //
        iterable: Box<Expr>,  // itérable
        body: Vec<Box<Stmt>>, //
    },

    Continuer,
    Sortir,

    /// Définition d'une fonction
    DefFn(DefFn),

    DefClasse {
        name: String,
        docs: Option<String>,
        fields: Vec<ClasseField>,
        init: Option<DefFn>,
        methods: Vec<DefMethod>,
    },

    Retourner(Vec<Box<Expr>>),
}

impl Stmt {
    /// Body en rust d'une fonction
    pub fn native_fn(
        body: Rc<dyn Fn(&mut Runner) -> Result<Option<ASObj>, ASErreurType>>,
    ) -> Box<Self> {
        Box::new(Stmt::Retourner(vec![Box::new(Expr::CallRust(CallRust(
            Rc::clone(&body),
        )))]))
    }

    pub fn mk_public(&mut self) {
        match self {
            Self::DefFn(def_fn) => def_fn.public = true,
            Self::Utiliser { public, .. } => {
                *public = true;
            }
            Self::Decl { var, .. } => match var {
                DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                    public,
                } => {
                    *var = DeclVar::Var {
                        name: name.clone(),
                        static_type: static_type.clone(),
                        is_const: *is_const,
                        public: true,
                    }
                }
                DeclVar::ListUnpack(s) => todo!(),
            },
            _ => {}
        }
    }
}

#[derive(Clone, Debug, new, Getters, PartialEq)]
pub struct DefFn {
    pub docs: Option<String>,
    pub name: Option<String>,
    pub params: Vec<FnParam>,
    pub return_type: Option<Box<Type>>,
    pub body: Vec<Box<Stmt>>,
    #[new(value = "false")]
    pub public: bool,
}

#[derive(Debug, PartialEq, Clone, new, Getters)]
pub struct FnParam {
    pub name: String,
    pub static_type: Option<Box<Type>>,
    pub default_value: Option<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone, Getters)]
pub struct ClasseField {
    pub name: String,
    pub vis: ClasseFieldVis,
    pub static_type: Option<Box<Type>>,
    pub default_value: Option<Box<Expr>>,
    pub is_const: bool,
    pub is_static: bool,
}

#[derive(Clone, Debug, new, Getters, PartialEq)]
pub struct DefMethod {
    docs: Option<String>,
    name: Option<String>,
    params: Vec<FnParam>,
    return_type: Option<Box<Type>>,
    body: Vec<Box<Stmt>>,
    is_static: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ClasseFieldVis {
    Publique,
    Privee,
}

impl Into<ASClasseFieldVis> for ClasseFieldVis {
    fn into(self) -> ASClasseFieldVis {
        match self {
            Self::Privee => ASClasseFieldVis::Privee,
            Self::Publique => ASClasseFieldVis::Publique,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DeclVar {
    Var {
        name: String,
        static_type: Option<Box<Type>>,
        is_const: bool,
        public: bool,
    },
    ListUnpack(Vec<DeclVar>),
}

impl DeclVar {
    pub fn mk_public(&mut self) {
        match self {
            DeclVar::Var {
                name,
                static_type,
                is_const,
                public,
            } => {
                *self = DeclVar::Var {
                    name: name.clone(),
                    static_type: static_type.clone(),
                    is_const: *is_const,
                    public: true,
                }
            }
            DeclVar::ListUnpack(vars) => {
                vars.iter_mut().for_each(|var| var.mk_public());
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignVar {
    Var {
        name: String,
        static_type: Option<Box<Type>>,
    },
    ListUnpack(Vec<AssignVar>),
    Slice {
        obj: Box<Expr>,
        slice: Box<Expr>,
    },
    AccessProp {
        obj: Box<Expr>,
        prop: String,
    },
}

impl From<DeclVar> for AssignVar {
    fn from(value: DeclVar) -> Self {
        match value {
            DeclVar::Var {
                name,
                static_type,
                is_const,
                public,
            } => Self::Var { name, static_type },
            DeclVar::ListUnpack(vars) => {
                AssignVar::ListUnpack(vars.into_iter().map(|var| Self::from(var)).collect())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LireVar {
    Decl(DeclVar),
    Assign(AssignVar),
}

impl Visitable for Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Stmt as S;

        match self {
            S::Afficher(..) => visitor.visit_stmt_afficher(self),
            S::Lire { .. } => visitor.visit_stmt_lire(self),
            S::Utiliser { .. } => visitor.visit_stmt_utiliser(self),
            S::Expr(..) => visitor.visit_stmt_expr(self),
            S::Decl { .. } => visitor.visit_stmt_decl(self),
            S::TypeDecl { .. } => visitor.visit_stmt_type(self),
            S::Assign { .. } => visitor.visit_stmt_assign(self),
            S::OpAssign { .. } => visitor.visit_stmt_opassign(self),
            S::Si { .. } => visitor.visit_stmt_si(self),
            S::CondStmt { .. } => visitor.visit_stmt_condstmt(self),
            S::TantQue { .. } => visitor.visit_stmt_tantque(self),
            S::Pour { .. } => visitor.visit_stmt_pour(self),
            S::Repeter { .. } => visitor.visit_stmt_repeter(self),
            S::DefFn { .. } => visitor.visit_stmt_deffn(self),
            S::DefClasse { .. } => visitor.visit_stmt_defclasse(self),
            S::Retourner(..) => visitor.visit_stmt_retourner(self),
            S::Sortir => visitor.visit_stmt_sortir(self),
            S::Continuer => visitor.visit_stmt_continuer(self),
        }
    }
}
