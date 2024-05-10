use crate::{
    as_obj::ASObj,
    visitor::{Visitable, Visitor},
};

use super::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Name(String),

    Lit(ASObj),

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

impl Visitable for Type {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Type as T;

        match self {
            T::Lit(..) => visitor.visit_type_lit(self),
            T::Name(..) => visitor.visit_type_name(self),
            T::BinOp { .. } => visitor.visit_type_binop(self),
            T::Array(..) => visitor.visit_type_array(self),
            T::Opt(..) => visitor.visit_type_opt(self),
        }
    }
}
