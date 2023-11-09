use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::{
    as_obj::{ASFnParam, ASObj, ASScope, ASType, ASVar},
    ast::Expr,
    visitor::Visitable,
};

pub static LISTE_MOD: Lazy<Arc<ASScope>> = Lazy::new(|| {
    Arc::new(ASScope::from(vec![ASVar::new_with_value(
        "trier",
        Some(ASType::Fonction),
        true,
        ASObj::native_fn(
            "trier",
            None,
            vec![
                ASFnParam {
                    name: "lst".into(),
                    static_type: ASType::Liste,
                    default_value: None,
                },
                ASFnParam {
                    name: "clef".into(),
                    static_type: ASType::Fonction,
                    default_value: Some(Box::new(Expr::Lit(ASObj::ASNul))),
                },
            ],
            |runner| {
                let env = runner.get_env();
                let ASObj::ASListe(lst) = env.get_value(&"lst".into()).unwrap() else { unreachable!() };
                let mut lst = lst.clone();
                Some(match env.get_value(&"clef".into()).unwrap() {
                    ASObj::ASNul => {
                        lst.sort_by(|a, b| a.partial_cmp(b).expect("Comparable"));
                        ASObj::ASListe(lst)
                    }
                    clef @ ASObj::ASFonc { .. } => {
                        let clef = Expr::literal(clef.clone());
                        for el in lst.iter_mut() {
                            let to_call = Expr::FnCall {
                                func: clef.clone(),
                                args: vec![Expr::literal(el.clone())],
                            };
                            to_call.accept(runner);
                            *el = runner.pop_value().unwrap();
                        }
                        lst.sort_by(|a, b| a.partial_cmp(b).expect("Comparable"));
                        ASObj::ASListe(lst)
                    }
                    _ => unreachable!(),
                })
            },
            ASType::Liste,
        ),
    )]))
});
