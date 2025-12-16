use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::as_obj::ASErreurType;
use crate::compiler::bytecode::instructions_to_string;
use crate::compiler::err::CompilationError;
use crate::compiler::obj::{ArcClosure, ArcUpvalue, Function, UpvalueSpec, Value};
use crate::runtime::err::RuntimeError;
use crate::runtime::vm::VM;

pub type ArcStructure = Arc<RwLock<ASStructure>>;
pub type ArcObjet = Arc<RwLock<ASObjet>>;
pub type ArcClosureMethod = Arc<RwLock<ClosureMethod>>;

#[derive(Debug)]
pub struct ASStructure {
    pub name: String,

    pub fields: Vec<ASFieldInfo>,
    pub inst_methods: HashMap<String, ArcClosure>,
    pub struct_methods: HashMap<String, ArcClosure>,
}

impl ASStructure {
    pub fn new(name: String, fields: Vec<ASFieldInfo>) -> Self {
        Self {
            name,
            fields,
            inst_methods: HashMap::new(),
            struct_methods: HashMap::new(),
        }
    }
}

impl Display for ASStructure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "structure {} {{ {} }}",
            self.name,
            self.fields
                .iter()
                .map(|field| format!("{}: {}", field.name, field.field_type.name))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub struct ASFieldInfo {
    pub name: String,
    pub field_type: TypeSpec,
    pub is_const: bool,
    pub is_private: bool,
    pub value: Option<Value>,
}

#[derive(Debug)]
pub struct ClosureMethod {
    pub closure: ArcClosure,
    pub inst_value: Value,
}

#[derive(Debug)]
pub struct ASObjet {
    pub structure: ArcStructure,
    pub fields: HashMap<String, Value>,
}

impl ASObjet {
    pub fn get_field(obj: Arc<RwLock<Self>>, attr: &str) -> Result<Value, RuntimeError> {
        // its a field
        if let Some(val) = obj.read().unwrap().fields.get(attr) {
            return Ok(val.clone());
        }

        let obj_val = obj.read().unwrap();
        // its a method
        let structure = obj_val.structure.read().unwrap();
        if let Some(method) = structure.inst_methods.get(attr) {
            return Ok(Value::Function(Function::ClosureMethod(Arc::new(
                RwLock::new(ClosureMethod {
                    closure: Arc::clone(method),
                    inst_value: Value::Objet(Arc::clone(&obj)),
                }),
            ))));
        }

        Err(RuntimeError::invalid_field(&obj_val.to_string(), attr))
    }
}

impl Display for ASObjet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fields_to_string = self.fields.iter().collect::<Vec<_>>();

        let structure_names = self
            .structure
            .read()
            .unwrap()
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect::<HashMap<_, _>>();

        fields_to_string.sort_by_key(|(name, _)| structure_names[*name]);

        write!(
            f,
            "{} {{\n  {}\n}}",
            self.structure.read().unwrap().name,
            fields_to_string
                .into_iter()
                .map(|(k, v)| format!("{}: {}", k, v.repr()))
                .collect::<Vec<_>>()
                .join(",\n  ")
        )
    }
}

impl ASObjet {
    pub fn new(structure: ArcStructure, fields: HashMap<String, Value>) -> Self {
        Self { structure, fields }
    }
}

#[derive(Clone, PartialEq)]
pub struct ASFunction {
    pub name: Option<String>,
    pub code: Vec<u16>,        // bytecode
    pub constants: Vec<Value>, // constant pool
    pub upvalue_count: usize,
    pub upvalue_specs: Vec<UpvalueSpec>, // from compiler: local? index?

    pub nb_params: usize,
}

impl ASFunction {
    pub fn new(name: Option<String>, nb_params: usize) -> Self {
        Self {
            name,
            code: vec![],
            constants: vec![],
            upvalue_count: 0,
            upvalue_specs: vec![],
            nb_params: nb_params,
        }
    }

    pub fn new_anonymous(nb_params: usize) -> Self {
        Self {
            name: None,
            code: vec![],
            constants: vec![],
            upvalue_count: 0,
            upvalue_specs: vec![],

            nb_params,
        }
    }
}

