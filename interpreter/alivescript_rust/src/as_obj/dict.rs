use std::{cell::RefCell, collections::HashMap, rc::Rc};

use derive_new::new;

use crate::{
    as_obj::ASObj,
    as_obj_utils::{Label, ObjPtr, RecursiveRepr, Seen},
};

#[derive(Debug, Clone, new, PartialEq, Default)]
pub struct ASDict(Vec<ASPaire>);

impl ASDict {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &ASObj) -> bool {
        self.0.iter().any(|pair| pair.key() == key)
    }

    pub fn get(&self, key: &ASObj) -> Option<&ASPaire> {
        self.0.iter().find(|pair| pair.key() == key)
    }

    pub fn get_val(&self, key: &ASObj) -> Option<&ASObj> {
        self.get(key).map(|pair| pair.val())
    }

    pub fn get_mut(&mut self, key: &ASObj) -> Option<&mut ASPaire> {
        self.0.iter_mut().find(|pair| pair.key() == key)
    }

    pub fn insert(&mut self, key: ASObj, val: ASObj) {
        if let Some(pair) = self.get_mut(&key) {
            pair.set_val(Box::new(val));
            return;
        }
        self.0.push(ASPaire::new(Box::new(key), Box::new(val)));
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn items(&self) -> impl Iterator<Item = &ASPaire> {
        self.0.iter()
    }
}

impl RecursiveRepr for ASDict {
    fn recursive_repr(
        &self,
        seen_map: Option<Rc<RefCell<HashMap<ObjPtr, (Label, Seen)>>>>,
    ) -> String {
        let seen_map = seen_map.unwrap_or_else(|| Rc::new(RefCell::new(HashMap::new())));

        let d = &self.0;
        let hash = d.as_ptr() as usize;
        let maybe_label = {
            let seen_map_borrow = seen_map.borrow();
            seen_map_borrow.get(&hash).map(|(label, seen)| *label)
        };
        if let Some(label) = maybe_label {
            seen_map.borrow_mut().insert(hash, (label, true));
            return format!("{{<{}>}}", label);
        }

        let label = seen_map.borrow().len() + 1;

        {
            let mut seen_t = seen_map.borrow_mut();
            seen_t.insert(hash, (label, false));
        }

        let res = d
            .iter()
            .map(|el| el.recursive_repr(Some(Rc::clone(&seen_map))))
            .collect::<Vec<_>>();

        let seen = seen_map.borrow()[&hash].1;

        format!(
            "{}{{{}}}",
            if seen {
                format!("<{}>@", label)
            } else {
                "".into()
            },
            res.join(", ")
        )
    }
}

#[derive(Debug, Clone, new, PartialEq)]
pub struct ASPaire {
    key: Box<ASObj>,
    val: Box<ASObj>,
}
impl ASPaire {
    pub fn set_val(&mut self, val: Box<ASObj>) {
        self.val = val;
    }

    pub fn key(&self) -> &ASObj {
        self.key.as_ref()
    }

    pub fn val(&self) -> &ASObj {
        self.val.as_ref()
    }
}
impl RecursiveRepr for ASPaire {
    fn recursive_repr(
        &self,
        seen_map: Option<Rc<RefCell<HashMap<ObjPtr, (Label, Seen)>>>>,
    ) -> String {
        let seen_map = seen_map.unwrap_or_else(|| Rc::new(RefCell::new(HashMap::new())));

        format!(
            "{}: {}",
            self.key.recursive_repr(Some(Rc::clone(&seen_map))),
            self.val.recursive_repr(Some(Rc::clone(&seen_map)))
        )
    }
}
