use crate::as_fonction;
use crate::runtime::vm::VM;
use std::{collections::HashMap, sync::LazyLock};

use crate::compiler::obj::Value;
use crate::compiler::value::Type;
use crate::runtime::err::RuntimeError;

pub const BUILTINS: LazyLock<HashMap<String, Value>> = std::sync::LazyLock::new(|| {
    HashMap::from_iter([
        as_fonction! {
            afficher(*varargs): Type::Nul => {
                println!("{}", varargs.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "));
                Ok(Value::Nul)
            }
        },
        as_fonction! {
            afficherErr(msg: {Tout}): Type::Nul => {
                eprintln!("{}", msg);
                Ok(Value::Nul)
            }
        },
        as_fonction! {
            typeDe(obj: {Tout}): Type::Type => {
                Ok(Value::TypeObj(obj.get_type()))
            }
        },
        as_fonction! {
            tailleDe(obj: {iterable()}): Type::Entier => {
                Ok(Value::Entier(match obj {
                    Value::Texte(t) => t.len(),
                    Value::Liste(l) => l.read().unwrap().len(),
                    _ => unreachable!()
                } as i64))
            }
        },
        as_fonction! {
            suite(debut: {Entier}, fin: {Entier}): Type::Liste(Type::Entier) => {
                let debut = debut.as_entier().unwrap();
                let fin = fin.as_entier().unwrap();

                Ok(Value::liste((debut..fin).map(|i| Value::Entier(i)).collect()))
            }
        },
        as_fonction! {
            abs(val: {nombre()}): Type::nombre() => {
                Ok(match val {
                    Value::Entier(i) => Value::Entier(i.abs()),
                    Value::Decimal(f) => Value::Decimal(f.abs()),
                    _ =>  unreachable!()
                })
            }
        },
        as_fonction! {
            entier(val: {nombre() | Texte}): Type::Entier => {
                Ok(match val {
                    Value::Entier(i) => Value::Entier(*i),
                    Value::Decimal(f) => Value::Entier(*f as i64),
                    Value::Texte(t) => {
                        let i = t.parse::<i64>().map_err(|_| RuntimeError::generic_err(
                            format!("Impossible de convertir {} en entier de base 10.", val.repr())
                        ))?;

                        Value::Entier(i)
                    }
                    _ => unreachable!()
                })
            }
        },
        as_fonction! {
            décimal(val: {nombre() | Texte}): Type::Decimal => {
                Ok(match val {
                    Value::Entier(i) => Value::Decimal(*i as f64),
                    Value::Decimal(f) => Value::Decimal(*f),
                    Value::Texte(t) => {
                        let f = t.parse::<f64>().map_err(|_| RuntimeError::generic_err(
                            format!("Impossible de convertir {} en décimal.", val.repr())
                        ))?;

                        Value::Decimal(f)
                    }
                    _ => unreachable!()
                })
            }
        },
        as_fonction! {
            decimal(val: {nombre() | Texte}): Type::Decimal => {
                Ok(match val {
                    Value::Entier(i) => Value::Decimal(*i as f64),
                    Value::Decimal(f) => Value::Decimal(*f),
                    Value::Texte(t) => {
                        let f = t.parse::<f64>().map_err(|_| RuntimeError::generic_err(
                            format!("Impossible de convertir {} en décimal.", val.repr())
                        ))?;

                        Value::Decimal(f)
                    }
                    _ => unreachable!()
                })
            }
        },
        as_fonction! {
            texte(val: {Tout}): Type::Texte => {
                Ok(Value::Texte(val.to_string()))
            }
        },
        as_fonction! {
            erreur(msg: {Tout}): Type::Type => {
                Err(RuntimeError::generic_err(msg))
            }
        },
    ])
});
