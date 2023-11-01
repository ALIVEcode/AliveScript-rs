use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::{
    as_obj::{ASObj, ASScope, ASType, ASVar},
    ast::{FnParam, Stmt},
};

pub static MATH_MOD: Lazy<Arc<ASScope>> = Lazy::new(|| {
    Arc::new(ASScope::from(vec![
        ASVar::new_with_value(
            "sin",
            Some(ASType::Fonction),
            true,
            ASObj::ASFonc {
                params: vec![FnParam {
                    name: "x".into(),
                    static_type: ASType::nombre(),
                    default_value: None,
                }],
                body: vec![Stmt::native_fn(|env| {
                    match env.get_var(&"x".into()).unwrap() {
                        (_, ASObj::ASDecimal(n)) => ASObj::ASDecimal(n.sin()),
                        (_, ASObj::ASEntier(i)) => ASObj::ASDecimal((*i as f64).sin()),
                        _ => unreachable!(),
                    }
                })],
                return_type: ASType::Decimal,
            },
        ),
        ASVar::new_with_value(
            "cos",
            Some(ASType::Fonction),
            true,
            ASObj::ASFonc {
                params: vec![FnParam::new("x", None, None)],
                body: vec![Stmt::native_fn(|env| {
                    match env.get_var(&"x".into()).unwrap() {
                        (_, ASObj::ASDecimal(n)) => ASObj::ASDecimal(n.cos()),
                        (_, ASObj::ASEntier(i)) => ASObj::ASDecimal((*i as f64).cos()),
                        _ => unreachable!(),
                    }
                })],
                return_type: ASType::Decimal,
            },
        ),
        ASVar::new_with_value(
            "PI",
            Some(ASType::Decimal),
            true,
            ASObj::ASDecimal(std::f64::consts::PI),
        ),
    ]))
});
