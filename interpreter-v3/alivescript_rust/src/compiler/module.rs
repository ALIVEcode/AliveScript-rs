use crate::{as_mod, as_obj::ASObj, compiler::obj::Value};

#[macro_export]
macro_rules! as_fonction_native {
    ($($desc:literal;)? $name:ident $([$vm:ident])? ($($param_name:ident : $param_type:expr $(=> $default:expr)?),* $(,)?)
     -> $return_type:expr; $body:expr) => {
         (
            String::from(std::stringify!($name)),
            $crate::compiler::obj::NativeFunction {
                name: std::rc::Rc::new(String::from(std::stringify!($name))),
                desc: std::rc::Rc::new($crate::opt_value!($($desc)?)),
                // std::vec![$(
                // $crate::as_obj::ASFnParam {
                //     name: std::stringify!($param_name).into(),
                //     static_type: $param_type,
                //     default_value: $crate::default_value!($($default)?),
                // },
                // )*],
                func: std::rc::Rc::new(|vm: &mut $crate::compiler::vm::VM| {
                    $(
                    let $param_name = vm.pop().unwrap();
                    )*
                    $(let $vm = vm;)?
                    $body
                }),
                // $return_type,
            }
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
                hashmap.insert(name, $crate::compiler::obj::Value::NativeFunction(func));
            })*
            std::rc::Rc::new(std::cell::RefCell::new(hashmap))
        });

    };
}

as_mod_native! {
    BUILTIN_MOD,
    as_fonction_native! {
        afficherErr(msg: ASType::any()) -> ASType::Nul; {
            eprintln!("{}", msg);
            Ok(Some(Value::Nul))
        }
    },
    as_fonction_native! {
        afficher(msg: ASType::any()) -> ASType::Nul; {
            println!("{}", msg);
            Ok(Some(Value::Nul))
        }
    },
    as_fonction_native! {
        typeDe(obj: ASType::any()) -> ASType::Type; {
            Ok(Some(Value::TypeObj(obj.get_type())))
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
    // as_fonction_native! {
    //     tailleDe[runner](obj: ASType::iterable()) -> ASType::Entier; {
    //         if let Some(result) = call_methode!(obj.__taille__() or throw, runner) {
    //             return result;
    //         }
    //         Ok(Some(ASObj::ASEntier(match obj {
    //             ASObj::ASTexte(t) => t.len(),
    //             ASObj::ASListe(l) => l.borrow().len(),
    //             ASObj::ASDict(d) => d.borrow().len(),
    //             _ => unreachable!()
    //         } as i64)))
    //     }
    // },
}
