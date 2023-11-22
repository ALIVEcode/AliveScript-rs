use std::rc::Rc;

use once_cell::sync::Lazy;

use crate::as_obj::{ASFnParam, ASObj, ASScope, ASType, ASVar};

pub const TEMPS_MOD: Lazy<Rc<ASScope>> = Lazy::new(|| {
    Rc::new(ASScope::from(vec![ASVar::new_with_value(
        "maj",
        Some(ASType::Fonction),
        true,
        ASObj::native_fn(
            "maj",
            None,
            vec![ASFnParam {
                name: "txt".into(),
                static_type: ASType::Texte,
                default_value: None,
            }],
            |runner| {
                let env = runner.get_env();
                let ASObj::ASTexte(txt) = env.get_value(&"txt".into()).unwrap() else {
                    unreachable!()
                };
                Ok(Some(ASObj::ASTexte(txt.to_uppercase())))
            },
            ASType::Texte,
        ),
    )]))
});
