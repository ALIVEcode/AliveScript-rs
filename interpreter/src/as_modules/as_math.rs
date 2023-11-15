use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::as_obj::{ASFnParam, ASObj, ASScope, ASType, ASVar};
use crate::fonction_as;

pub static MATH_MOD: Lazy<Arc<ASScope>> = Lazy::new(|| {
    Arc::new(ASScope::from(vec![
        ASVar::new_with_value(
            "sin",
            Some(ASType::Fonction),
            true,
            ASObj::native_fn(
                "sin",
                None,
                vec![ASFnParam {
                    name: "x".into(),
                    static_type: ASType::nombre(),
                    default_value: None,
                }],
                |runner| {
                    let env = runner.get_env();
                    Ok(Some(match env.get_var(&"x".into()).unwrap() {
                        (_, ASObj::ASDecimal(n)) => ASObj::ASDecimal(n.sin()),
                        (_, ASObj::ASEntier(i)) => ASObj::ASDecimal((*i as f64).sin()),
                        _ => unreachable!(),
                    }))
                },
                ASType::Decimal,
            ),
        ),
        fonction_as! {
            cos(x: ASType::nombre()) -> ASType::Decimal; {
                Ok(Some(match x {
                        ASObj::ASDecimal(n) => ASObj::ASDecimal(n.cos()),
                        ASObj::ASEntier(i) => ASObj::ASDecimal((*i as f64).cos()),
                        _ => unreachable!(),
                    }))
            }
        },
        fonction_as! {
            tan(x: ASType::nombre()) -> ASType::Decimal; {
                Ok(Some(match x {
                    ASObj::ASDecimal(i) => ASObj::ASDecimal(i.tan()),
                    ASObj::ASEntier(i) => ASObj::ASDecimal((*i as f64).tan()),
                    _ => unreachable!()
                }))
            }
        },
        ASVar::new_with_value(
            "PI",
            Some(ASType::Decimal),
            true,
            ASObj::ASDecimal(std::f64::consts::PI),
        ),
    ]))
});
