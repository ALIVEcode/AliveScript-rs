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

#[macro_export]
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
    (module $name:ident $(as $real_name:literal)? { $($struct_fields:tt)* }

    fn load(&$self:ident) {$({$($body:stmt)*})? [$($var:expr),* $(,)?]}) => {
        pub(crate) struct $name {
            $($struct_fields)*
        }

        impl $crate::stdlib::LazyModule for $name {
            fn name(&self) -> &'static str {
                $crate::optional_body!($($real_name)?, $($real_name)?, {stringify!($name)})
            }

            fn load(&$self) -> $crate::compiler::value::ArcModule {
                $($($body)*)?
                $crate::compiler::value::ASModule::from_iter(stringify!($name), [$($var),*])
            }
        }
    };
}
