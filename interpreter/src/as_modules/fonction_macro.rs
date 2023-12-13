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
macro_rules! default_value {
    () => {
        None
    };
    ($value:expr) => {
        Some($crate::ast::Expr::literal($value))
    };
}

#[macro_export]
macro_rules! as_fonction {
    ($($desc:literal;)? $name:ident $([$runner:ident])? ($($param_name:ident : $param_type:expr $(=> $default:expr)?),* $(,)?)
     -> $return_type:expr; $body:expr) => {
        $crate::as_obj::ASVar::new_with_value(
            std::stringify!($name),
            Some($crate::as_obj::ASType::Fonction),
            true,
            $crate::as_obj::ASObj::native_fn(
                std::stringify!($name),
                $crate::opt_value!($($desc)?),
                std::vec![$(
                $crate::as_obj::ASFnParam {
                    name: std::stringify!($param_name).into(),
                    static_type: $param_type,
                    default_value: $crate::default_value!($($default)?),
                },
                )*],
                #[allow(non_snake_case)]
                std::rc::Rc::new(|runner: &mut $crate::runner::Runner| {
                    let env = runner.get_env().clone();
                    $(
                    let $param_name = env.get_value(&std::stringify!($param_name).into()).unwrap();
                    )*
                    $(let $runner = runner;)?
                    $body
                }),
                $return_type,
                ),
        )
    };
}

#[macro_export]
macro_rules! as_var {
    ($var:ident : $param_type:expr => $val:expr) => {
        $crate::as_obj::ASVar::new_with_value(std::stringify!($var), Some($param_type), false, $val)
    };
    ($var:ident => $val:expr) => {
        $crate::as_obj::ASVar::new_with_value(std::stringify!($var), None, false, $val)
    };
    (const $var:ident : $param_type:expr => $val:expr) => {
        $crate::as_obj::ASVar::new_with_value(std::stringify!($var), Some($param_type), true, $val)
    };
    (const $var:ident => $val:expr) => {
        $crate::as_obj::ASVar::new_with_value(std::stringify!($var), None, true, $val)
    };
}
#[macro_export]
macro_rules! union_of {
    ($($type:expr),*) => {{
        #[allow(unused_imports)]
        use $crate::as_obj::ASType::*;
        $crate::as_obj::ASType::union(std::vec![$($type),*])
    }};
}

#[macro_export]
macro_rules! call_methode {
    ($obj:ident.$methode:ident($($args:expr),*), $runner:expr) => {
        if let $crate::as_obj::ASObj::ASClasseInst(ref inst) = $obj {
            let inst_env = inst.env().borrow();
            if let Some(meth) = inst_env.get_value(&stringify!($methode).to_owned()) {
                match meth {
                    $crate::as_obj::ASObj::ASMethode(..) => {
                        Some(Ok($runner.call_obj(meth.clone(), std::vec![$($args),*])))
                    }
                    _ => {
                        Some(Err($crate::as_obj::ASErreurType::new_erreur_type(
                                meth.get_type(),
                                $crate::as_obj::ASType::Fonction,
                            )))
                    }
                }
            } else {
                None 
            }
        } else {
            None
        }
    };
    ($obj:ident.$methode:ident($($args:expr),*) or throw, $runner:expr) => {
        if let $crate::as_obj::ASObj::ASClasseInst(ref inst) = $obj {
            let inst_env = inst.env().borrow();
            if let Some(meth) = inst_env.get_value(&stringify!($methode).to_owned()) {
                match meth {
                    $crate::as_obj::ASObj::ASMethode(..) => {
                        Some(Ok($runner.call_obj(meth.clone(), std::vec![$($args),*])))
                    }
                    _ => {
                        Some(Err($crate::as_obj::ASErreurType::new_erreur_type(
                                meth.get_type(),
                                $crate::as_obj::ASType::Fonction,
                            )))
                    }
                }
            } else {
                Some(Err($crate::as_obj::ASErreurType::new_erreur(None, format!("méthode {} non implémentée", stringify!($methode)))))
            }
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! as_cast {
    ($var:pat = $val:expr) => {
        let $var = $val else { std::unreachable!() };
    };
}

#[macro_export]
macro_rules! as_mod {
    ($name:ident, $($var:expr),* $(,)?) => {
        pub const $name: once_cell::sync::Lazy<std::rc::Rc<std::cell::RefCell<$crate::as_obj::ASScope>>> = once_cell::sync::Lazy::new(|| {
            std::rc::Rc::new(std::cell::RefCell::new($crate::as_obj::ASScope::from(std::vec![$($var),*])))
        });

    };
}
