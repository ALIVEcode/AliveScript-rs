use crate::as_obj::{ASObj, ASType};
use crate::{as_fonction, as_mod, as_var};

as_mod! {
    MATH_MOD,
    as_fonction! {
        sin(x: ASType::nombre()) -> ASType::Decimal; {
            Ok(Some(match x {
                    ASObj::ASDecimal(n) => ASObj::ASDecimal(n.sin()),
                    ASObj::ASEntier(i) => ASObj::ASDecimal((i as f64).sin()),
                    _ => unreachable!(),
                }))
        }
    },
    as_fonction! {
        cos(x: ASType::nombre()) -> ASType::Decimal; {
            Ok(Some(match x {
                    ASObj::ASDecimal(n) => ASObj::ASDecimal(n.cos()),
                    ASObj::ASEntier(i) => ASObj::ASDecimal((i as f64).cos()),
                    _ => unreachable!(),
                }))
        }
    },
    as_fonction! {
        tan(x: ASType::nombre()) -> ASType::Decimal; {
            Ok(Some(match x {
                ASObj::ASDecimal(i) => ASObj::ASDecimal(i.tan()),
                ASObj::ASEntier(i) => ASObj::ASDecimal((i as f64).tan()),
                _ => unreachable!()
            }))
        }
    },
    as_var!(const PI: ASType::Decimal => ASObj::ASDecimal(std::f64::consts::PI)),
}
