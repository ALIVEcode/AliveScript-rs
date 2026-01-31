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
    ({}, $body:tt $(, $else:tt)?) => {
        $($else)?
    };
    ({$($value:tt)+}, $body:tt $(, $else:tt)?) => {
        $body
    };
}

#[macro_export]
macro_rules! unpack {
    ($pat:pat = $expr:expr) => {
        let $pat = $expr else { unreachable!() };
    };
}

#[macro_export]
macro_rules! unpack_native {
    ($var:ident: &$t:ty = $expr:expr => $transform:block else $or_else:block) => {
        let $var = match $expr {
            ref o @ $crate::compiler::obj::Value::NativeObjet(..) => {
                $crate::unpack!(Value::NativeObjet(inner) = o);
                let inner = ::std::sync::Arc::clone(inner).as_any();
                if let Some($var) = inner.downcast_ref::<$t>() {
                    $transform
                } else {
                    $or_else
                }
            }
            _ => $or_else
        };
    };
    ($var:ident: &$t:ty = $expr:expr) => {
        $crate::unpack!(Value::NativeObjet(ref inner) = $expr);
        let inner = ::std::sync::Arc::clone(inner).as_any();
        let Some($var) = inner.downcast_ref::<$t>() else {
            return Err($crate::runtime::err::RuntimeError::generic_err(format!(
                "Objet invalide {:?}",
                inner
            )));
        };
    };
}

#[macro_export]
macro_rules! as_fonction {
    ($({$($prefix:stmt)*})? $($desc:literal;)? $name:ident $([$vm:ident])? ($(*$varargs:ident)?)
     $(: $return_type:expr =>)? $body:block) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new(true, $crate::compiler::value::Type::Fonction, $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.iter());
                    $(let $varargs = args;)?
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            })))
         )
     }};
    ($({$($prefix:stmt)*})? $($desc:literal;)? $name:ident $([$vm:ident])? ($($param_name:ident : {$($param_type:tt)+} $(=> $default:expr)?),+ $(, $(*$varargs:ident)?)?)
     $(: $return_type:expr =>)? $body:block) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new(true, $crate::compiler::value::Type::Fonction, $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.iter());
                    $(
                    let $param_name = {
                        let p = args.pop_front();
                        $crate::optional_body!({$($default)?}, {p.unwrap_or_else(|| $(&$default)?)}, {p.unwrap()})
                    };
                    if !$crate::compiler::value::Type::type_match(&$crate::as_type!($($param_type)+), &$param_name.get_type()) {
                        return Err($crate::runtime::err::RuntimeError::invalid_arg_type(
                            std::stringify!($name),
                            std::stringify!($param_name),
                            $crate::as_type!($($param_type)+),
                            $param_name.get_type(),
                         ));
                    }
                    )*
                    $($(let $varargs = args;)?)?
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            })))
         )
     }};
}

#[macro_export]
macro_rules! as_module_fonction {
    ($({$($prefix:stmt)*})? $($desc:literal;)? fonction $name:ident $([$vm:ident])? ($($param_name:ident : {$($param_type:tt)+} $(=> $default:expr)?),* $(, $(*$varargs:ident)?)?)
     $(: $return_type:expr =>)? $body:block fin fonction) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new_with_type_from_value(true,
            $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<$crate::compiler::obj::Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.into_iter());
                    $(
                    let $param_name = {
                        if let Some(p) = args.pop_front() {
                            p
                        } else {
                            $crate::optional_body!({$($default)?},
                                { $($default)? },
                                {
                                    return Err($crate::runtime::err::RuntimeError::call_error(stringify!($name), format!("valeur manquante pour le paramètre '{}'", stringify!($param_name))));
                                }
                            )
                        }

                    };

                    if !$crate::compiler::value::Type::type_match(&$crate::as_type!($($param_type)+), &$param_name.get_type()) {
                        return Err($crate::runtime::err::RuntimeError::invalid_arg_type(
                            std::stringify!($name),
                            std::stringify!($param_name),
                            $crate::as_type!($($param_type)+),
                            $param_name.get_type(),
                         ));
                    }
                    )*

                    $($(let $varargs = args;)?)?
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            })))
         )
     }};
    ($({$($prefix:stmt)*})? $($desc:literal;)? $name:ident $([$vm:ident])? ($(*$varargs:ident)?)
     $(: $return_type:expr =>)? $body:block) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new_with_type_from_value(true,
            $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<$crate::compiler::obj::Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.into_iter());
                    $($(let $varargs = args;)?)?
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            })))
         )
     }};
    ($({$($prefix:stmt)*})? $($desc:literal;)? $name:ident $([$vm:ident])? ($($param_name:ident : {$($param_type:tt)+} $(=> $default:expr)?),* $(, $(*$varargs:ident)?)?)
     $(: $return_type:expr =>)? $body:block) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new_with_type_from_value(true,
            $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc.to_string())?)),
                func: std::sync::Arc::new(move |_vm: &mut $crate::runtime::vm::VM, args: std::vec::Vec<$crate::compiler::obj::Value>| {
                    let mut args = std::collections::VecDeque::from_iter(args.into_iter());
                    $(
                    let $param_name = {
                        if let Some(p) = args.pop_front() {
                            p
                        } else {
                            $crate::optional_body!({$($default)?},
                                { $($default)? },
                                {
                                    return Err($crate::runtime::err::RuntimeError::call_error(stringify!($name), format!("valeur manquante pour le paramètre '{}'", stringify!($param_name))));
                                }
                            )
                        }

                    };

                    if !$crate::compiler::value::Type::type_match(&$crate::as_type!($($param_type)+), &$param_name.get_type()) {
                        return Err($crate::runtime::err::RuntimeError::invalid_arg_type(
                            std::stringify!($name),
                            std::stringify!($param_name),
                            $crate::as_type!($($param_type)+),
                            $param_name.get_type(),
                         ));
                    }
                    )*

                    $($(let $varargs = args;)?)?
                    $(let $vm = _vm;)?
                    $body
                }),
                // $return_type,
            })))
         )
     }};
}

