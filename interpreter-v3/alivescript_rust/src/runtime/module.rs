use std::{collections::HashMap, marker::PhantomData, sync::LazyLock};

use crate::{
    compiler::{
        obj::Value,
        value::{ArcModule, Type},
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

    fn load(&$self:ident) {$($body:stmt);*; [$($var:expr),* $(,)?]}) => {
        struct $name {
            $($struct_fields)*
        }

        impl $crate::runtime::module::LazyModule for $name {
            fn load(&$self) -> ArcModule {
                $($body)*
                $crate::compiler::value::ASModule::from_iter(stringify!($name), [$($var),*])
            }
        }
    };
}

as_module! {
    UNIT, "Unit",
    as_fonction! {
        entier(val: Type::Texte): Type::Entier => {
            let t = val.as_texte().unwrap();

            let i = t.parse::<i64>().map_err(|_| RuntimeError::generic_err(
                format!("Impossible de convertir {} en entier de base 10.", val.repr())
            ))?;

            Ok(Some(Value::Entier(i)))
        }
    },
}

pub fn get_stdlib() -> HashMap<String, LazyLock<ArcModule>> {
    HashMap::from_iter([UNIT].map(|(n, lazy_module)| (n.to_string(), lazy_module)))
}

pub trait LazyModule {
    fn load(&self) -> ArcModule;
}
