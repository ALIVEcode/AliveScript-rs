use std::rc::Rc;

use derive_getters::Getters;
use derive_new::new;

use crate::{
    as_obj::{ASClasseFieldVis, ASErreurType, ASObj},
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
}

#[derive(Clone, Debug, new, Getters, PartialEq)]
pub struct DefFn {
    docs: Option<String>,
    name: Option<String>,
    params: Vec<FnParam>,
    return_type: Option<Box<Type>>,
    body: Vec<Box<Stmt>>,
}

#[derive(Debug, PartialEq, Clone)]
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
    },
    ListUnpack(Vec<DeclVar>),
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

#[derive(Debug, PartialEq, Clone)]
pub enum LireVar {
    Decl(DeclVar),
    Assign(AssignVar),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Lit(ASObj),

    List(Vec<Box<Expr>>),

    // Paire {
    //     clef: Box<Expr>,
    //     val: Box<Expr>,
    // },
    Dict(Vec<Paire>),

    Ident(String),

    /// Définition d'une fonction
    DefFn(DefFn),

    Faire(Vec<Box<Stmt>>),

    AccessProp {
        obj: Box<Expr>,
        prop: String,
    },

    Slice {
        obj: Box<Expr>,
        slice: Box<Expr>,
    },

    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
        is_incl: bool,
    },

    FnCall {
        func: Box<Expr>,
        args: Vec<Box<Expr>>,
    },

    /* ClasseInst {
        classe: Box<Expr>,
        fields: Vec<Box<Expr>>,
    }, */
    BinOp {
        lhs: Box<Expr>,
        op: BinOpcode,
        rhs: Box<Expr>,
    },

    BinComp {
        lhs: Box<Expr>,
        op: BinCompcode,
        rhs: Box<Expr>,
    },

    BinLogic {
        lhs: Box<Expr>,
        op: BinLogiccode,
        rhs: Box<Expr>,
    },

    UnaryOp {
        expr: Box<Expr>,
        op: UnaryOpcode,
    },

    Ternary {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },

    CallRust(CallRust),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Paire {
    pub clef: Box<Expr>,
    pub val: Box<Expr>,
}

pub struct CallRust(pub Rc<dyn Fn(&mut Runner) -> Result<Option<ASObj>, ASErreurType>>);

impl Clone for CallRust {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
impl PartialEq for CallRust {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl std::fmt::Debug for CallRust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("définition interne")
    }
}

impl Expr {
    pub fn literal(obj: ASObj) -> Box<Expr> {
        Box::new(Expr::Lit(obj))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnaryOpcode {
    Pas,
    Negate,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOpcode {
    Mul,
    Div,
    DivInt,
    Add,
    Sub,
    Exp,
    Mod,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinCompcode {
    Eq,
    NotEq,
    Lth,
    Gth,
    Geq,
    Leq,
    Dans,
    PasDans,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinLogiccode {
    Et,
    Ou,
    NonNul,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Lit(String),

    Opt(Box<Type>),

    Array(Vec<Box<Type>>),

    BinOp {
        lhs: Box<Type>,
        op: TypeBinOpcode,
        rhs: Box<Type>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TypeBinOpcode {
    Union,
    Intersection, // TODO:
}

// Visitors
impl Visitable for Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Expr::*;

        match self {
            BinOp { .. } => visitor.visit_expr_binop(self),
            BinComp { .. } => visitor.visit_expr_bincomp(self),
            BinLogic { .. } => visitor.visit_expr_binlogic(self),
            UnaryOp { .. } => visitor.visit_expr_unaryop(self),
            Ternary { .. } => visitor.visit_expr_ternary(self),
            Lit(..) => visitor.visit_expr_lit(self),
            List(..) => visitor.visit_expr_list(self),
            Dict(..) => visitor.visit_expr_dict(self),
            Ident(..) => visitor.visit_expr_ident(self),
            AccessProp { .. } => visitor.visit_expr_accessprop(self),
            FnCall { .. } => visitor.visit_expr_fncall(self),
            Range { .. } => visitor.visit_expr_suite(self),
            Slice { .. } => visitor.visit_expr_slice(self),
            CallRust(..) => visitor.visit_expr_callrust(self),
            DefFn { .. } => visitor.visit_expr_deffn(self),
            Faire(..) => visitor.visit_expr_faire(self),
        }
    }
}

impl Visitable for Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Stmt::*;

        match self {
            Afficher(..) => visitor.visit_stmt_afficher(self),
            Lire { .. } => visitor.visit_stmt_lire(self),
            Utiliser { .. } => visitor.visit_stmt_utiliser(self),
            Expr(..) => visitor.visit_stmt_expr(self),
            Decl { .. } => visitor.visit_stmt_decl(self),
            Assign { .. } => visitor.visit_stmt_assign(self),
            OpAssign { .. } => visitor.visit_stmt_opassign(self),
            Si { .. } => visitor.visit_stmt_si(self),
            CondStmt { .. } => visitor.visit_stmt_condstmt(self),
            TantQue { .. } => visitor.visit_stmt_tantque(self),
            Pour { .. } => visitor.visit_stmt_pour(self),
            DefFn { .. } => visitor.visit_stmt_deffn(self),
            DefClasse { .. } => visitor.visit_stmt_defclasse(self),
            Retourner(..) => visitor.visit_stmt_retourner(self),
            Sortir => visitor.visit_stmt_sortir(self),
            Continuer => visitor.visit_stmt_continuer(self),
            node => todo!("{:?}", node),
        }
    }
}

impl Visitable for Type {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Type::*;

        match self {
            Lit(..) => visitor.visit_type_lit(self),
            BinOp { .. } => visitor.visit_type_binop(self),
            Array(..) => visitor.visit_type_array(self),
            Opt(..) => visitor.visit_type_opt(self),
        }
    }
}
