use std::rc::Rc;

use crate::{
    as_cast, as_fonction, as_mod,
    as_obj::{ASObj, ASType},
    ast::Expr,
    visitor::Visitable,
};

#[derive(PartialOrd, PartialEq, Debug)]
struct SortableASObj(ASObj);

impl Ord for SortableASObj {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("Not comparable")
    }
}

impl Eq for SortableASObj {
}

as_mod!(
    LISTE_MOD,
    as_fonction! {
        trier[runner](lst: ASType::Liste, clef: ASType::Fonction => ASObj::ASNul) -> ASType::Liste; {
            let env = runner.get_env();
            as_cast!(ASObj::ASListe(lst) = lst);
            let mut lst = lst.as_ref().clone();
            Ok(Some(match env.get_value(&"clef".into()).unwrap() {
                ASObj::ASNul => {
                    lst.get_mut()
                        .sort_by(|a, b| a.partial_cmp(b).expect("Comparable"));
                    ASObj::ASListe(Rc::new(lst))
                }
                clef @ ASObj::ASFonc { .. } => {
                    let clef = Expr::literal(clef.clone());
                    lst.get_mut()
                        .sort_by_cached_key(|el| {
                            let to_call = Expr::FnCall {
                                func: clef.clone(),
                                args: vec![Expr::literal(el.clone())],
                            };
                            to_call.accept(runner);
                            SortableASObj(runner.pop_value().unwrap())
                        });
                    ASObj::ASListe(Rc::new(lst))
                }
                _ => unreachable!(),
            }))
        }
    },
);
