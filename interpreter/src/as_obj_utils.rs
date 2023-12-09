use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type ObjPtr = usize;
pub type Label = usize;
pub type Seen = bool;

pub trait RecursiveRepr {
    fn recursive_repr(
        &self,
        seen_map: Option<Rc<RefCell<HashMap<ObjPtr, (Label, Seen)>>>>,
    ) -> String;
}
