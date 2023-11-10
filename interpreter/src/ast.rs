use crate::{
    as_obj::ASObj,
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
    },

    /// Afficher
    Afficher(Box<Expr>),

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
        return_type: Option<Box<Type>>,
    },

    DefStruct {
        name: String,
        fields: Vec<StructField>,
    },

    Retourner(Option<Box<Expr>>),
}

impl Stmt {
    /// Body en rust d'une fonction
    pub fn native_fn(body: fn(&mut Runner) -> Option<ASObj>) -> Box<Self> {
        Box::new(Stmt::Retourner(Some(Box::new(Expr::CallRust(body)))))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnParam {
    pub name: String,
    pub static_type: Option<Box<Type>>,
    pub default_value: Option<Box<Expr>>,
}

// impl FnParam {
//     pub fn new(
//         name: impl ToString,
//         static_type: Option<Type>,
//         default_value: Option<Box<Expr>>,
//     ) -> Self {
//         Self {
//             name: name.to_string(),
//             static_type: static_type.into(),
//             default_value,
//         }
//     }
//
//     pub fn to_asvar(&self) -> ASVar {
//         ASVar::new(self.name.clone(), Some(self.static_type.clone()), false)
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub name: String,
    pub static_type: Option<Box<Type>>,
    pub default_value: Option<Box<Expr>>,
    pub is_const: bool,
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
    Decl(DeclVar),
    Slice {
        obj: Box<Expr>,
        slice: Box<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum LireVar {
    Decl(DeclVar),
    Assign(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Lit(ASObj),

    List(Vec<Box<Expr>>),

    Paire {
        clef: Box<Expr>,
        val: Box<Expr>,
    },

    Dict(Vec<Box<Expr>>), // Expr est garanti d'être Expr::Paire

    Ident(String),

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

    CallRust(fn(&mut Runner) -> Option<ASObj>),
}

impl Expr {
    pub fn literal(obj: ASObj) -> Box<Expr> {
        Box::new(Expr::Lit(obj))
    }
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

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Lit(String),

    Opt(Box<Type>),

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
            Lit(..) => visitor.visit_expr_lit(self),
            List(..) => visitor.visit_expr_list(self),
            Dict(..) => visitor.visit_expr_dict(self),
            Paire { .. } => visitor.visit_expr_paire(self),
            Ident(..) => visitor.visit_expr_ident(self),
            AccessProp { .. } => visitor.visit_expr_accessprop(self),
            FnCall { .. } => visitor.visit_expr_fncall(self),
            Range { .. } => visitor.visit_expr_range(self),
            Slice { .. } => visitor.visit_expr_slice(self),
            CallRust(..) => visitor.visit_expr_callrust(self),
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
            Opt(..) => visitor.visit_type_opt(self),
        }
    }
}
