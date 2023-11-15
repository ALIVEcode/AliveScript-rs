use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::{
    as_obj::{ASFnParam, ASObj, ASScope, ASType, ASVar},
    ast::Expr, fonction_as, cast_as,
};

pub static TEXTE_MOD: Lazy<Arc<ASScope>> = Lazy::new(|| {
    Arc::new(ASScope::from(vec![
        fonction_as! {
            maj(txt: ASType::Texte) -> ASType::Texte; {
                cast_as!(ASObj::ASTexte(txt) = txt);
                Ok(Some(ASObj::ASTexte(txt.to_uppercase())))
            }
        },
        fonction_as! {
            minus(txt: ASType::Texte) -> ASType::Texte; {
                cast_as!(ASObj::ASTexte(txt) = txt);
                Ok(Some(ASObj::ASTexte(txt.to_lowercase())))
            }
        },
        ASVar::new_with_value(
            "indexDe",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "indexDe",
                None,
                vec![
                    ASFnParam {
                        name: "txt".into(),
                        static_type: ASType::Texte,
                        default_value: None,
                    },
                    ASFnParam {
                        name: "subtxt".into(),
                        static_type: ASType::Texte,
                        default_value: None,
                    },
                ],
                |runner| {
                    let env = runner.get_env();
                    let ASObj::ASTexte(txt) = env.get_value(&"txt".into()).unwrap() else {
                        unreachable!()
                    };
                    let ASObj::ASTexte(subtxt) = env.get_value(&"subtxt".into()).unwrap() else {
                        unreachable!()
                    };
                    let maybe_i = txt.find(subtxt);
                    Ok(Some(match maybe_i {
                        Some(i) => ASObj::ASEntier(i as i64),
                        None => ASObj::ASNul,
                    }))
                },
                ASType::Union(vec![ASType::Entier, ASType::Nul]),
            ),
        ),
        ASVar::new_with_value(
            "remplacer",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "remplacer",
                None,
                vec![
                    ASFnParam {
                        name: "txt".into(),
                        static_type: ASType::Texte,
                        default_value: None,
                    },
                    ASFnParam {
                        name: "pattern".into(),
                        static_type: ASType::Texte,
                        default_value: None,
                    },
                    ASFnParam {
                        name: "remplacement".into(),
                        static_type: ASType::Texte,
                        default_value: None,
                    },
                    ASFnParam {
                        name: "n".into(),
                        static_type: ASType::Entier,
                        default_value: Some(Expr::literal(ASObj::ASNul)),
                    },
                ],
                |runner| {
                    let env = runner.get_env();
                    let ASObj::ASTexte(txt) = env.get_value(&"txt".into()).unwrap() else {
                        unreachable!()
                    };
                    let ASObj::ASTexte(pattern) = env.get_value(&"pattern".into()).unwrap() else {
                        unreachable!()
                    };
                    let ASObj::ASTexte(remplacement) =
                        env.get_value(&"remplacement".into()).unwrap()
                    else {
                        unreachable!()
                    };
                    let i = env.get_value(&"n".into()).unwrap();
                    Ok(Some(match i {
                        ASObj::ASNul => ASObj::ASTexte(txt.replace(pattern, remplacement)),
                        ASObj::ASEntier(n) => {
                            ASObj::ASTexte(txt.replacen(pattern, remplacement, *n as usize))
                        }
                        _ => unreachable!(),
                    }))
                },
                ASType::Texte,
            ),
        ),
    ]))
});
