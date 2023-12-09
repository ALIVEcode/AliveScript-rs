use std::rc::Rc;

use crate::{
    as_cast, as_fonction, as_mod,
    as_obj::{ASObj::*, ASType, ASType::*},
    ast::Expr,
    visitor::Visitable,
};

as_mod! {
    INSPECTE_MOD,
    as_fonction! {
        inspecte[runner](obj: ASType::any(), options: Dict) -> Dict; {
            Ok(None)
        }
    }
}
