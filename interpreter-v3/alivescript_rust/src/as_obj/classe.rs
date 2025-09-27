use std::{cell::RefCell, collections::HashMap, fmt::Display, ptr, rc::Rc};

use derive_getters::Getters;
use derive_new::new;

use crate::{
    as_obj::{ASFonc, ASObj, ASScope, ASType},
    as_obj_utils::{Label, ObjPtr, RecursiveRepr, Seen},
    ast::Expr,
};

#[derive(Debug, new, Getters, PartialEq)]
pub struct ASClasse {
    name: String,
    docs: Option<String>,
    fields: Vec<ASClasseField>,
    init: Option<Rc<ASFonc>>,
    methods: Vec<Rc<ASFonc>>,
    static_env: Rc<RefCell<ASScope>>,
}

impl Clone for ASClasse {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            docs: self.docs.clone(),
            fields: self.fields.clone(),
            init: self.init.as_ref().map(Rc::clone),
            methods: self.methods.clone(),
            static_env: Rc::clone(&self.static_env),
        }
    }
}

#[derive(Debug, Clone, new, Getters)]
pub struct ASClasseInst {
    classe_parent: Rc<ASClasse>,
    env: Rc<RefCell<ASScope>>,
}

impl ASClasseInst {
    pub fn get_type(&self) -> ASType {
        ASType::Objet(self.classe_parent.name().clone())
    }
}

impl RecursiveRepr for ASClasseInst {
    fn recursive_repr(
        &self,
        seen_map: Option<Rc<RefCell<HashMap<ObjPtr, (Label, Seen)>>>>,
    ) -> String {
        let seen_map = seen_map.unwrap_or_else(|| Rc::new(RefCell::new(HashMap::new())));

        let hash = self as *const ASClasseInst as usize;
        let maybe_label = {
            let seen_map_borrow = seen_map.borrow();
            seen_map_borrow.get(&hash).map(|(label, seen)| *label)
        };
        if let Some(label) = maybe_label {
            seen_map.borrow_mut().insert(hash, (label, true));
            return format!("{}@<{}>", self.classe_parent.name(), label);
        }

        let label = seen_map.borrow().len() + 1;
        {
            let mut seen_t = seen_map.borrow_mut();
            seen_t.insert(hash, (label, false));
        }

        let env = self.env.borrow();
        let fields = self
            .classe_parent
            .fields()
            .iter()
            .filter_map(|field| {
                if field.name().starts_with("_") {
                    return None;
                }
                let field_val = env.get_value(&field.name).unwrap();
                Some(format!(
                    "{}={}",
                    field.name,
                    field_val.recursive_repr(Some(Rc::clone(&seen_map)))
                ))
            })
            .collect::<Vec<String>>();

        let seen = seen_map.borrow()[&hash].1;
        format!(
            "{}{}({})",
            self.classe_parent.name(),
            if seen {
                format!("@<{}>", label)
            } else {
                "".into()
            },
            fields.join(", "),
        )
    }
}

impl PartialEq for ASClasseInst {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Into<ASObj> for &Rc<ASClasseInst> {
    fn into(self) -> ASObj {
        ASObj::ASClasseInst(Rc::clone(self))
    }
}

impl Display for ASClasseInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.recursive_repr(None))
    }
}

#[derive(Debug, Clone, new, Getters)]
pub struct ASMethode {
    func: Rc<ASFonc>,
    inst: Rc<ASClasseInst>,
}

#[derive(Debug, PartialEq, Clone, Getters, new)]
pub struct ASClasseField {
    pub name: String,
    pub vis: ASClasseFieldVis,
    pub static_type: ASType,
    pub default_value: Option<Box<Expr>>,
    pub is_const: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ASClasseFieldVis {
    Publique,
    Privee,
}
