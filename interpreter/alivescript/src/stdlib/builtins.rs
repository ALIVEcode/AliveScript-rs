use crate::as_fonction;
use crate::runtime::vm::VM;
use std::{collections::HashMap, sync::LazyLock};

use crate::compiler::obj::Value;
use crate::compiler::value::Type;
use crate::runtime::err::RuntimeError;

pub const BUILTINS: LazyLock<HashMap<String, Value>> = std::sync::LazyLock::new(|| {
    HashMap::from_iter([
        as_fonction! {
            afficher(msg: Type::tout()): Type::Nul => {
                println!("{}", msg);
                Ok(Some(Value::Nul))
            }
        },
        as_fonction! {
            afficherErr(msg: Type::tout()): Type::Nul => {
                eprintln!("{}", msg);
                Ok(Some(Value::Nul))
            }
        },
        as_fonction! {
            typeDe(obj: Type::tout()): Type::Type => {
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
            suite(debut: Type::Entier, fin: Type::Entier): Type::Liste(Type::Entier) => {
                let debut = debut.as_entier().unwrap();
                let fin = fin.as_entier().unwrap();

                Ok(Some(Value::liste((debut..fin).map(|i| Value::Entier(i)).collect())))
            }
        },
        as_fonction! {
            abs(val: Type::nombre()): Type::nombre() => {
                Ok(Some(match val {
                    Value::Entier(i) => Value::Entier(i.abs()),
                    Value::Decimal(f) => Value::Decimal(f.abs()),
                    _ =>  unreachable!()
                }))
            }
        },
        as_fonction! {
            entier(val: Type::union_of(Type::Texte, Type::nombre())): Type::Entier => {
                Ok(Some(match val {
                    Value::Entier(i) => Value::Entier(*i),
                    Value::Decimal(f) => Value::Entier(*f as i64),
                    Value::Texte(t) => {
                        let i = t.parse::<i64>().map_err(|_| RuntimeError::generic_err(
                            format!("Impossible de convertir {} en entier de base 10.", val.repr())
                        ))?;

                        Value::Entier(i)
                    }
                    _ => unreachable!()
                }))
            }
        },
        as_fonction! {
            décimal(val: Type::union_of(Type::Texte, Type::nombre())): Type::Decimal => {
                Ok(Some(match val {
                    Value::Entier(i) => Value::Decimal(*i as f64),
                    Value::Decimal(f) => Value::Decimal(*f),
                    Value::Texte(t) => {
                        let f = t.parse::<f64>().map_err(|_| RuntimeError::generic_err(
                            format!("Impossible de convertir {} en décimal.", val.repr())
                        ))?;

                        Value::Decimal(f)
                    }
                    _ => unreachable!()
                }))
            }
        },
        as_fonction! {
            decimal(val: Type::union_of(Type::Texte, Type::nombre())): Type::Decimal => {
                Ok(Some(match val {
                    Value::Entier(i) => Value::Decimal(*i as f64),
                    Value::Decimal(f) => Value::Decimal(*f),
                    Value::Texte(t) => {
                        let f = t.parse::<f64>().map_err(|_| RuntimeError::generic_err(
                            format!("Impossible de convertir {} en décimal.", val.repr())
                        ))?;

                        Value::Decimal(f)
                    }
                    _ => unreachable!()
                }))
            }
        },
        as_fonction! {
            texte(val: Type::tout()): Type::Texte => {
                Ok(Some(Value::Texte(val.to_string())))
            }
        },
        as_fonction! {
            erreur(msg: Type::tout()): Type::Type => {
                Err(RuntimeError::generic_err(msg))
            }
        },
    ])
});
