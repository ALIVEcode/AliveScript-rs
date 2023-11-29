use unindent::Unindent;

use crate::{
    as_cast, as_fonction, as_mod,
    as_obj::{ASErreurType, ASFnParam, ASObj, ASType, ASVar},
    as_var,
};

as_mod! {
    BUILTIN_MOD,
    as_fonction! {
        typeDe(obj: ASType::any()) -> ASType::Texte; {
            Ok(Some(ASObj::ASTexte(obj.get_type().to_string())))
        }
    },
    as_fonction! {
        typeVar[runner](nomVar: ASType::Texte) -> ASType::Texte; {
            let env = runner.get_env();
            as_cast!(ASObj::ASTexte(nom_var) = nomVar);
            let maybe_var = env.get_var(nom_var).map(|v| &v.0);
            Ok(Some(match maybe_var {
                Some(var) => ASObj::ASTexte(var.get_type().to_string()),
                None => ASObj::ASNul,
            }))
        }
    },
    as_fonction! {
        tailleDe(obj: ASType::iterable()) -> ASType::Entier; {
            Ok(Some(ASObj::ASEntier(match obj {
                ASObj::ASTexte(t) => t.len(),
                ASObj::ASListe(l) => l.borrow().len(),
                ASObj::ASDict(d) => d.borrow().len(),
                _ => unreachable!()
            } as i64)))
        }
    },
    as_fonction! {
        booleen(obj: ASType::any() => ASObj::ASBooleen(true)) -> ASType::Booleen; {
            Ok(Some(ASObj::ASBooleen(obj.to_bool())))
        }
    },
    as_fonction! {
        texte(obj: ASType::any() => ASObj::ASTexte("".to_owned())) -> ASType::Texte; {
            Ok(Some(ASObj::ASTexte(obj.to_string())))
        }
    },
    ASVar::new_with_value(
        "entier",
        Some(ASType::Fonction),
        true,
        ASObj::native_fn(
            "entier",
            None,
            vec![
                ASFnParam::native(
                    "obj",
                    ASType::union_of(ASType::Decimal, ASType::Texte),
                    Some(ASObj::ASEntier(0)),
                ),
                ASFnParam::native("base", ASType::Entier, Some(ASObj::ASEntier(10))),
            ],
            |runner| {
                let env = runner.get_env();
                let obj = env.get_value(&"obj".into()).unwrap();
                let ASObj::ASEntier(base) = env.get_value(&"base".into()).unwrap() else {
                    unreachable!()
                };
                Ok(Some(match obj {
                    ASObj::ASEntier(_) => obj.clone(),
                    ASObj::ASDecimal(d) => ASObj::ASEntier(*d as i64),
                    ASObj::ASTexte(s) => {
                        ASObj::ASEntier(i64::from_str_radix(s, *base as u32).unwrap())
                    }
                    _ => unreachable!(),
                }))
            },
            ASType::Entier,
        ),
    ),
    ASVar::new_with_value(
        "decimal",
        Some(ASType::Fonction),
        true,
        ASObj::native_fn(
            "decimal",
            Some("Tente de convertir du texte en valeur décimal. En cas d'échec: la fonction produit une erreur."),
            vec![ASFnParam::native(
                "obj",
                ASType::union_of(ASType::Decimal, ASType::Texte),
                Some(ASObj::ASDecimal(0f64)),
            )],
            |runner| {
                let env = runner.get_env();
                let obj = env.get_value(&"obj".into()).unwrap();
                Ok(Some(match obj {
                    ASObj::ASEntier(i) => ASObj::ASDecimal(*i as f64),
                    ASObj::ASDecimal(_) => obj.clone(),
                    ASObj::ASTexte(s) => ASObj::ASDecimal(s.parse().unwrap()),
                    _ => unreachable!(),
                }))
            },
            ASType::Decimal,
        ),
    ),
    as_fonction! {
        info(f: ASType::Fonction) -> ASType::Texte; {
            let ASObj::ASFonc(f) = f else {unreachable!()};
            Ok(Some(ASObj::ASTexte(format!(
                "{}\n  {}",
                f.to_string(),
                f.docs()
                    .clone()
                    .map(|doc| doc.unindent().replace("\n", "\n  "))
                    .unwrap_or("<sans-documentation>".into()),
            ))))
        }
    },
    as_fonction! {
        get_attr(obj: ASType::any(), attr: ASType::Texte) -> ASType::any(); {
            as_cast!(ASObj::ASTexte(attr) = attr);

            match obj {
                ASObj::ASClasseInst(inst) => inst
                    .env()
                    .borrow()
                    .get_value(attr)
                    .map(|v| Some(v.clone()))
                    .ok_or_else(|| ASErreurType::new_erreur_access_propriete(obj.clone(), attr.clone())),
                _ => Err(ASErreurType::new_erreur_access_propriete(obj.clone(), attr.clone()))
            }
        }
    },
    as_fonction! {
        contient_attr(obj: ASType::any(), attr: ASType::Texte) -> ASType::Booleen; {
            as_cast!(ASObj::ASTexte(attr) = attr);

            match obj {
                ASObj::ASClasseInst(inst) => Ok(Some(ASObj::ASBooleen(inst.env().borrow().get_value(attr).is_some()))),
                _ => Err(ASErreurType::new_erreur_access_propriete(obj.clone(), attr.clone()))
            }
        }
    },
    as_var! {
        const ALPHABET: ASType::Texte => ASObj::ASTexte("abcdefghijklmnopqrstuvwxyz".into())
    },
    as_var! {
        const CHIFFRES: ASType::Texte => ASObj::ASTexte("0123456789".into())
    },
    as_var! {
        const SYMBOLES: ASType::Texte => ASObj::ASTexte("+-*/%&|!^~<>=()[]{}.,:;".into())
    }
}
