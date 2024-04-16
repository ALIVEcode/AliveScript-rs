use crate::visitor::{Visitable, Visitor};

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

impl Visitable for Type {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Type as T;

        match self {
            T::Lit(..) => visitor.visit_type_lit(self),
            T::BinOp { .. } => visitor.visit_type_binop(self),
            T::Array(..) => visitor.visit_type_array(self),
            T::Opt(..) => visitor.visit_type_opt(self),
        }
    }
}
