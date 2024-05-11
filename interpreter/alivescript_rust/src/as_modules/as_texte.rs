use crate::{
    as_cast, as_fonction, as_mod,
    as_obj::{ASErreurType, ASObj, ASType},
};

as_mod! {
    TEXTE_MOD,
    as_fonction! {
        maj(txt: ASType::Texte) -> ASType::Texte; {
            as_cast!(ASObj::ASTexte(txt) = txt);
            Ok(Some(ASObj::ASTexte(txt.to_uppercase())))
        }
    },
    as_fonction! {
        minus(txt: ASType::Texte) -> ASType::Texte; {
            as_cast!(ASObj::ASTexte(txt) = txt);
            Ok(Some(ASObj::ASTexte(txt.to_lowercase())))
        }
    },
    as_fonction! {
        car(c: ASType::Entier) -> ASType::Texte; {
            as_cast!(ASObj::ASEntier(i) = c);
            match char::from_u32(i as u32) {
                Some(c) => Ok(Some(ASObj::ASTexte(c.to_string()))),
                None => Err(ASErreurType::new_erreur_valeur(
                    Some(format!("La valeur {} n'est pas un caractère valide", i)),
                    c,
                )),
            }
        }
    },
    as_fonction! {
        couper(txt: ASType::Texte, pattern: ASType::Texte, limite: ASType::optional(ASType::Entier) => ASObj::ASNul) -> ASType::Liste; {
            as_cast!(ASObj::ASTexte(txt) = txt);
            as_cast!(ASObj::ASTexte(pattern) = pattern);

            Ok(Some(ASObj::liste(match limite {
                ASObj::ASNul => txt.split(&pattern).map(ASObj::texte).collect::<Vec<_>>(),
                ASObj::ASEntier(n) => txt.splitn(n as usize + 1, &pattern).map(ASObj::texte).collect::<Vec<_>>(),
                _ => unreachable!(),
            })))
        }
    },
    as_fonction! {
        remplacer(txt: ASType::Texte,
                  pattern: ASType::Texte,
                  remplacement: ASType::Texte,
                  n: ASType::optional(ASType::Entier) => ASObj::ASNul) -> ASType::Texte;
        {
            as_cast!(ASObj::ASTexte(txt) = txt);
            as_cast!(ASObj::ASTexte(pattern) = pattern);
            as_cast!(ASObj::ASTexte(remplacement) = remplacement);
            Ok(Some(match n {
                ASObj::ASNul => ASObj::ASTexte(txt.replace(&pattern, &remplacement)),
                ASObj::ASEntier(n) => {
                    ASObj::ASTexte(txt.replacen(&pattern, &remplacement, n as usize))
                }
                _ => unreachable!(),
            }))
        }
    },
}
