use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::{
    as_obj::{ASFnParam, ASObj, ASScope, ASType, ASVar},
    ast::Stmt,
};

pub static BUILTIN_MOD: Lazy<Arc<ASScope>> = Lazy::new(|| {
    Arc::new(ASScope::from(vec![
        ASVar::new_with_value(
            "typeDe",
            Some(ASType::Fonction),
            true,
            ASObj::ASFonc {
                params: vec![ASFnParam {
                    name: "o".into(),
                    static_type: ASType::Tout,
                    default_value: None,
                }],
                body: vec![Stmt::native_fn(|runner| {
                    let env = runner.get_env();
                    let o = env.get_value(&"o".into()).unwrap();
                    ASObj::ASTexte(o.get_type().to_string())
                })],
                return_type: ASType::Texte,
            },
        ),
        ASVar::new_with_value(
            "typeVar",
            Some(ASType::Fonction),
            true,
            ASObj::ASFonc {
                params: vec![ASFnParam {
                    name: "nomVar".into(),
                    static_type: ASType::Texte,
                    default_value: None,
                }],
                body: vec![Stmt::native_fn(|runner| {
                    let env = runner.get_env();
                    let ASObj::ASTexte(nom_var) = env.get_value(&"nomVar".into()).unwrap() else { unreachable!() };
                    let maybe_var = env.get_var(nom_var).map(|v| &v.0);
                    match maybe_var {
                        Some(var) => ASObj::ASTexte(var.get_type().to_string()),
                        None => ASObj::ASNul,
                    }
                })],
                return_type: ASType::union_of(ASType::Texte, ASType::Nul),
            },
        ),
    ]))
});
