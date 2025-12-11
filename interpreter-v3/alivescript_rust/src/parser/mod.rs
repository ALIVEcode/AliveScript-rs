use crate::{
    as_obj::{ASErreur, ASErreurType, ASObj},
    ast::{
        AssignVar, BinCompcode, BinLogiccode, BinOpcode, DeclVar, DefFn, Expr, FnParam, Stmt, Type,
        UnaryOpcode,
    },
    utils::Invert,
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
            // Ternary
            .op(Op::postfix(Rule::Ternary))
            .op(Op::postfix(Rule::AccessProp))

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
    };
}

lazy_static::lazy_static! {
    static ref PRATT_TYPE_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        // Precedence is defined lowest to highest
        PrattParser::new()
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

fn parse_top_expr(primary: Pair<Rule>) -> Result<Box<Expr>, PestError<Rule>> {
    match primary.as_rule() {
        Rule::List => Ok(Box::new(Expr::List(
            primary
                .into_inner()
                .map(|arg| parse_expr(arg.into_inner()))
                .collect::<Result<Vec<_>, _>>()?,
        ))),
        Rule::Expr => parse_expr(primary.into_inner()),
        Rule::ListExpr => Ok(Box::new(Expr::List(
            primary
                .into_inner()
                .map(|expr| parse_expr(expr.into_inner()))
                .collect::<Result<_, _>>()?,
        ))),
        Rule::Ident => Ok(Box::new(Expr::Ident(primary.as_str().to_string()))),
        Rule::Lit => Ok(Expr::literal(parse_lit(
            primary.into_inner().next().unwrap(),
        )?)),
        Rule::FnCall => {
            let mut inner = primary.into_inner();
            Ok(Box::new(Expr::FnCall {
                func: parse_expr(inner.next().unwrap().into_inner())?,
                args: inner
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(|arg| parse_expr(arg.into_inner()))
                    .collect::<Result<Vec<_>, _>>()?,
            }))
        }
        Rule::DebutBloc => Ok(Box::new(Expr::Debut(build_ast_stmts(
            primary.into_inner(),
        )?))),

        Rule::EssayerExpr => Ok(Box::new(Expr::Essayer(parse_expr(primary.into_inner())?))),

        Rule::FnExpr => {
            let inner = primary.into_inner();
            Ok(Box::new(Expr::DefFn(DefFn::new(
                None,
                None,
                parse_fn_params(inner.find_first_tagged("params").unwrap().into_inner())?,
                inner
                    .find_first_tagged("return_type")
                    .map(|te| parse_type(te.into_inner()))
                    .invert()?,
                inner
                    .find_first_tagged("body")
                    .map(|body| match body.as_rule() {
                        Rule::Expr => Ok(vec![Box::new(Stmt::Retourner(vec![parse_expr(
                            body.into_inner(),
                        )?]))]),
                        Rule::StmtBody => build_ast_stmts(body.into_inner()),
                        _ => unreachable!(),
                    })
                    .invert()?
                    .unwrap(),
            ))))
        }
        rule => Err(PestError::new_from_span(
            PestErrorVariant::ParsingError {
                positives: vec![Rule::term],
                negatives: vec![rule],
            },
            primary.as_span(),
        )),
    }
}

fn parse_expr<'a>(
    pairs: impl Iterator<Item = Pair<'a, Rule>>,
) -> Result<Box<Expr>, PestError<Rule>> {
    PRATT_EXPR_PARSER
        .map_primary(parse_top_expr)
        .map_prefix(|prefix, rhs| {
            let rhs = rhs?;

            if let Ok(op) = UnaryOpcode::try_from(&prefix) {
                Ok(Box::new(Expr::UnaryOp { expr: rhs, op }))
            } else {
                Err(PestError::new_from_span(
                    PestErrorVariant::ParsingError {
                        positives: vec![Rule::Not, Rule::Neg, Rule::Pos],
                        negatives: vec![prefix.as_rule()],
                    },
                    prefix.as_span(),
                ))
            }
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
            if let Ok(op) = BinLogiccode::try_from(&infix) {
                return Ok(Box::new(Expr::BinLogic { lhs, op, rhs }));
            }

            match infix.as_rule() {
                rule @ (Rule::Range | Rule::RangeEq) => {
                    let inner = infix.into_inner();
                    let start = lhs;
                    let end = inner
                        .find_first_tagged("end")
                        .map(|end| parse_expr(end.into_inner()))
                        .invert()?;

                    let (end, step) = if end.is_some() {
                        (end.unwrap(), Some(rhs))
                    } else {
                        (rhs, None)
                    };

                    Ok(Box::new(Expr::Range {
                        start,
                        end,
                        step,
                        is_incl: rule == Rule::RangeEq,
                    }))
                }
                _ => Err(PestError::new_from_span(
                    PestErrorVariant::ParsingError {
                        positives: vec![Rule::Not, Rule::Neg, Rule::Pos],
                        negatives: vec![infix.as_rule()],
                    },
                    infix.as_span(),
                )),
            }
        })
        .map_postfix(|lhs, postfix| {
            let lhs = lhs?;

            match postfix.as_rule() {
                Rule::AccessProp => {
                    let mut inner = postfix.into_inner();
                    Ok(Box::new(Expr::AccessProp {
                        obj: lhs,
                        prop: inner.next().unwrap().as_str().to_string(),
                    }))
                }
                Rule::Ternary => {
                    let inner = postfix.into_inner();
                    let then_expr = parse_expr(
                        inner
                            .clone()
                            .find(|p| p.as_rule() == Rule::TernaryThen)
                            .unwrap()
                            .into_inner(),
                    )?; // skip the "?"
                    let else_expr = parse_expr(
                        inner
                            .clone()
                            .find(|p| p.as_rule() == Rule::TernaryElse)
                            .unwrap()
                            .into_inner(),
                    )?; // skip the ":"
                    Ok(Box::new(Expr::Ternary {
                        cond: lhs,
                        then_expr,
                        else_expr,
                    }))
                }
                _ => Err(PestError::new_from_span(
                    PestErrorVariant::ParsingError {
                        positives: vec![Rule::Not, Rule::Neg, Rule::Pos],
                        negatives: vec![postfix.as_rule()],
                    },
                    postfix.as_span(),
                )),
            }
        })
        .parse(pairs)
}

