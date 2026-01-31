use core::time;
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dyn_fmt::AsStrFormatExt;
use rand::random_range;
use uuid::timestamp;

use crate::{as_module, as_module_fonction, unpack};
use crate::{
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::err::RuntimeError,
};

use toml;

impl From<toml::Value> for Value {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(s) => Value::Texte(s),
            toml::Value::Integer(i) => Value::Entier(i),
            toml::Value::Float(f) => Value::Decimal(f),
            toml::Value::Boolean(b) => Value::Booleen(b),
            toml::Value::Datetime(datetime) => todo!(),
            toml::Value::Array(values) => {
                Value::liste(values.into_iter().map(|v| Value::from(v)).collect())
            }
            toml::Value::Table(map) => Value::dict(HashMap::from_iter(
                map.into_iter().map(|(k, v)| (k, Value::from(v))),
            )),
        }
    }
}

impl TryFrom<Value> for toml::Value {
    type Error = RuntimeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Entier(i) => todo!(),
            Value::Decimal(f) => todo!(),
            Value::Booleen(b) => todo!(),
            Value::Texte(s) => todo!(),
            Value::Liste(rw_lock) => todo!(),
            Value::Dict(rw_lock) => todo!(),
            Value::Nul | Value::Vide => Err(RuntimeError::generic_err(
                "Les valeurs nul sont non permises dans le format Toml",
            ))?,
            Value::Function(f) => todo!(),
            Value::TypeObj(_) => todo!(),
            Value::Structure(rw_lock) => todo!(),
            Value::Objet(rw_lock) => todo!(),
            Value::Module(rw_lock) => todo!(),
            Value::NativeObjet(native_objet) => todo!(),
        })
    }
}

as_module! {
    module Toml {}

    fn load(&self) {
        [
            as_module_fonction! {
                fonction charger(t: {Texte}) {
                    let val: toml::Value = toml::from_str(t.as_texte()?).map_err(
                        |e| RuntimeError::generic_err(format!("Erreur avec le Toml: {}", e))
                    )?;

                    Ok(Value::from(val))
                } fin fonction
            },
            as_module_fonction! {
                fonction textifier(t: {Tout}) {
                    Ok(Value::Texte(
                        toml::to_string(&<toml::Value as TryFrom<Value>>::try_from(t)?)
                            .map_err(|e| RuntimeError::generic_err(e))?
                    ))
                } fin fonction
            },
        ]
    }
}
