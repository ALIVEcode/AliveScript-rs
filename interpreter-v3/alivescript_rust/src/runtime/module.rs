use core::time;
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use rand::random_range;
use uuid::timestamp;

use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

#[macro_export]
macro_rules! opt_value {
    () => {
        None
    };
    ($value:expr) => {
        Some($value)
    };
}

#[macro_export]
macro_rules! optional_body {
    ($value:tt, $body:tt $(, $else:tt)?) => {
        $body
    };
    (, $body:tt $(, $else:tt)?) => {
        $($else)?
    };
}

macro_rules! unpack {
    ($pat:pat = $expr:expr) => {
        let $pat = $expr else { unreachable!() };
    };
}

#[macro_export]
macro_rules! as_fonction {
    ($({$($prefix:stmt)*})? $($desc:literal;)? $name:ident $([$vm:ident])? ($($param_name:ident : $param_type:expr $(=> $default:expr)?),* $(,)?)
     : $return_type:expr => $body:block) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                // std::vec![$(
                // $crate::as_obj::ASFnParam {
                //     name: std::stringify!($param_name).into(),
                //     static_type: $param_type,
                //     default_value: $crate::default_value!($($default)?),
                // },
                // )*],
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.iter());
                    $(
                    let $param_name = {
                        let p = args.pop_front();
                        $crate::optional_body!($($default)?, {p.unwrap_or_else(|| $($default)?)}, {p.unwrap()})
                    };
                    if !$crate::compiler::value::Type::type_match(&$param_type, &$param_name.get_type()) {
                        return Err($crate::runtime::err::RuntimeError::invalid_arg_type(
                            std::stringify!($name),
                            std::stringify!($param_name),
                            $param_type,
                            $param_name.get_type(),
                         ));
                    }
                    )*
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            }))
         )
     }};
}
#[macro_export]
macro_rules! as_module_fonction {
    ($({$($prefix:stmt)*})? $($desc:literal;)? $name:ident $([$vm:ident])? ($($param_name:ident : $param_type:expr $(=> $default:expr)?),* $(,)?)
     : $return_type:expr => $body:block) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new_with_type_from_value(true,
            $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                // std::vec![$(
                // $crate::as_obj::ASFnParam {
                //     name: std::stringify!($param_name).into(),
                //     static_type: $param_type,
                //     default_value: $crate::default_value!($($default)?),
                // },
                // )*],
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.iter());
                    $(
                    let $param_name = {
                        let p = args.pop_front();
                        $crate::optional_body!($($default)?, {p.unwrap_or_else(|| $($default)?)}, {p.unwrap()})
                    };
                    if !$crate::compiler::value::Type::type_match(&$param_type, &$param_name.get_type()) {
                        return Err($crate::runtime::err::RuntimeError::invalid_arg_type(
                            std::stringify!($name),
                            std::stringify!($param_name),
                            $param_type,
                            $param_name.get_type(),
                         ));
                    }
                    )*
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            })))
         )
     }};
}

#[macro_export]
macro_rules! as_module {
    ($var_name:ident, $name:literal, $($var:expr),* $(,)?) => {
        pub const $var_name: (&'static str, std::sync::LazyLock<$crate::compiler::value::ArcModule>) = ($name, std::sync::LazyLock::new(|| {
            $crate::compiler::value::ASModule::from_iter($name, [$($var),*])
        }));

    };
}

#[macro_export]
macro_rules! as_module2 {
    (module $name:ident { $($struct_fields:tt)* }

    fn load(&$self:ident) {$({$($body:stmt)*})? [$($var:expr),* $(,)?]}) => {
        struct $name {
            $($struct_fields)*
        }

        impl $crate::runtime::module::LazyModule for $name {
            fn load(&$self) -> ArcModule {
                $($($body)*)?
                $crate::compiler::value::ASModule::from_iter(stringify!($name), [$($var),*])
            }
        }
    };
}