fn parse_fn_params(pairs: Pairs<Rule>) -> Result<Vec<FnParam>, PestError<Rule>> {
    Ok(pairs
        .filter_map(|pair| {
            let span = pair.as_span();
            let inner = pair.into_inner();
            let name = inner.find_first_tagged("p_name");

            let Some(name) = name else {
                return Some(Err(PestError::new_from_span(
                    PestErrorVariant::ParsingError {
                        positives: vec![Rule::Ident],
                        negatives: inner.map(|p| p.as_rule()).collect(),
                    },
                    span,
                )));
            };

            let static_type = inner
                .find_first_tagged("p_type")
                .map(|t| parse_type(t.into_inner()))
                .invert();

            let Ok(static_type) = static_type else {
                return Some(Err(static_type.err().unwrap()));
            };

            let default_value = inner
                .find_first_tagged("p_default")
                .map(|d| parse_expr(d.into_inner()))
                .invert();

            let Ok(default_value) = default_value else {
                return Some(Err(default_value.err().unwrap()));
            };

            Some(Ok(FnParam::new(
                name.as_str().to_string(),
                static_type,
                default_value,
            )))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn parse_assign_vars(
    pairs: Pairs<Rule>,
    is_const: Option<bool>,
    public: Option<bool>,
) -> Result<DeclVar, PestError<Rule>> {
    let mut vars = vec![];
    let mut is_const = is_const.unwrap_or(false);
    let mut public = public.unwrap_or(false);

    for pair in pairs {
        match pair.as_rule() {
            Rule::Const => is_const = true,
            Rule::Var | Rule::Assign => {}
            Rule::Pub => public = true,
            Rule::TypeExpr => {
                let DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                    public,
                } = vars.pop().unwrap()
                else {
                    return Err(PestError::new_from_span(
                        PestErrorVariant::CustomError {
                            message: "Only vars can be typed".into(),
                        },
                        pair.as_span(),
                    ));
                };
                let static_type = Some(parse_type(pair.into_inner())?);
                vars.push(DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                    public,
                });
            }
            Rule::Ident => {
                vars.push(DeclVar::Var {
                    name: pair.as_str().to_string(),
                    static_type: None,
                    is_const,
                    public,
                });
            }
            Rule::MultiDeclIdent => {
                vars.push(parse_assign_vars(
                    pair.into_inner(),
                    Some(is_const),
                    Some(public),
                )?);
            }
            Rule::DeclIdentList => {
                vars.push(parse_assign_vars(
                    pair.into_inner(),
                    Some(is_const),
                    Some(public),
                )?);
            }
            _ => panic!("{:#?}", pair),
        }
    }

    if vars.len() == 1 {
        Ok(vars[0].clone())
    } else {
        Ok(DeclVar::ListUnpack(vars))
    }
}

fn parse_assign(pairs: Pairs<Rule>) -> Result<(DeclVar, Box<Expr>), PestError<Rule>> {
    let mut name = None;
    let mut static_type = None;
    let mut is_const = false;
    let mut public = false;
    let mut expr = None;
    let mut var_list = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::Const => is_const = true,
            Rule::Var | Rule::Assign => {}
            Rule::Pub => public = true,
            Rule::Expr => expr = Some(parse_expr(pair.into_inner())?),
            Rule::TypeExpr => static_type = Some(parse_type(pair.into_inner())?),
            Rule::Ident => name = Some(pair.as_str().to_string()),
            Rule::MultiDeclIdent => {
                var_list = Some(parse_assign_vars(
                    pair.into_inner(),
                    Some(is_const),
                    Some(public),
                )?)
            }
            Rule::DeclIdentList => {
                var_list = Some(parse_assign_vars(
                    pair.into_inner(),
                    Some(is_const),
                    Some(public),
                )?)
            }
            _ => panic!("{:#?}", pair),
        }
    }

    match var_list {
        Some(v) => Ok((v, expr.unwrap())),
        None => Ok((
            DeclVar::Var {
                name: name.unwrap(),
                static_type,
                is_const,
                public,
            },
            expr.unwrap(),
        )),
    }
}

