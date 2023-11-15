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
macro_rules! fonction_as {
    ($($desc:literal;)? $name:ident ($($param_name:ident : $param_type:expr $(=> $default:expr)?),* $(,)?)
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
                |runner| {
                    let env = runner.get_env();
                    $(
                    let $param_name = env.get_value(&std::stringify!($param_name).into()).unwrap();
                    )*
                    $body
                },
                $return_type,
                ),
        )
    };
}

#[macro_export]
macro_rules! unpack_as {
    ($var:pat = $val:expr) => {
        let $var = $val else { std::unreachable!() };
    };
}
