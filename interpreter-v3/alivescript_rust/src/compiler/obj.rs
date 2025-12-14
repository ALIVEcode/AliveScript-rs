use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::sync::{Arc, Mutex, RwLock};

use crate::as_obj::{self, ASErreurType};
use crate::ast::CallRust;
use crate::compiler::value::{ASStructure, BaseType, Closure, Function, NativeFunction};
use crate::runtime::err::RuntimeError;
use crate::runtime::vm::VM;

pub type ArcUpvalue = Arc<RwLock<Upvalue>>;
pub type ArcClosure = Arc<Closure>;
pub type ArcFunction = Arc<Function>;

#[derive(Debug, Clone)]
pub enum Value {
    Entier(i64),
    Decimal(f64),
    Booleen(bool),
    Nul,
    Texte(String),
    Closure(ArcClosure),
    NativeFunction(NativeFunction),
    Liste(Arc<RwLock<Vec<Value>>>),
    TypeObj(BaseType),
    Structure(Arc<RwLock<ASStructure>>),
}

impl Value {
    pub fn liste(values: Vec<Value>) -> Self {
        Self::Liste(Arc::new(RwLock::new(values)))
    }

    pub fn get_type(&self) -> BaseType {
        use Value as V;

        match self {
            V::Entier(..) => BaseType::Entier,
            V::Decimal(..) => BaseType::Decimal,
            V::Texte(..) => BaseType::Texte,
            V::Nul => BaseType::Nul,
            V::Booleen(..) => BaseType::Booleen,
            V::Liste(lst) => BaseType::Liste(Box::new(
                lst.read()
                    .unwrap()
                    .iter()
                    .map(|v| v.get_type())
                    .reduce(|t1, t2| BaseType::union_of(t1, t2))
                    .unwrap_or(BaseType::Tout),
            )),
            V::Closure(..) => BaseType::Fonction,
            V::NativeFunction(..) => BaseType::Fonction,
            V::TypeObj(t) => BaseType::Type,
            as_type => todo!("Type inconnue {:?}", as_type),
        }
    }

    pub fn to_bool(&self) -> bool {
        use Value as V;

        match self {
            V::Entier(x) => *x != 0,
            V::Decimal(x) => *x != 0f64,
            V::Texte(s) => !s.is_empty(),
            V::Booleen(b) => *b,
            V::Nul => false,
            V::Liste(l) => !l.read().unwrap().is_empty(),
            _ => true,
        }
    }

    pub fn div_int(&self, rhs: Self) -> Result<Value, RuntimeError> {
        use Value as V;

        Ok(match (self, rhs) {
            (V::Entier(x), V::Entier(y)) => V::Entier(x / y),
            (V::Decimal(x), V::Entier(y)) => V::Entier(*x as i64 / y),
            (V::Entier(x), V::Decimal(y)) => V::Entier(x / y as i64),
            (V::Decimal(x), V::Decimal(y)) => V::Entier(*x as i64 / y as i64),
            (_, rhs) => Err(RuntimeError::invalid_op(
                "++",
                self.get_type(),
                rhs.get_type(),
            ))?,
        })
    }

    pub fn pow(&self, rhs: Self) -> Result<Value, RuntimeError> {
        use Value as V;

        Ok(match (self, rhs) {
            (V::Entier(x), V::Entier(y)) => V::Entier(x.pow(y as u32)),
            (V::Decimal(x), V::Entier(y)) => V::Decimal(x.powi(y as i32)),
            (V::Entier(x), V::Decimal(y)) => V::Decimal((*x as f64).powf(y)),
            (V::Decimal(x), V::Decimal(y)) => V::Decimal(x.powf(y)),
            (_, rhs) => Err(RuntimeError::invalid_op(
                "++",
                self.get_type(),
                rhs.get_type(),
            ))?,
        })
    }