#[macro_export]
macro_rules! as_module_var {
    ($({$($prefix:stmt)*})? const $varname:ident: {$($vartype:tt)+} = $val:expr) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new(true, $crate::as_type!($($vartype)+), $val)
         )
     }};
    ($({$($prefix:stmt)*})? var $varname:ident: {$($vartype:tt)+} = $val:expr) => {{
         $($($prefix)*)?;
         (
            String::from(std::stringify!($name)),
            $crate::compiler::value::ASField::new(false, $crate::as_type!($($vartype)+), $val)
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
                $crate::optional_body!({$($real_name)?}, {$($real_name)?}, {stringify!($name)})
            }

            fn load(&$self) -> $crate::compiler::value::ArcModule {
                $($($body)*)?
                $crate::compiler::value::ASModule::from_iter(stringify!($name), [$($var),*])
            }
        }
    };
}

#[macro_export]
macro_rules! as_type {
    ($name:literal $(| $($rest:tt)+)?) => {
        $crate::optional_body!(
            {$($($rest)+)?},
            {$crate::compiler::value::Type::union_of(
                $crate::compiler::value::Type::Objet($name.to_string()),
                $crate::as_type!($($($rest)+)?)
            )},
            {$crate::compiler::value::Type::Objet($name.to_string())}
        )
    };
    (Optionnel ($($arg:tt)+) $(| $($rest:tt)+)?) => {
        $crate::optional_body!(
            {$($($rest)+)?},
            {$crate::compiler::value::Type::union_of(
                $crate::compiler::value::Type::Optional(Box::new($crate::as_type!($($arg)+))),
                $crate::as_type!($($($rest)+)?)
            )},
            {$crate::compiler::value::Type::Optional(Box::new($crate::as_type!($($arg)+)))}
        )
    };
    (Dict ($($arg:tt)+) $(| $($rest:tt)+)?) => {
        $crate::optional_body!(
            {$($($rest)+)?},
            {$crate::compiler::value::Type::union_of(
                $crate::compiler::value::Type::Dict(Box::new($crate::compiler::value::Type::Texte), Box::new($crate::as_type!($($arg)+))),
                $crate::as_type!($($($rest)+)?)
            )},
            {$crate::compiler::value::Type::Dict(Box::new($crate::compiler::value::Type::Texte), Box::new($crate::as_type!($($arg)+)))}
        )
    };
    (Liste ($($arg:tt)+) $(| $($rest:tt)+)?) => {
        $crate::optional_body!(
            {$($($rest)+)?},
            {$crate::compiler::value::Type::union_of(
                $crate::compiler::value::Type::Liste(Box::new($crate::as_type!($($arg)+))),
                $crate::as_type!($($($rest)+)?)
            )},
            {$crate::compiler::value::Type::Liste(Box::new($crate::as_type!($($arg)+)))}
        )
    };
    ($name:ident($($args:tt)*) $(| $($rest:tt)+)?) => {
        $crate::optional_body!(
            {$($($rest)+)?},
            {$crate::compiler::value::Type::union_of(
                $crate::compiler::value::Type::$name($($args)*),
                $crate::as_type!($($($rest)+)?)
            )},
            {$crate::compiler::value::Type::$name($($args)*)}
        )
    };
    ($name:ident $(| $($rest:tt)+)?) => {
        $crate::optional_body!(
            {$($($rest)+)?},
            {$crate::compiler::value::Type::union_of(
                $crate::compiler::value::Type::$name,
                $crate::as_type!($($($rest)+)?)
            )},
            {$crate::compiler::value::Type::$name}
        )
    };
}
