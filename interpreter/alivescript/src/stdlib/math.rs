use core::time;
use std::{
    collections::HashMap,
    marker::PhantomData,
    process::exit,
    sync::{Arc, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dyn_fmt::AsStrFormatExt;
use rand::random_range;
use uuid::timestamp;

use crate::{as_module, as_module_fonction, as_module_var, unpack};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

as_module! {
    module Math {}

    fn load(&self) {
        [
            as_module_var!(const PI: {Decimal} = Value::Decimal(std::f64::consts::PI)),
            as_module_var!(const E: {Decimal} = Value::Decimal(std::f64::consts::E)),
            as_module_var!(const TAU: {Decimal} = Value::Decimal(std::f64::consts::TAU)),
            as_module_var!(const INF: {Decimal} = Value::Decimal(f64::INFINITY)),
            as_module_var!(const INF_NÉG: {Decimal} = Value::Decimal(f64::NEG_INFINITY)),
            as_module_var!(const PUN: {Decimal} = Value::Decimal(f64::NAN)),
            as_module_var!(const NAN: {Decimal} = Value::Decimal(f64::NAN)),

            as_module_fonction! {
                plafond(x: {nombre()}) {
                    Ok(x.do_math_op(|x| x.ceil())?)
                }
            },
            as_module_fonction! {
                plancher(x: {nombre()}) {
                    Ok(x.do_math_op(|x| x.floor())?)
                }
            },
            as_module_fonction! {
                tronquer(x: {nombre()}) {
                    Ok(x.do_math_op(|x| x.trunc())?)
                }
            },
            as_module_fonction! {
                arrondir(x: {nombre()}, precision: {Entier} => Value::Entier(0)) {
                    let precision = (10 as f64).powi(precision.as_entier()? as i32);
                    Ok(x.do_math_op(|x| (x * precision).round() / precision)?)
                }
            },
            as_module_fonction! {
                abs(x: {nombre()}) {
                    Ok(x.do_math_op(|x| x.abs())?)
                }
            },
            as_module_fonction! {
                racine(x: {nombre()}, n: {nombre()} => Value::Entier(2)) {
                    let n = n.as_decimal()?;
                    Ok(Value::Decimal(x.as_decimal()?.powf(1.0 / n)))
                }
            },
            as_module_fonction! {
                exp(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.exp()))
                }
            },
            as_module_fonction! {
                ln(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.ln()))
                }
            },
            as_module_fonction! {
                log(x: {nombre()}, base: {nombre()} => Value::Entier(10)) {
                    let base = base.as_decimal()?;
                    if base == 10.0 {
                        Ok(Value::Decimal(x.as_decimal()?.log10()))
                    } else if base == 2.0 {
                        Ok(Value::Decimal(x.as_decimal()?.log2()))
                    } else {
                        Ok(Value::Decimal(x.as_decimal()?.log(base)))
                    }
                }
            },
            as_module_fonction! {
                cos(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.cos()))
                }
            },
            as_module_fonction! {
                sin(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.sin()))
                }
            },
            as_module_fonction! {
                tan(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.tan()))
                }
            },
            as_module_fonction! {
                cosh(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.cosh()))
                }
            },
            as_module_fonction! {
                sinh(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.sinh()))
                }
            },
            as_module_fonction! {
                tanh(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.tanh()))
                }
            },
            as_module_fonction! {
                acos(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.acos()))
                }
            },
            as_module_fonction! {
                asin(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.asin()))
                }
            },
            as_module_fonction! {
                atan(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.atan()))
                }
            },
            as_module_fonction! {
                atan2(x: {nombre()}, y: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.atan2(y.as_decimal()?)))
                }
            },
            as_module_fonction! {
                enDegrés(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.to_degrees()))
                }
            },
            as_module_fonction! {
                enRadians(x: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.to_radians()))
                }
            },
            as_module_fonction! {
                hypot(x: {nombre()}, y: {nombre()}) {
                    Ok(Value::Decimal(x.as_decimal()?.hypot(y.as_decimal()?)))
                }
            },
        ]
    }
}