    pub fn extend(&self, rhs: Self) -> Result<Value, RuntimeError> {
        use Value as V;

        match (self, rhs) {
            (Value::Texte(s), rhs @ Value::Texte(..)) => self.clone() + rhs,
            (Value::Liste(l), Value::Liste(l2)) => {
                let mut l3 = l.read().unwrap().clone();
                l3.extend(l2.read().unwrap().to_owned());
                Ok(Value::Liste(Arc::new(RwLock::new(l3))))
            }

            // (ASDict(d), ASDict(d2)) => {
            //     let mut d3 = d.borrow().clone();+
            //     for e in d2.borrow().items() {
            //         d3.insert(e.key().to_owned(), e.val().to_owned());
            //     }
            //     Ok(ASObj::dict(d3))
            // }
            //
            // (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            // (ASClasse(classe), _) => todo!("Check présense du field?"),
            // (ASModule { name, alias, env }, _) => todo!(),
            (_, rhs) => Err(RuntimeError::invalid_op(
                "++",
                self.get_type(),
                rhs.get_type(),
            )),
        }
    }

    pub fn contains(&self, rhs: &Self) -> Result<bool, RuntimeError> {
        use Value as V;

        match (self, rhs) {
            (Value::Texte(s), Value::Texte(sub_s)) => Ok(s.contains(sub_s)),
            (Value::Liste(l), rhs) => Ok(l.read().unwrap().contains(rhs)),
            // (ASDict(d), rhs) => Ok(d.borrow().contains(rhs)),

            // (ASTuple(_), _) => todo!("Tuple pas encore (et peut-être jamais) dans le langage"),
            // (ASClasse(classe), _) => todo!("Check présense du field?"),
            // (ASModule { name, alias, env }, _) => todo!(),
            _ => Err(RuntimeError::invalid_op(
                "dans",
                self.get_type(),
                rhs.get_type(),
            )),
        }
    }

    pub fn as_entier(&self) -> Option<i64> {
        match &self {
            Value::Entier(i) => Some(*i as i64),
            _ => None,
        }
    }

    pub fn as_decimal(&self) -> Option<f64> {
        match &self {
            Value::Entier(i) => Some(*i as f64),
            Value::Decimal(d) => Some(*d as f64),
            _ => None,
        }
    }

    pub fn as_texte(&self) -> Option<&str> {
        match &self {
            Value::Texte(s) => Some(&s),
            _ => None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_str = match self {
            Value::Entier(i) => i.to_string(),
            Value::Decimal(d) => d.to_string(),
            Value::Texte(s) => s.clone(),
            Value::Booleen(b) => if *b { "vrai" } else { "faux" }.into(),
            Value::Nul => "nul".into(),
            Value::TypeObj(t) => t.to_string(),
            Value::Structure(s) => format!("{:?}", s),
            Value::Liste(vals) => format!(
                "[{}]",
                vals.read()
                    .unwrap()
                    .iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Closure(closure) => format!(
                "fonction {}()",
                closure
                    .function
                    .name
                    .as_ref()
                    .unwrap_or(&"anonyme".to_string())
            ),
            Value::NativeFunction(native_function) => {
                format!("fonction native {}()", native_function.name)
            }
        };

        write!(f, "{}", to_str)
    }
}

#[derive(Debug, Clone)]
pub enum UpvalueLocation {
    Open(usize),                // index into VM.stack
    Closed(Arc<RwLock<Value>>), // heap cell
}

impl PartialEq for UpvalueLocation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Open(l0), Self::Open(r0)) => l0 == r0,
            (Self::Closed(l0), Self::Closed(r0)) => Arc::ptr_eq(l0, r0),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Upvalue {
    pub location: UpvalueLocation,
}

impl Upvalue {
    pub fn get(&self, vm: &VM) -> Value {
        match &self.location {
            UpvalueLocation::Open(idx) => vm.stack[*idx].clone(),
            UpvalueLocation::Closed(cell) => cell.read().unwrap().clone(),
        }
    }

    pub fn set(&mut self, vm: &mut VM, v: Value) {
        match &mut self.location {
            UpvalueLocation::Open(idx) => vm.stack[*idx] = v,
            UpvalueLocation::Closed(cell) => *cell.write().unwrap() = v,
        }
    }

    pub fn close(&mut self, vm: &VM) {
        if let UpvalueLocation::Open(idx) = self.location {
            let v = vm.stack[idx].clone();
            self.location = UpvalueLocation::Closed(Arc::new(RwLock::new(v)));
        }
    }

