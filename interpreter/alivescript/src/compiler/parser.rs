use crate::{
    Rule,
    compiler::bytecode::{BinCompcode, BinLogiccode, BinOpcode, UnaryOpcode},
};
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, PrattParser},
};

lazy_static::lazy_static! {
    pub static ref PRATT_EXPR_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Ternary
            .op(Op::postfix(Rule::Ternary))
            // Logic op
            .op(Op::infix(Rule::Or, Left))
            .op(Op::infix(Rule::And, Left))
            .op(Op::prefix(Rule::Not))

            // Comparaison op
            .op(Op::infix(Rule::In, Left) |
                Op::infix(Rule::NotIn, Left) |
                Op::infix(Rule::Eq, Left) |
                Op::infix(Rule::Neq, Left) |
                Op::infix(Rule::Gt, Left) |
                Op::infix(Rule::Lt, Left) |
                Op::infix(Rule::Gte, Left) |
                Op::infix(Rule::Lte, Left))

            // Bitwise op
            .op(Op::infix(Rule::Xor, Left))
            .op(Op::infix(Rule::Pipe, Left))
            .op(Op::infix(Rule::Ampersant, Left))
            // Range op
            .op(Op::infix(Rule::Range, Left) | Op::infix(Rule::RangeEq, Left))
            // Bitwise op
            .op(Op::infix(Rule::BitwiseLeft, Left) | Op::infix(Rule::BitwiseRight, Left))

            // Arithmetic op
            .op(Op::infix(Rule::Add, Left) | Op::infix(Rule::Sub, Left))
            .op(Op::infix(Rule::Mul, Left) |
                Op::infix(Rule::Div, Left) |
                Op::infix(Rule::DivInt, Left) |
                Op::infix(Rule::Modulo, Left))
            .op(Op::prefix(Rule::Neg) | Op::prefix(Rule::Pos))
            .op(Op::infix(Rule::Pow, Right))

            .op(Op::postfix(Rule::AccessProp))
    };
}

lazy_static::lazy_static! {
    pub static ref PRATT_TYPE_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Op};

        // Precedence is defined lowest to highest
        PrattParser::new().op(Op::postfix(Rule::TypeArgs)).op(Op::infix(Rule::Pipe, Assoc::Left))
    };
}

impl TryFrom<&Pair<'_, Rule>> for BinOpcode {
    type Error = ();

    fn try_from(pair: &Pair<Rule>) -> Result<Self, Self::Error> {
        use BinOpcode as B;
        Ok(match pair.as_rule() {
            Rule::Add => B::Add,
            Rule::Sub => B::Sub,
            Rule::Mul => B::Mul,
            Rule::Div => B::Div,
            Rule::DivInt => B::DivInt,
            Rule::Pow => B::Exp,
            Rule::Pipe => B::BitwiseOr,
            Rule::Ampersant => B::BitwiseAnd,
            Rule::Modulo => B::Mod,
            _ => Err(())?,
        })
    }
}

impl TryFrom<&Pair<'_, Rule>> for BinCompcode {
    type Error = ();

    fn try_from(pair: &Pair<Rule>) -> Result<Self, Self::Error> {
        use BinCompcode as B;
        Ok(match pair.as_rule() {
            Rule::Eq => B::Eq,
            Rule::Neq => B::NotEq,
            Rule::Lt => B::Lth,
            Rule::Gt => B::Gth,
            Rule::Lte => B::Leq,
            Rule::Gte => B::Geq,
            Rule::In => B::Dans,
            Rule::NotIn => B::PasDans,
            _ => Err(())?,
        })
    }
}

impl TryFrom<&Pair<'_, Rule>> for UnaryOpcode {
    type Error = ();

    fn try_from(pair: &Pair<Rule>) -> Result<Self, Self::Error> {
        use UnaryOpcode as U;
        Ok(match pair.as_rule() {
            Rule::Neg => U::Negate,
            Rule::Not => U::Pas,
            Rule::Pos => U::Positive,
            _ => Err(())?,
        })
    }
}

impl TryFrom<&Pair<'_, Rule>> for BinLogiccode {
    type Error = ();

    fn try_from(pair: &Pair<Rule>) -> Result<Self, Self::Error> {
        use BinLogiccode as B;
        Ok(match pair.as_rule() {
            Rule::And => B::Et,
            Rule::Or => B::Ou,
            Rule::NonNull => B::NonNul,
            _ => Err(())?,
        })
    }
}
