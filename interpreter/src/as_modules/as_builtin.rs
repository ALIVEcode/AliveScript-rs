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
                    static_type: ASType::Tout,
                    default_value: None,
                }],
                |runner| {
                    let env = runner.get_env();
                    let o = env.get_value(&"o".into()).unwrap();
                    Some(ASObj::ASTexte(o.get_type().to_string()))
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
                    let ASObj::ASTexte(nom_var) = env.get_value(&"nomVar".into()).unwrap() else { unreachable!() };
                    let maybe_var = env.get_var(nom_var).map(|v| &v.0);
                    Some(match maybe_var {
                        Some(var) => ASObj::ASTexte(var.get_type().to_string()),
                        None => ASObj::ASNul,
                    })
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
                    static_type: ASType::Tout,
                    default_value: Some(Expr::literal(ASObj::ASTexte("\n".into()))),
                }],
                |runner| {
                    let obj = {
                        let env = runner.get_env();
                        env.get_value(&"obj".into()).unwrap().to_string()
                    };
                    runner.send_data(Data::Afficher(obj));
                    None
                },
                ASType::Rien,
            ),
        ),
    ]))
});
