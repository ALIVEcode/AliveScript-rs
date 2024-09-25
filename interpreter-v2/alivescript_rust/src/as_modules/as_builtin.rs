use unindent::Unindent;

use crate::{
    as_cast, as_fonction, as_mod,
    as_obj::{ASDict, ASErreurType, ASObj, ASPaire, ASType},
    as_var, call_methode,
    data::Data,
    union_of,
};

as_mod! {
    BUILTIN_MOD,
    as_fonction! {
        lancer(err: ASType::Erreur) -> ASType::Nul; {
            as_cast!(ASObj::ASErreur(ref err) = err);
            Err(err.err_type().clone())
        }
    },
    as_fonction! {
        Erreur(nom: ASType::Texte, msg: ASType::Texte => ASObj::texte("")) -> ASType::Erreur; {
            as_cast!(ASObj::ASTexte(ref msg) = msg);
            Err(ASErreurType::new_erreur(None, msg.clone()))
        }
    },
    as_fonction! {
        typeErr(msg: ASType::Erreur) -> ASType::Texte; {
            as_cast!(ASObj::ASTexte(ref msg) = msg);
            Err(ASErreurType::new_erreur(None, msg.clone()))
        }
    },
    as_fonction! {
        afficherErr(msg: ASType::any()) -> ASType::Nul; {
            eprintln!("{}", msg);
            Ok(Some(ASObj::ASNul))
        }
    },
    as_fonction! {
        afficher[runner](msg: ASType::any()) -> ASType::Nul; {
            runner.send_data(Data::Afficher(msg.to_string()));
            Ok(Some(ASObj::ASNul))
        }
    },
    as_fonction! {
        typeDe(obj: ASType::any()) -> ASType::Type; {
            Ok(Some(ASObj::ASTypeObj(obj.get_type())))
        }
    },
    as_fonction! {
        typeVar[runner](nomVar: ASType::Texte) -> ASType::Texte; {
            let env = runner.get_env_mut();
            as_cast!(ASObj::ASTexte(nom_var) = nomVar);
            let maybe_var = env.get_var(&nom_var).map(|v| v.0);
            Ok(Some(match maybe_var {
                Some(var) => ASObj::ASTexte(var.get_type().to_string()),
                None => ASObj::ASNul,
            }))
        }
    },
    as_fonction! {
        tailleDe[runner](obj: ASType::iterable()) -> ASType::Entier; {
            if let Some(result) = call_methode!(obj.__taille__() or throw, runner) {
                return result;
            }
            Ok(Some(ASObj::ASEntier(match obj {
                ASObj::ASTexte(t) => t.len(),
                ASObj::ASListe(l) => l.borrow().len(),
                ASObj::ASDict(d) => d.borrow().len(),
                _ => unreachable!()
            } as i64)))
        }
    },
    as_fonction! {
        indexDe[runner](lst: ASType::iterable_ordonne(), obj: ASType::any()) -> ASType::Entier; {
            if let Some(result) = call_methode!(lst.__indexDe__(obj.clone()) or throw, runner) {
                return result;
            }
            Ok(Some(match lst {
                ASObj::ASTexte(t) => t.chars().position(|e| {
                    ASObj::ASTexte(e.to_string()) == obj
                })
                .map(|i| ASObj::ASEntier(i as i64))
                .unwrap_or(ASObj::ASNul),
                ASObj::ASListe(l) => l.borrow().iter()
                    .position(|e| e == &obj)
                    .map(|i| ASObj::ASEntier(i as i64))
                    .unwrap_or(ASObj::ASNul),
                _ => unreachable!()
            }))
        }
    },
    as_fonction! {
        booleen[runner](obj: ASType::any() => ASObj::ASBooleen(true)) -> ASType::Booleen; {
            if let Some(result) = call_methode!(obj.__booleen__(), runner) {
                return result;
            }
            Ok(Some(ASObj::ASBooleen(obj.to_bool())))
        }
    },
    as_fonction! {
        texte[runner](obj: ASType::any() => ASObj::ASTexte("".to_owned())) -> ASType::Texte; {
            if let Some(result) = call_methode!(obj.__texte__(), runner) {
                return result;
            }
            Ok(Some(ASObj::ASTexte(obj.to_string())))
        }
    },
    as_fonction! {
        entier[runner](obj: union_of!(ASType::nombre(), Texte, ClasseInst, Classe) => ASObj::ASEntier(0),
                       base: ASType::Entier => ASObj::ASEntier(10)) -> ASType::Entier; {
            if let Some(result) = call_methode!(obj.__entier__() or throw, runner) {
                return result;
            }
            as_cast!(ASObj::ASEntier(base) = base);
            Ok(Some(match obj {
                ASObj::ASEntier(_) => obj.clone(),
                ASObj::ASDecimal(d) => ASObj::ASEntier(d as i64),
                ASObj::ASTexte(ref s) => {
                    ASObj::ASEntier(i64::from_str_radix(s, base as u32).map_err(|_| {
                        ASErreurType::new_erreur_valeur(Some(format!("\"{}\" ne peut pas se faire convertir en entier", &obj)), obj.clone())
                    })?)
                }
                _ => unreachable!(),
            }))
        }
    },
    as_fonction! {
        decimal[runner](obj: union_of!(ASType::nombre(), Texte, ClasseInst, Classe) => ASObj::ASDecimal(0f64)) -> ASType::Decimal; {
            if let Some(result) = call_methode!(obj.__decimal__() or throw, runner) {
                return result;
            }
            Ok(Some(match obj {
                ASObj::ASEntier(i) => ASObj::ASDecimal(i as f64),
                ASObj::ASDecimal(_) => obj.clone(),
                ASObj::ASTexte(s) => ASObj::ASDecimal(s.parse().unwrap()),
                _ => unreachable!(),
            }))
        }
    },
    as_fonction! {
        liste[runner](obj: ASType::iterable() => ASObj::liste(vec![])) -> ASType::Liste; {
            if let Some(result) = call_methode!(obj.__liste__(), runner) {
                return result;
            }
            let mut obj_iter = obj;
            if let Some(result) = call_methode!(obj_iter.__iter__() -> ASType::iterable(); or throw, runner) {
                obj_iter = result?.unwrap();
            }
            Ok(Some(match obj_iter {
                ASObj::ASTexte(t) => ASObj::liste(
                    t.chars().map(|c| ASObj::ASTexte(c.to_string())).collect(),
                ),
                ASObj::ASListe(l) => ASObj::liste(l.borrow().iter().cloned().collect()),
                ASObj::ASDict(d) => ASObj::liste(
                    d.borrow().items().map(|pair| ASObj::liste(
                                vec![pair.key().clone(), pair.val().clone()]
                            )).collect(),
                ),
                ASObj::ASClasseInst(..) => {
                    let mut liste = vec![];
                    while let Some(prochain) = runner.prochain(&obj_iter)? {
                        liste.push(prochain);
                    }
                    ASObj::liste(liste)
                },
                _ => unreachable!()
            }))
        }
    },
    as_fonction! {
        dict[runner](obj: ASType::iterable() => ASObj::dict(ASDict::default())) -> ASType::Dict; {
            if let Some(result) = call_methode!(obj.__dict__() or throw, runner) {
                return result;
            }
            Ok(Some(match obj {
                ASObj::ASListe(l) => {
                    let mut dict = Vec::with_capacity(l.borrow().len());
                    for e in l.borrow().iter() {
                        let ASObj::ASListe(kv) = e else {
                            return Err(ASErreurType::new_erreur_type(ASType::Liste, e.get_type()));
                        };
                        if kv.borrow().len() != 2 {
                            return Err(ASErreurType::new_erreur_valeur(
                                    Some("Seules les listes de listes à deux arguments peuvent former un dictionnaire".into()),
                                    e.clone()
                                ));
                        }
                        let kv = kv.borrow();
                        dict.push(ASPaire::new(Box::new(kv[0].clone()), Box::new(kv[1].clone())));
                    }
                    ASObj::dict(ASDict::new(dict))
                }
                ASObj::ASDict(d) => ASObj::ASDict(d.clone()),
                _ => unreachable!()
            }))
        }
    },
    as_fonction! {
        info(f: ASType::Fonction, opt: ASType::Dict => ASObj::dict(ASDict::default())) -> ASType::Texte; {
            let ASObj::ASFonc(f) = f else {unreachable!()};
            as_cast!(ASObj::ASDict(opt) = opt);
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
        getAttr[runner](obj: ASType::any(), attr: ASType::Texte, default_val: ASType::any() => ASObj::ASNoValue) -> ASType::any(); {
            if let Some(result) = call_methode!(obj.__getAttr__(attr.clone()), runner) {
                return result;
            }

            as_cast!(ASObj::ASTexte(ref attr_val) = attr);

            let result = match obj {
                ASObj::ASClasseInst(ref inst) => inst
                    .env()
                    .borrow()
                    .get_value(attr_val)
                    .map(|v| Some(v.clone()))
                    .ok_or_else(|| {
                        ASErreurType::new_erreur_access_propriete(obj.clone(), attr_val.clone())
                    }),
                ASObj::ASClasse(ref classe) => classe
                    .static_env()
                    .borrow()
                    .get_value(attr_val)
                    .map(|v| Some(v.clone()))
                    .ok_or_else(|| {
                        ASErreurType::new_erreur_access_propriete(obj.clone(), attr_val.clone())
                    }),
                ASObj::ASDict(ref d) => d
                    .borrow()
                    .get(&attr)
                    .map(|v| Some(v.val().clone()))
                    .ok_or_else(|| ASErreurType::new_erreur_access_propriete(obj.clone(), attr_val.clone())),
                _ => Err(ASErreurType::new_erreur_access_propriete(obj.clone(), attr_val.clone()))
            };
            if result.is_err() && default_val != ASObj::ASNoValue {
                Ok(Some(default_val))
            } else {
                result
            }
        }
    },
    as_fonction! {
        contientAttr(obj: ASType::any(), attr: ASType::Texte) -> ASType::Booleen; {
            as_cast!(ASObj::ASTexte(ref attr_val) = attr);

            match obj {
                ASObj::ASClasseInst(inst) => Ok(Some(
                        ASObj::ASBooleen(inst
                                         .env()
                                         .borrow()
                                         .get_value(attr_val)
                                         .is_some()))),

                ASObj::ASDict(d) => Ok(Some(
                        ASObj::ASBooleen(d
                                         .borrow()
                                         .get(&attr)
                                         .is_some()))),

                _ => Ok(Some(ASObj::ASBooleen(false)))
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
        const ALPHANUM: ASType::Texte => ASObj::ASTexte("abcdefghijklmnopqrstuvwxyz0123456789".into())
    },
    as_var! {
        const SYMBOLES: ASType::Texte => ASObj::ASTexte("+-*/%&|!^~@<>=()[]{}.,:;".into())
    },
    as_var! {
        const BOB: ASType::Texte => ASObj::ASTexte("(~°3°)~".into())
    }
}
