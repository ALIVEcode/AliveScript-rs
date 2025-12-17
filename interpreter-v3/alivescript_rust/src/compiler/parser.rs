use crate::Rule;
use pest::pratt_parser::PrattParser;

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
        PrattParser::new().op(Op::postfix(Rule::TypeArgs))
    };
}