    pub fn to_closed(&self, vm: &VM) -> Upvalue {
        if let UpvalueLocation::Open(idx) = self.location {
            let v = vm.stack[idx].clone();
            Upvalue {
                location: UpvalueLocation::Closed(Arc::new(RwLock::new(v))),
            }
        } else {
            Upvalue {
                location: self.location.clone(),
            }
        }
    }
}

#[derive(Debug)]
pub struct CallFrame {
    pub closure: Arc<Closure>,
    pub ip: usize,
    pub base: usize, // where this frame's locals start in VM.stack
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpvalueSpec {
    Local(usize),   // refers to local slot index in the parent function
    Upvalue(usize), // refers to parent's upvalue number (chain)
}

impl UpvalueSpec {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local(..))
    }

    pub fn index(&self) -> usize {
        match self {
            UpvalueSpec::Local(index) => *index,
            UpvalueSpec::Upvalue(index) => *index,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Entier(i), Value::Decimal(d)) | (Value::Decimal(d), Value::Entier(i)) => {
                *d == *i as f64
            }
            (Value::Entier(i1), Value::Entier(i2)) => i1 == i2,
            (Value::Texte(t1), Value::Texte(t2)) => t1 == t2,
            (Value::Booleen(b1), Value::Booleen(b2)) => b1 == b2,
            (Value::Liste(l1), Value::Liste(l2)) => {
                l1.read().unwrap().as_ref() as &Vec<Value>
                    == l2.read().unwrap().as_ref() as &Vec<Value>
            }
            // (ASDict(d1), ASDict(d2)) => d1 == d2,
            // (ASFonc(f1), ASFonc(f2)) => f1 == f2,
            // (ASClasse(classe1), ASClasse(classe2)) => classe1 == classe2,
            // (ASClasseInst(inst1), ASClasseInst(inst2)) => inst1 == inst2,
            (Value::Nul, Value::Nul) => true,
            _ => false,
        }
    }
}

impl Add for Value {
    type Output = Result<Value, RuntimeError>;

    fn add(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Liste(l), any) => Value::Liste({
                let mut l = l.read().unwrap().clone();
                l.push(any);
                Arc::new(RwLock::new(l))
            }),
            (Value::Texte(s), any) => Value::Texte(format!("{}{}", s, any.to_string())),
            (any, Value::Texte(s)) => Value::Texte(format!("{}{}", any.to_string(), s)),
            (Value::Entier(x), Value::Entier(y)) => Value::Entier(x + y),
            (Value::Decimal(x), Value::Entier(y)) => Value::Decimal(x + y as f64).into(),
            (Value::Entier(x), Value::Decimal(y)) => Value::Decimal(x as f64 + y).into(),
            (Value::Decimal(x), Value::Decimal(y)) => Value::Decimal(x + y).into(),
            (l, r) => Err(RuntimeError::invalid_op("+", l.get_type(), r.get_type()))?,
        })
    }
}

impl Sub for Value {
    type Output = Result<Value, RuntimeError>;

    fn sub(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Texte(s), Value::Texte(s2)) => Value::Texte(s.replace(s2.as_str(), "")),
            (Value::Entier(x), Value::Entier(y)) => Value::Entier(x - y),
            (Value::Decimal(x), Value::Entier(y)) => Value::Decimal(x - y as f64),
            (Value::Entier(x), Value::Decimal(y)) => Value::Decimal(x as f64 - y),
            (Value::Decimal(x), Value::Decimal(y)) => Value::Decimal(x - y),
            (l, r) => Err(RuntimeError::invalid_op("-", l.get_type(), r.get_type()))?,
        })
    }
}

impl Mul for Value {
    type Output = Result<Value, RuntimeError>;

