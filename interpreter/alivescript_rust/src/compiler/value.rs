use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::compiler::Local;
use crate::compiler::bytecode::instructions_to_string;
use crate::compiler::err::CompilationErrorKind;
use crate::compiler::obj::{ArcUpvalue, Function, UpvalueSpec, Value};
use crate::runtime::err::RuntimeError;
use crate::runtime::vm::VM;

pub type ArcClosureProto = Arc<ClosureProto>;
pub type ArcClosureInst = Arc<ClosureInst>;
pub type ArcFunction = Arc<ASFunction>;
pub type ArcStructure = Arc<RwLock<ASStructure>>;
pub type ArcObjet = Arc<RwLock<ASObjet>>;
pub type ArcClosureMethod = Arc<RwLock<ClosureMethod>>;
pub type ArcModule = Arc<RwLock<ASModule>>;
pub type ArcNativeObjet = Arc<RwLock<dyn NativeObjet>>;

#[derive(Debug, Clone)]
pub struct ASField {
    pub value: Value,
    pub is_const: bool,
    pub field_type: Type,
}

impl ASField {
    pub fn new(is_const: bool, field_type: Type, value: Value) -> Self {
        Self {
            value,
            is_const,
            field_type,
        }
    }

    pub fn new_with_type_from_value(is_const: bool, value: Value) -> Self {
        Self {
            field_type: value.get_type(),
            value,
            is_const,
        }
    }
}

#[derive(Debug)]
pub struct ASModule {
    pub name: String,
    pub members: HashMap<String, ASField>,
}

impl ASModule {
    pub fn new(name: impl ToString, members: HashMap<String, ASField>) -> Self {
        Self {
            name: name.to_string(),
            members,
        }
    }

    pub fn from_iter<T: IntoIterator<Item = (String, ASField)>>(
        name: impl ToString,
        iter: T,
    ) -> ArcModule {
        ArcModule::new(RwLock::new(ASModule::new(name, HashMap::from_iter(iter))))
    }

    pub fn get_member(&self, name: &str) -> Result<Value, RuntimeError> {
        // its a field
        if let Some(val) = self.members.get(name) {
            return Ok(val.value.clone());
        }

        Err(RuntimeError::invalid_field(&self.name, name))
    }

