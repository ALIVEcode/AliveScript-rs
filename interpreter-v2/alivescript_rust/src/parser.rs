use crate::{
    as_obj::{ASErreur, ASErreurType, ASObj},
    ast::{AssignVar, BinCompcode, BinOpcode, DeclVar, Expr, Stmt, Type},
    Rule,
};
use pest::error::{Error as PestError, ErrorVariant as PestErrorVariant};
use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::PrattParser,
};

lazy_static::lazy_static! {
    static ref PRATT_EXPR_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(Rule::add, Left) | Op::infix(Rule::sub, Left))
            .op(Op::infix(Rule::mul, Left) | Op::infix(Rule::div, Left))
    };
}

lazy_static::lazy_static! {
    static ref PRATT_TYPE_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(Rule::add, Left) | Op::infix(Rule::sub, Left))
            .op(Op::infix(Rule::mul, Left) | Op::infix(Rule::div, Left))
    };
}

impl TryFrom<&Pair<'_, Rule>> for BinOpcode {
    type Error = ();

    fn try_from(pair: &Pair<Rule>) -> Result<Self, Self::Error> {
        use BinOpcode as B;
        Ok(match pair.as_rule() {
            Rule::add => B::Add,
            Rule::mul => B::Mul,
            _ => Err(())?,
        })
    }
}

impl TryFrom<&Pair<'_, Rule>> for BinCompcode {
    type Error = ();

    fn try_from(pair: &Pair<Rule>) -> Result<Self, Self::Error> {
        use BinCompcode as B;
        Ok(match pair.as_rule() {
            Rule::eq => B::Eq,
            _ => Err(())?,
        })
    }
}

fn parse_lit(pair: Pair<Rule>) -> Result<ASObj, PestError<Rule>> {
    Ok(match pair.as_rule() {
        Rule::integer => ASObj::ASEntier(pair.as_str().parse::<i64>().unwrap()),
        Rule::decimal => ASObj::ASDecimal(pair.as_str().parse::<f64>().unwrap()),
        Rule::bool => ASObj::ASBooleen(pair.as_str() == "vrai"),
        Rule::null => ASObj::ASNul,
        Rule::text => {
            let slice = pair.as_str();
            let s: String = slice[1..slice.len() - 1].parse().unwrap();
            ASObj::ASTexte(
                s.replace(r"\n", "\n")
                    .replace(r"\t", "\t")
                    .replace(r"\r", "\r")
                    .to_owned(),
            )
        }
        rule => Err(PestError::new_from_span(
            PestErrorVariant::ParsingError {
                positives: vec![Rule::lit],
                negatives: vec![rule],
            },
            pair.as_span(),
        ))?,
    })
}

fn parse_expr(pairs: Pairs<Rule>) -> Result<Box<Expr>, PestError<Rule>> {
    PRATT_EXPR_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::expr => parse_expr(primary.into_inner()),
            Rule::term => parse_expr(primary.into_inner()),
            Rule::ident => Ok(Box::new(Expr::Ident(primary.as_str().to_string()))),
            Rule::lit => Ok(Expr::literal(parse_lit(
                primary.into_inner().next().unwrap(),
            )?)),
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::term],
                    negatives: vec![rule],
                },
                primary.as_span(),
            )),
        })
        .map_infix(|lhs, infix, rhs| {
            let lhs = lhs?;
            let rhs = rhs?;

            if let Ok(op) = BinOpcode::try_from(&infix) {
                return Ok(Box::new(Expr::BinOp { lhs, op, rhs }));
            }
            if let Ok(op) = BinCompcode::try_from(&infix) {
                return Ok(Box::new(Expr::BinComp { lhs, op, rhs }));
            }

            todo!()
        })
        .parse(pairs)
}

fn parse_type(pairs: Pairs<Rule>) -> Result<Box<Type>, PestError<Rule>> {
    PRATT_TYPE_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::typeTerm => parse_type(primary.into_inner()),
            Rule::typeExpr => parse_type(primary.into_inner()),
            Rule::ident => Ok(Box::new(Type::Name(primary.as_str().to_string()))),
            Rule::lit => Ok(Box::new(Type::Lit(parse_lit(
                primary.into_inner().next().unwrap(),
            )?))),
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::typeTerm],
                    negatives: vec![rule],
                },
                primary.as_span(),
            )),
        })
        .map_infix(|lhs, infix, rhs| todo!())
        .parse(pairs)
}

fn parse_assign(pairs: Pairs<Rule>) -> Result<(DeclVar, Box<Expr>), PestError<Rule>> {
    let mut name = None;
    let mut static_type = None;
    let mut is_const = false;
    let mut expr = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::constKw => is_const = true,
            Rule::varKw | Rule::assign => {}
            Rule::expr => expr = Some(parse_expr(pair.into_inner())?),
            Rule::typeExpr => static_type = Some(parse_type(pair.into_inner())?),
            Rule::ident => name = Some(pair.as_str().to_string()),
            _ => panic!("{:#?}", pair),
        }
    }

    Ok((
        DeclVar::Var {
            name: name.unwrap(),
            static_type,
            is_const,
        },
        expr.unwrap(),
    ))
}

pub fn build_ast_stmts(pairs: Pairs<Rule>) -> Result<Vec<Box<Stmt>>, PestError<Rule>> {
    let mut stmts = vec![];
    for pair in pairs {
        if matches!(pair.as_rule(), Rule::EOI) {
            continue;
        };
        stmts.push(Box::new(match pair.as_rule() {
            Rule::afficherStmt => Stmt::Afficher(vec![parse_expr(pair.into_inner())?]),
            Rule::declStmt => {
                let (var, val) = parse_assign(pair.into_inner())?;
                Stmt::Decl { var, val }
            }
            Rule::assignStmt => {
                let (
                    DeclVar::Var {
                        name,
                        static_type,
                        is_const,
                    },
                    val,
                ) = parse_assign(pair.into_inner())?
                else {
                    unreachable!();
                };
                Stmt::Assign {
                    var: AssignVar::Var { name, static_type },
                    val,
                }
            }
            Rule::expr => Stmt::Expr(parse_expr(pair.into_inner())?),
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::stmt],
                    negatives: vec![rule],
                },
                pair.as_span(),
            ))?,
        }));
    }

    Ok(stmts)
}
