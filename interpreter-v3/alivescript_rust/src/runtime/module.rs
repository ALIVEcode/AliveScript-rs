use crate::{
    compiler::{obj::Value, value::Type},
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
macro_rules! as_fonction_native {
    ($($desc:literal;)? $name:ident $([$vm:ident])? ($($param_name:ident : $param_type:expr $(=> $default:expr)?),* $(,)?)
     : $return_type:expr => $body:block) => {
         (
            String::from(std::stringify!($name)),
            $crate::compiler::obj::Value::Function($crate::compiler::obj::Function::NativeFunction($crate::compiler::value::NativeFunction {
                name: std::sync::Arc::new(String::from(std::stringify!($name))),
                desc: std::sync::Arc::new($crate::opt_value!($($desc)?)),
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
     };
}

#[macro_export]
macro_rules! as_mod_native {
    ($name:ident, $($var:expr),* $(,)?) => {
        pub const $name: once_cell::sync::Lazy<std::rc::Rc<std::cell::RefCell<::std::collections::HashMap<String, $crate::compiler::obj::Value>>>> = once_cell::sync::Lazy::new(|| {
            let mut hashmap = ::std::collections::HashMap::new();
            $({
                let (name, func) = $var;
                hashmap.insert(name, func);
            })*
            std::rc::Rc::new(std::cell::RefCell::new(hashmap))
        });

    };
}

as_mod_native! {
    BUILTIN_MOD,
    as_fonction_native! {
        afficher(msg: Type::any()): Type::Nul => {
            println!("{}", msg);
            Ok(Some(Value::Nul))
        }
    },
    as_fonction_native! {
        afficherErr(msg: Type::any()): Type::Nul => {
            eprintln!("{}", msg);
            Ok(Some(Value::Nul))
        }
    },
    as_fonction_native! {
        typeDe(obj: Type::any()): Type::Type => {
            Ok(Some(Value::TypeObj(obj.get_type())))
        }
    },
    as_fonction_native! {
        tailleDe(obj: Type::iterable()): Type::Entier => {
            Ok(Some(Value::Entier(match obj {
                Value::Texte(t) => t.len(),
                Value::Liste(l) => l.read().unwrap().len(),
                _ => unreachable!()
            } as i64)))
        }
    },
    as_fonction_native! {
        entier(val: Type::Texte): Type::Entier => {
            let t = val.as_texte().unwrap();

            let i = t.parse::<i64>().map_err(|_| RuntimeError::generic_err(
                format!("Impossible de convertir {} en entier de base 10.", val.repr())
            ))?;

            Ok(Some(Value::Entier(i)))
        }
    },
    // as_fonction_native! {
    //     format(template: ASType::Texte, attrs: ASType::Liste => ASObj::liste(vec![])) -> ASType::Texte; {
    //         as_cast!(ASObj::ASTexte(template) = template);
    //         as_cast!(ASObj::ASListe(attrs) = attrs);
    //
    //         let result = template.format(attrs.borrow().iter());
    //         Ok(Some(ASObj::ASTexte(result)))
    //     }
    // },
}
