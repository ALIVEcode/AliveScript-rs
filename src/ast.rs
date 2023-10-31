use crate::{
    as_obj::{ASObj, ASType, ASVar, ASEnv},
    visitor::{Visitable, Visitor},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// Expression seule
    Expr(Box<Expr>),

    Utiliser {
        module: String,
        alias: Option<String>,     // None signifie utiliser le nom du module
        vars: Option<Vec<String>>, // None signifie tout utiliser
    },

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

    OpAssign {
        var: Box<Expr>,
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

    Retourner(Option<Box<Expr>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnParam {
    pub name: String,
    pub static_type: Option<ASType>,
    pub default_value: Option<Box<Expr>>,
}

impl FnParam {
    pub fn to_asvar(&self) -> ASVar {
        ASVar::new(self.name.clone(), self.static_type.clone(), false)
    }
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

    CallRust(fn(&mut ASEnv) -> ASObj),
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
            BinComp { .. } => visitor.visit_expr_bincomp(self),
            Lit(..) => visitor.visit_expr_lit(self),
            Ident(..) => visitor.visit_expr_ident(self),
            FnCall { .. } => visitor.visit_expr_fncall(self),
            Range { .. } => visitor.visit_expr_range(self),
            CallRust(..) => visitor.visit_expr_callrust(self),
            _ => todo!(),
        }
    }
}

impl Visitable for Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Stmt::*;

        match self {
            Afficher(..) => visitor.visit_stmt_afficher(self),
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
            Retourner(..) => visitor.visit_stmt_retourner(self),
            Sortir => visitor.visit_stmt_sortir(self),
            Continuer => visitor.visit_stmt_continuer(self),
            node => todo!("{:?}", node),
        }
    }
}