impl Debug for ASFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("code", &instructions_to_string(&self.code))
            .field("constants", &self.constants)
            .field("upvalue_count", &self.upvalue_count)
            .field("upvalue_specs", &self.upvalue_specs)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub function: Arc<ASFunction>,
    pub upvalues: Vec<ArcUpvalue>,
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
            && self
                .upvalues
                .iter()
                .zip(other.upvalues.iter())
                .all(|(u1, u2)| Arc::ptr_eq(&u1, &u2))
    }
}
pub struct NativeFunction {
    pub func: Arc<dyn Fn(&mut VM, Vec<Value>) -> Result<Option<Value>, RuntimeError>>,
    pub name: Arc<String>,
    pub desc: Arc<Option<String>>,
}

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
            desc: Arc::clone(&self.desc),
            name: Arc::clone(&self.name),
        }
    }
}
impl PartialEq for NativeFunction {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fonction native {}()", self.name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeSpec {
    pub name: String,
    pub value: TypeSpecValue,
}
impl TypeSpec {
    pub fn new_computed(
        name: String,
        args: Vec<ArgParam>,
        value: Rc<dyn FnMut(Vec<TypeSpec>) -> BaseType>,
    ) -> Self {
        Self {
            name,
            value: TypeSpecValue::Computed {
                type_params: args,
                compute_type: value,
            },
        }
    }

    pub fn new_simple(name: String, value: BaseType) -> Self {
        Self {
            name,
            value: TypeSpecValue::BaseType(value),
        }
    }

    pub fn as_base_type(self) -> Result<BaseType, CompilationError> {
        match self.value {
            TypeSpecValue::Computed {
                type_params,
                compute_type,
            } => Err(CompilationError::generic_error("Expected base type")),
            TypeSpecValue::BaseType(base_type) => Ok(base_type),
        }
    }

    pub fn compute(self, args: Vec<TypeSpec>) -> Result<BaseType, CompilationError> {
        match self.value {
            TypeSpecValue::Computed {
                type_params,
                compute_type,
            } => todo!(),
            TypeSpecValue::BaseType(base_type) => base_type.with_type_args(
                args.into_iter()
                    .map(|arg| arg.as_base_type())
                    .collect::<Result<_, _>>()?,
            ),
        }
    }
}

impl From<BaseType> for TypeSpec {
    fn from(value: BaseType) -> Self {
        Self::new_simple(value.to_string(), value.into())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArgParam {
    name: String,
    default_value: Option<TypeSpec>,
}

#[derive(Clone)]
pub enum TypeSpecValue {
    Computed {
        type_params: Vec<ArgParam>,
        compute_type: Rc<dyn FnMut(Vec<TypeSpec>) -> BaseType>,
    },
    BaseType(BaseType),
}
impl PartialEq for TypeSpecValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Computed {
                    type_params: l_type_params,
                    compute_type: l_compute_type,
                },
                Self::Computed {
                    type_params: r_type_params,
                    compute_type: r_compute_type,
                },
            ) => l_type_params == r_type_params,
            (Self::BaseType(l0), Self::BaseType(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl Debug for TypeSpecValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Computed {
                type_params,
                compute_type,
            } => f
                .debug_struct("Computed")
                .field("type_params", type_params)
                .finish(),
            Self::BaseType(arg0) => f.debug_tuple("BaseType").field(arg0).finish(),
        }
    }
}