as_module2! {
    module Test { }

    fn load(&self) {
        [
            as_module_fonction! {
                affirmer(cond: Type::tout(), msg: Type::Texte): Type::Nul => {
                    if !cond.to_bool() {
                        let t = msg.as_texte().unwrap();
                        Err(RuntimeError::assertion_error(t))
                    } else {
                        Ok(None)
                    }
                }
            },
            as_module_fonction! {
                affirmerÉgaux(val1: Type::tout(), val2: Type::tout(), msg: Type::Texte): Type::Nul => {
                    if val1 != val2 {
                        let t = msg.as_texte().unwrap();
                        Err(RuntimeError::assertion_error(format!("{}\nGauche: {}\nDroite: {}", t, val1.repr(), val2.repr())))
                    } else {
                        Ok(None)
                    }
                }
            },
        ]
    }
}
as_module2! {
    module Texte {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: Type::Texte): Type::Entier => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Some(Value::Entier(inst.len() as i64)))
                }
            },
            as_module_fonction! {
                est_numerique(inst: Type::Texte): Type::Booleen => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Some(Value::Booleen(inst.chars().all(|c| c.is_ascii_digit()))))
                }
            },
        ]
    }
}
as_module2! {
    module Liste {}

    fn load(&self) {
        [
            as_module_fonction! {
                taille(inst: Type::liste_tout()): Type::Entier => {
                    unpack!(Value::Liste(lst) = inst);

                    Ok(Some(Value::Entier(lst.read().unwrap().len() as i64)))
                }
            },
            as_module_fonction! {
                ajouter(inst: Type::liste_tout(), val: Type::Tout): Type::Nul => {
                    unpack!(Value::Liste(lst) = inst);

                    lst.write().unwrap().push(val.clone());

                    Ok(Some(Value::Nul))
                }
            },
        ]
    }
}
as_module2! {
    module Aleatoire {}

    fn load(&self) {
        [
            as_module_fonction! {
                choix(iter: Type::iterable()): Type::Tout => {
                    match iter {
                        Value::Liste(lst) => {
                            let i = random_range(0..lst.read().unwrap().len());
                            Ok(Some(lst.read().unwrap()[i].clone()))
                        }
                        _ => Err(RuntimeError::value_error(
                            format!("dans la fonction 'Alétoire.choix', argument #1 invalide '{}'", iter.get_type())
                        ))
                    }
                }
            },
            as_module_fonction! {
                entre(min: Type::Entier, max: Type::Entier): Type::Entier => {
                    let min = min.as_entier().unwrap();
                    let max = max.as_entier().unwrap();
                    let i = random_range(min..max);
                    Ok(Some(Value::Entier(i)))
                }
            },
        ]
    }
}
as_module2! {
    module Systeme {}

    fn load(&self) {
        [
            as_module_fonction! {
                temps(): Type::Entier => {
                    Ok(Some(Value::Entier(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64
                    )))
                }
            },
        ]
    }
}

as_module2! {
    module Debug {}

    fn load(&self) {
        [
            as_module_fonction! {
                nb(inst: Type::Texte): Type::Booleen => {
                    let inst = inst.as_texte().unwrap();
                    Ok(Some(Value::Booleen(inst.chars().all(|c| c.is_ascii_digit()))))
                }
            },
        ]
    }
}

pub fn get_stdlib() -> HashMap<String, Arc<dyn LazyModule>> {
    let mut stdlib: HashMap<String, Arc<dyn LazyModule>> = HashMap::new();

    stdlib.insert("Texte".to_string(), Arc::new(Texte {}));
    stdlib.insert("Liste".to_string(), Arc::new(Liste {}));
    stdlib.insert("Test".to_string(), Arc::new(Test {}));
    stdlib.insert("Système".to_string(), Arc::new(Systeme {}));
    stdlib.insert("Aléatoire".to_string(), Arc::new(Aleatoire {}));
    stdlib.insert("Debug".to_string(), Arc::new(Debug {}));

    stdlib
}

pub trait LazyModule {
    fn load(&self) -> ArcModule;
}
