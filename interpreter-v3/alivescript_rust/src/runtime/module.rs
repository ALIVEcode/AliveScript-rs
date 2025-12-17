use crate::compiler::{obj::Value, value::Type};

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
        afficherErr(msg: Type::any()): ASType::Nul => {
            eprintln!("{}", msg);
            Ok(Some(Value::Nul))
        }
    },
    as_fonction_native! {
        afficher(msg: Type::any()): ASType::Nul => {
            println!("{}", msg);
            Ok(Some(Value::Nul))
        }
    },
    as_fonction_native! {
        typeDe(obj: Type::any()): ASType::Type => {
            Ok(Some(Value::TypeObj(obj.get_type())))
        }
    },
    as_fonction_native! {
        tailleDe(obj: Type::iterable()): ASType::Entier => {
            Ok(Some(Value::Entier(match obj {
                Value::Texte(t) => t.len(),
                Value::Liste(l) => l.read().unwrap().len(),
                _ => unreachable!()
            } as i64)))
        }
    },
    // as_fonction_native! {
    //     typeVar[runner](nomVar: ASType::Texte) -> ASType::Texte; {
    //         let env = runner.get_env_mut();
    //         as_cast!(ASObj::ASTexte(nom_var) = nomVar);
    //         let maybe_var = env.get_var(&nom_var).map(|v| v.0);
    //         Ok(Some(match maybe_var {
    //             Some(var) => ASObj::ASTexte(var.get_type().to_string()),
    //             None => ASObj::ASNul,
    //         }))
    //     }
    // },
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