impl From<BaseType> for TypeSpecValue {
    fn from(value: BaseType) -> Self {
        Self::BaseType(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BaseType {
    /// Type englobant tous les autres types, sauf [`ASType::Rien`] et [`ASType::Nul`]
    Tout,
    /// Type de retour d'une fonction qui ne retourne rien.
    /// Peut seulement être placé sur un retour de fonction.
    Rien,

    /// Type représentant l'absence d'une valeur.
    Nul,

    Entier,
    Decimal,
    Booleen,
    Texte,

    Liste(Box<BaseType>),

    Fonction,

    Module(String),
    Objet(String),
    Struct(StructType),

    Union(Vec<BaseType>),
    Array(Vec<BaseType>),
    Optional(Box<BaseType>),

    Type,
}
#[derive(Debug, PartialEq, Clone)]
pub struct StructType {
    pub name: String,
    pub fields: HashMap<String, TypeSpec>,
}

impl StructType {
    pub fn new(name: String, fields: HashMap<String, TypeSpec>) -> Self {
        Self { name, fields }
    }
}

impl BaseType {
    pub fn liste_tout() -> Self {
        Self::Liste(Box::new(BaseType::Tout))
    }

    pub fn with_type_args(&self, args: Vec<BaseType>) -> Result<BaseType, CompilationError> {
        match self {
            BaseType::Liste(astype) => Ok(BaseType::Liste(Box::new(Self::union(args)))),
            BaseType::Objet(_) => todo!(),
            _ => Err(CompilationError::generic_error(format!(
                "Le type {} ne prend pas d'arguments",
                self
            ))),
        }
    }
}

impl BaseType {
    pub fn nombre() -> BaseType {
        BaseType::Union(vec![Self::Entier, Self::Decimal])
    }

    pub fn iterable() -> BaseType {
        BaseType::Union(vec![Self::Liste(Box::new(Self::Tout)), Self::Texte])
    }

    pub fn iterable_ordonne() -> BaseType {
        BaseType::Union(vec![Self::Liste(Box::new(Self::Tout)), Self::Texte])
    }

    pub fn union(types: Vec<BaseType>) -> BaseType {
        if types.len() == 1 {
            types.into_iter().nth(0).unwrap()
        } else {
            let mut types: Vec<BaseType> = types
                .into_iter()
                .flat_map(|t| match t {
                    BaseType::Union(as_types) => as_types,
                    t => vec![t],
                })
                .collect();

            types.sort_by_key(|t| t.to_string());

            types.dedup();

            BaseType::Union(types)
        }
    }

    pub fn union_of(type1: BaseType, type2: BaseType) -> BaseType {
        BaseType::union(vec![type1, type2])
    }

    pub fn optional(type1: BaseType) -> BaseType {
        BaseType::Optional(Box::new(type1))
    }

    pub fn any() -> BaseType {
        BaseType::optional(BaseType::Tout)
    }

    pub fn is_tout(&self) -> bool {
        use BaseType::*;

        match self {
            Union(types) => types.contains(&BaseType::Tout),
            Optional(t) => t.is_tout(),
            _ => self == &BaseType::Tout,
        }
    }

    pub fn is_primitif(&self) -> bool {
        use BaseType::*;
        matches!(self, Entier | Decimal | Texte | Booleen | Nul | Optional(_))
    }

    pub fn type_match(type1: &BaseType, type2: &BaseType) -> bool {
        use BaseType::*;

        match (type1, type2) {
            (t1, t2) if t1 == t2 => true,

            (Tout, other) | (other, Tout) => other != &Rien && other != &Nul,

            // (Lit(o1), Lit(o2)) => o1 == o2,
            (Optional(t), other) | (other, Optional(t)) => {
                other == &Nul || BaseType::type_match(t.as_ref(), other)
            }

            (Union(types), other) | (other, Union(types)) => {
                types.iter().any(|t| BaseType::type_match(t, &other))
            }

            (Liste(t1), Liste(t2)) => Self::type_match(t1, t2),

            (Liste(t), Array(arr)) | (Array(arr), Liste(t)) => {
                arr.iter().all(|el| Self::type_match(el, t))
            }

            (Array(types1), Array(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                return types1
                    .iter()
                    .zip(types2)
                    .all(|(t1, t2)| BaseType::type_match(t1, t2));
            }

            (Decimal, Entier) => true,
            _ => false,
        }
    }
}

impl From<Option<BaseType>> for BaseType {
    fn from(value: Option<BaseType>) -> Self {
        value.unwrap_or(BaseType::any())
    }
}

impl FromStr for BaseType {
    type Err = ASErreurType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "entier" => Ok(Self::Entier),
            "decimal" | "décimal" => Ok(Self::Decimal),
            "booleen" | "booléen" => Ok(Self::Booleen),
            "nombre" => Ok(Self::nombre()),
            "iterable" | "itérable" => Ok(Self::iterable()),
            "texte" => Ok(Self::Texte),
            "liste" => Ok(Self::Liste(Box::new(BaseType::Tout))),
            "rien" => Ok(Self::Nul),
            "nul" => Ok(Self::Nul),
            "tout" => Ok(Self::Tout),
            "fonction" => Ok(Self::Fonction),
            // "module" => Ok(Self::Module),
            other => Ok(Self::Objet(other.into())),
        }
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BaseType as B;

        let to_string = match self {
            B::Tout => "tout".into(),
            B::Rien => "rien".into(),
            B::Nul => "nul".into(),
            B::Optional(t) => format!("{}?", t),
            B::Entier => "entier".into(),
            B::Decimal => "décimal".into(),
            B::Booleen => "booléen".into(),
            B::Texte => "texte".into(),
            B::Liste(t) => format!("liste<{}>", t),
            B::Fonction => "fonction".into(),
            B::Union(types) => types
                .iter()
                .map(Self::to_string)
                .collect::<Vec<String>>()
                .join(" | "),
            B::Array(types) => format!(
                "[{}]",
                types
                    .iter()
                    .map(Self::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            B::Type => "type".into(),
            B::Module(name) => name.clone(),
            B::Objet(s) => s.clone(),
            B::Struct(struct_type) => struct_type.name.clone(),
        };
        write!(f, "{}", to_string)
    }
}
