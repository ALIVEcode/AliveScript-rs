use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::{
    as_obj::{ASFnParam, ASObj, ASScope, ASType, ASVar},
    ast::Expr,
    data::Data,
};

pub static BUILTIN_MOD: Lazy<Arc<ASScope>> = Lazy::new(|| {
    Arc::new(ASScope::from(vec![
        ASVar::new_with_value(
            "typeDe",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "typeDe",
                None,
                vec![ASFnParam {
                    name: "obj".into(),
                    static_type: ASType::any(),
                    default_value: None,
                }],
                |runner| {
                    let env = runner.get_env();
                    let obj = env.get_value(&"obj".into()).unwrap();
                    Ok(Some(ASObj::ASTexte(obj.get_type().to_string())))
                },
                ASType::Texte,
            ),
        ),
        ASVar::new_with_value(
            "typeVar",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "typeVar",
                None,
                vec![ASFnParam {
                    name: "nomVar".into(),
                    static_type: ASType::Texte,
                    default_value: None,
                }],
                |runner| {
                    let env = runner.get_env();
                    let ASObj::ASTexte(nom_var) = env.get_value(&"nomVar".into()).unwrap() else {
                        unreachable!()
                    };
                    let maybe_var = env.get_var(nom_var).map(|v| &v.0);
                    Ok(Some(match maybe_var {
                        Some(var) => ASObj::ASTexte(var.get_type().to_string()),
                        None => ASObj::ASNul,
                    }))
                },
                ASType::union_of(ASType::Texte, ASType::Nul),
            ),
        ),
        ASVar::new_with_value(
            "afficher",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "afficher",
                None,
                vec![ASFnParam {
                    name: "obj".into(),
                    static_type: ASType::any(),
                    default_value: Some(Expr::literal(ASObj::ASTexte("\n".into()))),
                }],
                |runner| {
                    let obj = {
                        let env = runner.get_env();
                        env.get_value(&"obj".into()).unwrap().to_string()
                    };
                    runner.send_data(Data::Afficher(obj));
                    Ok(None)
                },
                ASType::Rien,
            ),
        ),
        ASVar::new_with_value(
            "booleen",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "booleen",
                None,
                vec![ASFnParam {
                    name: "obj".into(),
                    static_type: ASType::any(),
                    default_value: Some(Expr::literal(ASObj::ASBooleen(true))),
                }],
                |runner| {
                    let env = runner.get_env();
                    let obj = env.get_value(&"obj".into()).unwrap();
                    Ok(Some(ASObj::ASBooleen(obj.to_bool())))
                },
                ASType::Booleen,
            ),
        ),
        ASVar::new_with_value(
            "entier",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "entier",
                None,
                vec![
                    ASFnParam::native(
                        "obj",
                        ASType::union_of(ASType::Decimal, ASType::Texte),
                        Some(ASObj::ASEntier(0)),
                    ),
                    ASFnParam::native("base", ASType::Entier, Some(ASObj::ASEntier(10))),
                ],
                |runner| {
                    let env = runner.get_env();
                    let obj = env.get_value(&"obj".into()).unwrap();
                    let ASObj::ASEntier(base) = env.get_value(&"base".into()).unwrap() else {
                        unreachable!()
                    };
                    Ok(Some(match obj {
                        ASObj::ASEntier(_) => obj.clone(),
                        ASObj::ASDecimal(d) => ASObj::ASEntier(*d as i64),
                        ASObj::ASTexte(s) => {
                            ASObj::ASEntier(i64::from_str_radix(s, *base as u32).unwrap())
                        }
                        _ => unreachable!(),
                    }))
                },
                ASType::Entier,
            ),
        ),
        ASVar::new_with_value(
            "decimal",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "decimal",
                None,
                vec![ASFnParam::native(
                    "obj",
                    ASType::union_of(ASType::Decimal, ASType::Texte),
                    Some(ASObj::ASDecimal(0f64)),
                )],
                |runner| {
                    let env = runner.get_env();
                    let obj = env.get_value(&"obj".into()).unwrap();
                    Ok(Some(match obj {
                        ASObj::ASEntier(i) => ASObj::ASDecimal(*i as f64),
                        ASObj::ASDecimal(_) => obj.clone(),
                        ASObj::ASTexte(s) => ASObj::ASDecimal(s.parse().unwrap()),
                        _ => unreachable!(),
                    }))
                },
                ASType::Decimal,
            ),
        ),
    ]))
});
