use serde::Serialize;

use crate::compiler::value::Closure;

const BYTECODE_VERSION: u8 = 1;


impl Serialize for Closure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