    fn mul(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Texte(s), Value::Entier(n)) => {
                Value::Texte(s.repeat(if n >= 0 { n as usize } else { 0 }))
            }
            (Value::Entier(x), Value::Entier(y)) => Value::Entier(x * y),
            (Value::Decimal(x), Value::Entier(y)) => Value::Decimal(x * y as f64),
            (Value::Entier(x), Value::Decimal(y)) => Value::Decimal(x as f64 * y),
            (Value::Decimal(x), Value::Decimal(y)) => Value::Decimal(x * y),
            (Value::Liste(l), Value::Entier(n)) => Value::Liste(if n <= 0 {
                Arc::new(RwLock::new(vec![]))
            } else {
                let n = n as usize;
                let l = l.read().unwrap();
                let len = l.len();
                let mut new_vec = Vec::with_capacity(n * len);
                for i in 0..n * len {
                    new_vec.push(l[i % len].clone());
                }
                Arc::new(RwLock::new(new_vec))
            }),
            (l, r) => Err(RuntimeError::invalid_op("*", l.get_type(), r.get_type()))?,
        })
    }
}

impl Div for Value {
    type Output = Result<Value, RuntimeError>;

    fn div(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Value::Decimal(x as f64 / y as f64),
            (Value::Decimal(x), Value::Entier(y)) => Value::Decimal(x / y as f64),
            (Value::Entier(x), Value::Decimal(y)) => Value::Decimal(x as f64 / y),
            (Value::Decimal(x), Value::Decimal(y)) => Value::Decimal(x / y),
            (l, r) => Err(RuntimeError::invalid_op("*", l.get_type(), r.get_type()))?,
        })
    }
}

impl Rem for Value {
    type Output = Result<Value, RuntimeError>;

    fn rem(self, rhs: Self) -> Self::Output {
        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        Ok(match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Value::Entier((x % y + y) % y),
            (Value::Decimal(x), Value::Entier(y)) => {
                Value::Decimal((x % y as f64 + y as f64) % y as f64)
            }
            (Value::Entier(x), Value::Decimal(y)) => Value::Decimal((x as f64 % y + y) % y),
            (Value::Decimal(x), Value::Decimal(y)) => Value::Decimal((x % y + y) % y),
            _ => Err(RuntimeError::invalid_op("% (modulo)", type_1, type_2))?,
        })
    }
}

impl BitXor for Value {
    type Output = Result<Value, RuntimeError>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Ok(Value::Entier(x ^ y)),
            (Value::Booleen(x), Value::Booleen(y)) => Ok(Value::Booleen(x ^ y)),
            _ => Err(RuntimeError::invalid_op("xor", type_1, type_2))?,
        }
    }
}

impl BitAnd for Value {
    type Output = Result<Value, RuntimeError>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Ok(Value::Entier(x & y)),
            (Value::Booleen(x), Value::Booleen(y)) => Ok(Value::Booleen(x & y)),
            _ => Err(RuntimeError::invalid_op("&", type_1, type_2))?,
        }
    }
}

impl BitOr for Value {
    type Output = Result<Value, RuntimeError>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Ok(Value::Entier(x | y).into()),
            (Value::Booleen(x), Value::Booleen(y)) => Ok(Value::Booleen(x | y).into()),
            _ => Err(RuntimeError::invalid_op("|", type_1, type_2))?,
        }
    }
}

impl Shl for Value {
    type Output = Result<Value, RuntimeError>;

    fn shl(self, rhs: Self) -> Self::Output {
        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Ok(Value::Entier(x << y)),
            _ => Err(RuntimeError::invalid_op("<<", type_1, type_2))?,
        }
    }
}

impl Shr for Value {
    type Output = Result<Value, RuntimeError>;

    fn shr(self, rhs: Self) -> Self::Output {
        let type_1 = self.get_type().clone();
        let type_2 = rhs.get_type().clone();

        match (self, rhs) {
            (Value::Entier(x), Value::Entier(y)) => Ok(Value::Entier(x >> y)),
            _ => Err(RuntimeError::invalid_op(">>", type_1, type_2))?,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Value as V;

        let (x, y) = match (self, other) {
            (V::Entier(x), V::Entier(y)) => (*x as f64, *y as f64),
            (V::Decimal(x), V::Entier(y)) => (*x, *y as f64),
            (V::Entier(x), V::Decimal(y)) => (*x as f64, *y),
            (V::Decimal(x), V::Decimal(y)) => (*x, *y),
            _ => None?,
        };

        x.partial_cmp(&y)
    }
}
