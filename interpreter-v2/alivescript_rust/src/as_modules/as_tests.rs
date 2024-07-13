use crate::{as_fonction, as_mod, as_obj::ASType};

use crate::as_obj::ASErreurType;
use crate::as_obj::ASObj::*;
use crate::as_obj::ASType::*;

as_mod!(
    TEST_MOD,
    as_fonction! {
        affirmer(test: Booleen) -> Rien; {
            if test.to_bool() {
                Ok(None)
            } else {
                Err(ASErreurType::new_erreur_affirmation("".into(), ASBooleen(true), ASBooleen(false)))
            }
        }
    },

    as_fonction! {
        affirmerFaux(test: Booleen) -> Rien; {
            if !test.to_bool() {
                Ok(None)
            } else {
                Err(ASErreurType::new_erreur_affirmation("".into(), ASBooleen(false), ASBooleen(true)))
            }
        }
    },

    as_fonction! {
        affirmerEgal(obj1: ASType::any(), obj2: ASType::any()) -> ASType::Rien; {
            if obj1 == obj2 {
                Ok(None)
            } else {
                Err(ASErreurType::new_erreur_affirmation(format!("{} == {}", obj1, obj2), ASBooleen(true), ASBooleen(false)))
            }
        }
    },

    as_fonction! {
        affirmerPasEgal(obj1: ASType::any(), obj2: ASType::any()) -> ASType::Rien; {
            if obj1 != obj2 {
                Ok(None)
            } else {
                Err(ASErreurType::new_erreur_affirmation(format!("{} != {}", obj1, obj2), ASBooleen(false), ASBooleen(true)))
            }
        }
    },

    as_fonction! {
        affirmerEgalType(obj1: ASType::any(), obj2: ASType::any()) -> ASType::Rien; {
            if obj1.get_type() == obj2.get_type() {
                Ok(None)
            } else {
                Err(ASErreurType::new_erreur_affirmation(format!("{} == {}", obj1.get_type(), obj2.get_type()), ASBooleen(true), ASBooleen(false)))
            }
        }
    },
);