fn parse_lit(pair: Pair<Rule>) -> Result<ASObj, PestError<Rule>> {
    Ok(match pair.as_rule() {
        Rule::Integer => ASObj::ASEntier(pair.as_str().parse::<i64>().unwrap()),
        Rule::Decimal => ASObj::ASDecimal(pair.as_str().parse::<f64>().unwrap()),
        Rule::Bool => ASObj::ASBooleen(pair.as_str() == "vrai"),
        Rule::Null => ASObj::ASNul,
        Rule::Text => {
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
                positives: vec![Rule::Lit],
                negatives: vec![rule],
            },
            pair.as_span(),
        ))?,
    })
}

fn parse_type(pairs: Pairs<Rule>) -> Result<Box<Type>, PestError<Rule>> {
    PRATT_TYPE_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::TypeExpr => parse_type(primary.into_inner()),
            Rule::Ident => Ok(Box::new(Type::Name(primary.as_str().to_string()))),
            Rule::Lit => Ok(Box::new(Type::Lit(parse_lit(
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

fn parse_if(pair: Pair<Rule>) -> Result<Stmt, PestError<Rule>> {
    let mut elif_brs = vec![];
    let mut else_br = None;
    let inner = pair.clone().into_inner();
    let mut curr_br = pair;
    let cond = parse_expr(
        inner
            .clone()
            .find(|p| matches!(p.as_node_tag(), Some("cond")))
            .unwrap()
            .into_inner(),
    )?;
    let then_br = build_ast_stmts(
        inner
            .clone()
            .find(|p| matches!(p.as_node_tag(), Some("body")))
            .unwrap()
            .into_inner(),
    )?;

    loop {
        match curr_br.as_rule() {
            Rule::SiStmt => {
                if let Some(next_br) = curr_br
                    .into_inner()
                    .find(|p| matches!(p.as_rule(), Rule::sinonSiBr | Rule::sinonBr))
                {
                    curr_br = next_br;
                } else {
                    break;
                }
            }
            Rule::sinonSiBr => {
                let mut inner_elif = curr_br.into_inner();
                let cond = parse_expr(
                    inner_elif
                        .find(|p| matches!(p.as_node_tag(), Some("cond")))
                        .unwrap()
                        .into_inner(),
                )?;
                let body = build_ast_stmts(
                    inner_elif
                        .find(|p| matches!(p.as_node_tag(), Some("body")))
                        .unwrap()
                        .into_inner(),
                )?;

                elif_brs.push((cond, body));

                if let Some(next_br) =
                    inner_elif.find(|p| matches!(p.as_rule(), Rule::sinonSiBr | Rule::sinonBr))
                {
                    curr_br = next_br;
                } else {
                    break;
                }
            }
            Rule::sinonBr => {
                let body = curr_br.into_inner().find_first_tagged("body").unwrap();
                if body.as_rule() == Rule::StmtBody {
                    else_br = Some(build_ast_stmts(body.into_inner())?);
                } else {
                    else_br = Some(vec![build_ast_stmt(body)?]);
                }
                break;
            }
            _ => {}
        }
    }

    Ok(Stmt::Si {
        cond,
        then_br,
        elif_brs,
        else_br,
    })
}

pub fn build_ast_stmt(pair: Pair<Rule>) -> Result<Box<Stmt>, PestError<Rule>> {
    Ok(Box::new(match pair.as_rule() {
        Rule::AfficherStmt => Stmt::Afficher(vec![parse_expr(pair.into_inner().skip(1))?]),
        Rule::UtiliserStmt => {
            let inner = pair.into_inner();
            let module_name = inner.clone().next().unwrap();
            let alias = inner
                .clone()
                .find(|node| node.as_rule() == Rule::ModuleAlias)
                .map(|alias| alias.as_str().to_string());
            let vars = inner
                .clone()
                .find(|node| node.as_rule() == Rule::UtiliserMembers)
                .map(|node| {
                    node.into_inner()
                        .find_tagged("member")
                        .map(|node| node.as_str().to_string())
                        .collect::<Vec<String>>()
                });
            Stmt::Utiliser {
                module: module_name.as_str().trim_matches('"').to_string(),
                alias,
                vars,
                is_path: module_name.as_node_tag().is_some_and(|node| node == "path"),
                public: false,
            }
        }
        Rule::DeclStmt => {
            let (var, val) = parse_assign(pair.into_inner())?;
            Stmt::Decl { var, val }
        }
        Rule::AssignStmt => match parse_assign(pair.into_inner())? {
            (
                DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                    public,
                },
                val,
            ) => Stmt::Assign {
                var: AssignVar::Var { name, static_type },
                val,
            },
            (decl @ DeclVar::ListUnpack(..), val) => Stmt::Assign {
                var: AssignVar::from(decl),
                val,
            },
        },
        Rule::CommandStmt => {
            let mut inner = pair.into_inner();
            Stmt::Expr(Box::new(Expr::FnCall {
                func: parse_top_expr(inner.next().unwrap())?,
                args: vec![parse_top_expr(inner.next().unwrap())?],
            }))
        }
        Rule::PubStmt => {
            let mut inner = pair.into_inner();
            let mut result = build_ast_stmt(inner.nth(1).unwrap())?;
            result.mk_public();
            *result
        }
        Rule::FnDef => {
            let mut inner = pair.into_inner();
            Stmt::DefFn(DefFn::new(
                None,
                inner
                    .find_first_tagged("name")
                    .map(|node| node.as_str().to_string()),
                parse_fn_params(inner.find_first_tagged("params").unwrap().into_inner())?,
                inner
                    .find_first_tagged("return_type")
                    .map(|te| parse_type(te.into_inner()))
                    .invert()?,
                inner
                    .find(|node| node.as_rule() == Rule::FnBody)
                    .map(|body| match body.into_inner().next().unwrap() {
                        body if body.as_rule() == Rule::Expr => Ok(vec![Box::new(
                            Stmt::Retourner(vec![parse_expr(body.into_inner())?]),
                        )]),
                        body if body.as_rule() == Rule::StmtBody => {
                            build_ast_stmts(body.into_inner())
                        }
                        _ => unreachable!(),
                    })
                    .invert()?
                    .unwrap(),
            ))
        }
        Rule::SiStmt => {
            parse_if(pair)?
            // let inner = pair.into_inner();
            // Stmt::Si {
            //     cond: parse_expr(
            //         inner
            //             .clone()
            //             .find(|p| matches!(p.as_node_tag(), Some("cond")))
            //             .unwrap()
            //             .into_inner(),
            //     )?,
            //     then_br: build_ast_stmts(
            //         inner
            //             .clone()
            //             .find(|p| matches!(p.as_node_tag(), Some("body")))
            //             .unwrap()
            //             .into_inner(),
            //     )?,
            //     elif_brs: inner
            //         .clone()
            //         .filter_map(|elif| {
            //             if elif.as_rule() != Rule::sinonSiBr {
            //                 return None;
            //             };
            //             let mut inner_elif = elif.into_inner();
            //             let cond = parse_expr(
            //                 inner_elif
            //                     .find(|p| matches!(p.as_node_tag(), Some("cond")))
            //                     .unwrap()
            //                     .into_inner(),
            //             );
            //             let body = build_ast_stmts(inner_elif.last().unwrap().into_inner());
            //             let Ok(cond) = cond else {
            //                 return Some(Err(cond.err().unwrap()));
            //             };
            //             let Ok(body) = body else {
            //                 return Some(Err(body.err().unwrap()));
            //             };
            //             Some(Ok((cond, body)))
            //         })
            //         .collect::<Result<_, _>>()?,
            //     else_br: inner
            //         .clone()
            //         .find(|p| p.as_rule() == Rule::sinonBr)
            //         .map(|br| build_ast_stmts(br.into_inner()))
            //         .invert()?,
            // }
        }
        Rule::PourStmt => {
            let inner = pair.into_inner();
            Stmt::Pour {
                var: parse_assign_vars(
                    inner
                        .clone()
                        .find_first_tagged("vars")
                        .unwrap()
                        .into_inner(),
                    None,
                    Some(false),
                )?,
                iterable: parse_expr(
                    inner
                        .clone()
                        .find_first_tagged("iter")
                        .unwrap()
                        .into_inner(),
                )?,
                body: inner
                    .clone()
                    .find(|p| p.as_rule() == Rule::StmtBody)
                    .map(|body| build_ast_stmts(body.into_inner()))
                    .invert()?
                    .unwrap_or_default(),
            }
        }
        Rule::ContinuerStmt => Stmt::Continuer,
        Rule::SortirStmt => Stmt::Sortir,
        Rule::RetournerStmt => Stmt::Retourner(
            pair.into_inner()
                .skip(1)
                .map(|expr| parse_expr(expr.into_inner()))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Rule::Expr => Stmt::Expr(parse_expr(pair.into_inner())?),
        rule => Err(PestError::new_from_span(
            PestErrorVariant::ParsingError {
                positives: vec![Rule::stmt],
                negatives: vec![rule],
            },
            pair.as_span(),
        ))?,
    }))
}

pub fn build_ast_stmts(pairs: Pairs<Rule>) -> Result<Vec<Box<Stmt>>, PestError<Rule>> {
    let mut stmts = vec![];
    for pair in pairs {
        if matches!(pair.as_rule(), Rule::EOI) {
            continue;
        };
        stmts.push(build_ast_stmt(pair)?);
    }

    Ok(stmts)
}
