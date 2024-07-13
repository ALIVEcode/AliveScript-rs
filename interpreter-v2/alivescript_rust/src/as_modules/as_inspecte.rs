use crate::{
    as_fonction, as_mod,
    as_obj::{ASType, ASType::*},
};

as_mod! {
    INSPECTE_MOD,
    as_fonction! {
        estFonction(obj: ASType::any(), options: Dict) -> Dict; {
            Ok(None)
        }
    }
}
