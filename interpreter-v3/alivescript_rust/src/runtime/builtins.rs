use crate::as_fonction;
use std::{collections::HashMap, sync::LazyLock};

use crate::compiler::obj::Value;
use crate::compiler::value::Type;
use crate::runtime::err::RuntimeError;

pub const BUILTINS: LazyLock<HashMap<String, Value>> = std::sync::LazyLock::new(|| {
    HashMap::from_iter([
        as_fonction! {
            afficher(msg: Type::any()): Type::Nul => {
                println!("{}", msg);
                Ok(Some(Value::Nul))
            }
        },
        as_fonction! {
            afficherErr(msg: Type::any()): Type::Nul => {
                eprintln!("{}", msg);
                Ok(Some(Value::Nul))
            }
        },
        as_fonction! {
            typeDe(obj: Type::any()): Type::Type => {
                Ok(Some(Value::TypeObj(obj.get_type())))
            }
        },
        as_fonction! {
            tailleDe(obj: Type::iterable()): Type::Entier => {
                Ok(Some(Value::Entier(match obj {
                    Value::Texte(t) => t.len(),
                    Value::Liste(l) => l.read().unwrap().len(),
                    _ => unreachable!()
                } as i64)))
            }
        },
        as_fonction! {
            entier(val: Type::Texte): Type::Entier => {
                let t = val.as_texte().unwrap();

                let i = t.parse::<i64>().map_err(|_| RuntimeError::generic_err(
                    format!("Impossible de convertir {} en entier de base 10.", val.repr())
                ))?;

                Ok(Some(Value::Entier(i)))
            }
        },
    ])
});
