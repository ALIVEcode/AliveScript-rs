use std::rc::Rc;

use crate::{
    as_obj::{ASErreurType, ASObj},
    ast::{DefFn, Stmt},
    runner::Runner,
    visitor::{Visitable, Visitor},
};

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
    Positive,
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
    Extend,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
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

// Visitors
impl Visitable for Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Expr as E;

        match self {
            E::BinOp { .. } => visitor.visit_expr_binop(self),
            E::BinComp { .. } => visitor.visit_expr_bincomp(self),
            E::BinLogic { .. } => visitor.visit_expr_binlogic(self),
            E::UnaryOp { .. } => visitor.visit_expr_unaryop(self),
            E::Ternary { .. } => visitor.visit_expr_ternary(self),
            E::Lit(..) => visitor.visit_expr_lit(self),
            E::List(..) => visitor.visit_expr_list(self),
            E::Dict(..) => visitor.visit_expr_dict(self),
            E::Ident(..) => visitor.visit_expr_ident(self),
            E::AccessProp { .. } => visitor.visit_expr_accessprop(self),
            E::FnCall { .. } => visitor.visit_expr_fncall(self),
            E::Range { .. } => visitor.visit_expr_suite(self),
            E::Slice { .. } => visitor.visit_expr_slice(self),
            E::CallRust(..) => visitor.visit_expr_callrust(self),
            E::DefFn { .. } => visitor.visit_expr_deffn(self),
            E::Faire(..) => visitor.visit_expr_faire(self),
        }
    }
}
