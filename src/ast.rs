use crate::{
    as_obj::{ASObj, ASType},
    visitor::{Visitable, Visitor},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// Expression seule
    Expr(Box<Expr>),

    /// Afficher
    Afficher(Box<Expr>),

    /// Déclaration
    Decl {
        var: DeclVar,
        val: Box<Expr>,
    },

    /// Affectation
    Assign {
        var: Box<Expr>,
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
    DefFn {
        name: String,
        params: Vec<FnParam>,
        body: Vec<Box<Stmt>>,
        return_type: Option<ASType>,
    },

    DefStruct {
        name: String,
        fields: Vec<StructField>,
    },

    Retourner(Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnParam {
    pub name: String,
    pub static_type: Option<ASType>,
    pub default_value: Option<Box<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub name: String,
    pub static_type: Option<ASType>,
    pub default_value: Option<Box<Expr>>,
    pub is_const: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DeclVar {
    Var {
        name: String,
        static_type: Option<ASType>,
        is_const: bool,
    },
    ListUnpack(Vec<DeclVar>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Lit(ASObj),

    List(Vec<Box<Expr>>),

    Dict(Vec<(Box<Expr>, Box<Expr>)>),

    Ident(String),

    FnCall {
        func: Box<Expr>,
        args: Vec<Box<Expr>>,
    },

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
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinCompcode {
    Eq,
    NotEq,
    Lth,
    Gth,
    Geq,
    Leq,
}

// Visitors
impl Visitable for Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Expr::*;

        match self {
            BinOp { .. } => visitor.visit_expr_binop(self),
            Lit(..) => visitor.visit_expr_lit(self),
            Ident(..) => visitor.visit_expr_ident(self),
            _ => todo!(),
        }
    }
}

impl Visitable for Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Stmt::*;

        match self {
            Afficher(..) => visitor.visit_stmt_afficher(self),
            Decl { .. } => visitor.visit_stmt_decl(self),
            Assign { .. } => visitor.visit_stmt_assign(self),
            _ => todo!(),
        }
    }
}
