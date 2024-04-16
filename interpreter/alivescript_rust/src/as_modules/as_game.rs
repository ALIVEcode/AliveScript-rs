
use crate::{
    as_fonction, as_mod,
    as_obj::{ASType, ASType::*},
};

as_mod! {
    GAME_MOD,
    as_fonction! {
        abc(obj: ASType::any(), options: Dict) -> Dict; {
            Ok(None)
        }
    }
}