    pub fn set_member(&mut self, name: &str, val: Value) -> Result<(), RuntimeError> {
        let Some(member) = self.members.get_mut(name) else {
            return Err(RuntimeError::invalid_field(&self.name, name));
        };

        if member.is_const {
            return Err(RuntimeError::assign_to_const(format!(
                "{}.{}",
                self.name, name
            )));
        }

        member.value = val;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FieldProto {
    pub value_idx: usize,
    pub is_const: bool,
    pub field_type: Type,
}

#[derive(Debug)]
pub struct ModuleProto {
    pub name: String,
    pub load_fn: ClosureProto,
    pub exported_members: HashMap<String, FieldProto>,
}

#[derive(Debug)]
pub struct ASStructure {
    pub name: String,

    pub fields: Vec<ASFieldInfo>,
    pub methods: HashMap<String, ArcClosureInst>,
}

impl ASStructure {
    pub fn new(name: String, fields: Vec<ASFieldInfo>) -> Self {
        Self {
            name,
            fields,
            methods: HashMap::new(),
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
    pub closure: ArcClosureInst,
    pub inst_value: Value,
}

#[derive(Debug)]
pub struct ASObjet {
    pub structure: ArcStructure,
    pub fields: HashMap<String, ASField>,
}

impl ASObjet {
    pub fn get_field(
        vm: &mut VM,
        obj: Arc<RwLock<Self>>,
        attr: &str,
    ) -> Result<Value, RuntimeError> {
        // its a field
        if let Some(val) = obj.read().unwrap().fields.get(attr) {
            return Ok(val.value.clone());
        }

        let obj_val = obj.read().unwrap();
        // its a method
        let structure = obj_val.structure.read().unwrap();
        if let Some(method) = structure.methods.get(attr) {
            // let closure = vm.resolve_proto_closure_upvalues(Arc::clone(method))?;
            return Ok(Value::Function(Function::ClosureMethod(Arc::new(
                RwLock::new(ClosureMethod {
                    closure: Arc::clone(method),
                    inst_value: Value::Objet(Arc::clone(&obj)),
                }),
            ))));
        }

        Err(RuntimeError::invalid_field(&obj_val.to_string(), attr))
    }

    pub fn set_field(
        vm: &mut VM,
        obj: Arc<RwLock<Self>>,
        attr: &str,
        val: Value,
    ) -> Result<(), RuntimeError> {
        let mut obj = obj.write().unwrap();

        let Some(member) = obj.fields.get_mut(attr) else {
            return Err(RuntimeError::invalid_field(
                &obj.structure.read().unwrap().name,
                attr,
            ));
        };

        if member.is_const {
            return Err(RuntimeError::assign_to_const_field(attr));
        }

        member.value = val;
        Ok(())
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
                .map(|(k, v)| format!("{}: {}", k, v.value.repr()))
                .collect::<Vec<_>>()
                .join(",\n  ")
        )
    }
}

impl ASObjet {
    pub fn new(structure: ArcStructure, fields: HashMap<String, ASField>) -> Self {
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

pub mod closure_state {
    #[derive(Debug, Clone)]
    pub struct Proto;
    #[derive(Debug, Clone)]
    pub struct Inst;
}

#[derive(Debug, Clone)]
pub struct Closure<T> {
    pub function: Arc<ASFunction>,
    pub upvalues: Vec<ArcUpvalue>,
    pub phantom: PhantomData<T>,
}

impl ClosureProto {
    pub fn new(function: Arc<ASFunction>) -> Self {
        Self {
            function,
            upvalues: vec![],
            phantom: PhantomData,
        }
    }
}

impl ClosureInst {
    pub fn new(function: Arc<ASFunction>, upvalues: Vec<ArcUpvalue>) -> Self {
        Self {
            function,
            upvalues,
            phantom: PhantomData,
        }
    }
}

pub type ClosureInst = Closure<closure_state::Inst>;
pub type ClosureProto = Closure<closure_state::Proto>;

impl<T> PartialEq for Closure<T> {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
            && self
                .upvalues
                .iter()
                .zip(other.upvalues.iter())
                .all(|(u1, u2)| Arc::ptr_eq(&u1, &u2))
    }
}

#[derive(Debug, Clone)]
pub struct NativeMethod {
    pub func: NativeFunction,
    pub inst_value: Box<Value>,
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
        value: Rc<dyn FnMut(Vec<TypeSpec>) -> Type>,
    ) -> Self {
        Self {
            name,
            value: TypeSpecValue::Computed {
                type_params: args,
                compute_type: value,
            },
        }
    }

    pub fn new_simple(name: String, value: Type) -> Self {
        Self {
            name,
            value: TypeSpecValue::BaseType(value),
        }
    }

    pub fn as_base_type(self) -> Result<Type, CompilationErrorKind> {
        match self.value {
            TypeSpecValue::Computed {
                type_params,
                compute_type,
            } => Err(CompilationErrorKind::generic_error("Expected base type")),
            TypeSpecValue::BaseType(base_type) => Ok(base_type),
        }
    }

    pub fn compute(self, args: Vec<TypeSpec>) -> Result<Type, CompilationErrorKind> {
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

impl From<Type> for TypeSpec {
    fn from(value: Type) -> Self {
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
        compute_type: Rc<dyn FnMut(Vec<TypeSpec>) -> Type>,
    },
    BaseType(Type),
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

impl From<Type> for TypeSpecValue {
    fn from(value: Type) -> Self {
        Self::BaseType(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
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

    Liste(Box<Type>),

    Fonction,

    Module(String),
    Objet(String),
    Struct(StructType),

    Union(Vec<Type>),
    Array(Vec<Type>),
    Optional(Box<Type>),

    Type,
}
#[derive(Debug, PartialEq, Clone)]
pub struct StructType {
    pub name: String,
    pub fields: HashMap<String, Type>,
}

impl StructType {
    pub fn new(name: String, fields: HashMap<String, Type>) -> Self {
        Self { name, fields }
    }
}

impl Type {
    pub fn liste_tout() -> Self {
        Self::Liste(Box::new(Type::Tout))
    }

    pub fn with_type_args(&self, args: Vec<Type>) -> Result<Type, CompilationErrorKind> {
        match self {
            Type::Liste(astype) => Ok(Type::Liste(Box::new(Self::union(args)))),
            Type::Objet(_) => todo!(),
            _ => Err(CompilationErrorKind::generic_error(format!(
                "Le type {} ne prend pas d'arguments",
                self
            ))),
        }
    }
}

impl Type {
    pub fn nombre() -> Type {
        Type::Union(vec![Self::Entier, Self::Decimal])
    }

    pub fn iterable() -> Type {
        Type::Union(vec![Self::Liste(Box::new(Self::Tout)), Self::Texte])
    }

    pub fn iterable_ordonne() -> Type {
        Type::Union(vec![Self::Liste(Box::new(Self::Tout)), Self::Texte])
    }

    pub fn union(types: Vec<Type>) -> Type {
        if types.len() == 1 {
            types.into_iter().nth(0).unwrap()
        } else {
            let mut types: Vec<Type> = types
                .into_iter()
                .flat_map(|t| match t {
                    Type::Union(as_types) => as_types,
                    t => vec![t],
                })
                .collect();

            types.sort_by_key(|t| t.to_string());

            types.dedup();

            Type::Union(types)
        }
    }

    pub fn union_of(type1: Type, type2: Type) -> Type {
        Type::union(vec![type1, type2])
    }

    pub fn optional(type1: Type) -> Type {
        Type::Optional(Box::new(type1))
    }

    pub fn tout() -> Type {
        Type::optional(Type::Tout)
    }

    pub fn is_tout(&self) -> bool {
        match self {
            Type::Union(types) => types.contains(&Type::Tout),
            Type::Optional(t) => t.is_tout(),
            _ => self == &Type::Tout,
        }
    }

    pub fn is_primitif(&self) -> bool {
        matches!(
            self,
            Type::Entier
                | Type::Decimal
                | Type::Texte
                | Type::Booleen
                | Type::Nul
                | Type::Optional(_)
        )
    }

    pub fn type_match(type1: &Type, type2: &Type) -> bool {
        match (type1, type2) {
            (t1, t2) if t1 == t2 => true,

            (Type::Tout, other) | (other, Type::Tout) => {
                other != &Type::Rien && other != &Type::Nul
            }

            // (Lit(o1), Lit(o2)) => o1 == o2,
            (Type::Optional(t), other) | (other, Type::Optional(t)) => {
                other == &Type::Nul || Type::type_match(t.as_ref(), other)
            }

            (Type::Union(types), other) | (other, Type::Union(types)) => {
                types.iter().any(|t| Type::type_match(t, &other))
            }

            (Type::Liste(t1), Type::Liste(t2)) => Self::type_match(t1, t2),

            (Type::Liste(t), Type::Array(arr)) | (Type::Array(arr), Type::Liste(t)) => {
                arr.iter().all(|el| Self::type_match(el, t))
            }

            (Type::Array(types1), Type::Array(types2)) => {
                if types1.len() != types2.len() {
                    return false;
                }
                return types1
                    .iter()
                    .zip(types2)
                    .all(|(t1, t2)| Type::type_match(t1, t2));
            }

            (Type::Decimal, Type::Entier) => true,
            _ => false,
        }
    }
}

impl From<Option<Type>> for Type {
    fn from(value: Option<Type>) -> Self {
        value.unwrap_or(Type::tout())
    }
}

impl FromStr for Type {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "entier" => Ok(Self::Entier),
            "decimal" | "décimal" => Ok(Self::Decimal),
            "booleen" | "booléen" => Ok(Self::Booleen),
            "nombre" => Ok(Self::nombre()),
            "iterable" | "itérable" => Ok(Self::iterable()),
            "texte" | "chaine" | "chaîne" => Ok(Self::Texte),
            "liste" => Ok(Self::Liste(Box::new(Type::Tout))),
            "rien" | "nul" | "vide" => Ok(Self::Nul),
            "tout" => Ok(Self::Tout),
            "fonction" => Ok(Self::Fonction),
            // "module" => Ok(Self::Module),
            other => Ok(Self::Objet(other.into())),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Type as B;

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
            B::Module(name) => format!("module '{}'", name.clone()),
            B::Objet(s) => s.clone(),
            B::Struct(struct_type) => struct_type.name.clone(),
        };
        write!(f, "{}", to_string)
    }
}

pub trait NativeObjet: Debug {
    fn type_name(&self) -> &'static str;

    fn get_member(&mut self, vm: &mut VM, name: &str) -> Result<Value, RuntimeError>;

    fn set_member(&mut self, vm: &mut VM, name: &str, val: Value) -> Result<(), RuntimeError> {
        Err(RuntimeError::generic_err(
            "Cet objet ne supporte pas l'affectation",
        ))
    }

    fn display(&self) -> String {
        format!("objet natif {}", self.type_name())
    }
}
